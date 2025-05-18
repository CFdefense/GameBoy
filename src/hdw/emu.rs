use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Import your required modules
use crate::hdw::bus::Bus;
use crate::hdw::cart::Cartridge;
use crate::hdw::cpu::CPU;
use crate::hdw::timer::Timer;
use crate::hdw::ui::{UI, ui_handle_events};

use once_cell::sync::OnceCell;

// Global static EmuContext holder
static EMU_CONTEXT: OnceCell<Arc<Mutex<EmuContext>>> = OnceCell::new();

// Emulator context
pub struct EmuContext {
    pub running: bool,
    pub paused: bool,
    pub die: bool,
    pub ticks: u64,
    pub cpu: Option<Arc<Mutex<CPU>>>,
    debug_limit: Option<u32>,
    instruction_count: u32,
    timer: Timer,
}

impl EmuContext {
    fn new(debug_limit: Option<u32>) -> Self {
        EmuContext {
            running: true,
            paused: false,
            die: false,
            ticks: 0,
            cpu: None,
            debug_limit,
            instruction_count: 0,
            timer: Timer::new(),
        }
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }
}

// Function to initialize the global EmuContext reference.
// This should be called once from emu_run after ctx is created and configured.
pub fn init_global_emu_context(ctx: Arc<Mutex<EmuContext>>) {
    // Attempt to set the context. If it's already set, .set() will return an Err.
    // We are choosing to do nothing if it's already initialized.
    let _ = EMU_CONTEXT.set(ctx); 
}

// CPU thread function
fn cpu_run(cpu: Arc<Mutex<CPU>>, ctx: Arc<Mutex<EmuContext>>) {
    while ctx.lock().unwrap().running {
        if ctx.lock().unwrap().paused {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        let ticks;
        {
            let ctx_lock = ctx.lock().unwrap();
            ticks = ctx_lock.ticks;
        }

        // Execute a CPU step
        let result = {
            let mut cpu_lock = cpu.lock().unwrap();
            cpu_lock.step(ticks)
        };

        if !result {
            println!("CPU Stopped");
            ctx.lock().unwrap().running = false;
            break;
        }

        // Update ticks and check debug limit
        {
            let mut ctx_lock = ctx.lock().unwrap();
            ctx_lock.instruction_count += 1;
            
            if let Some(limit) = ctx_lock.debug_limit {
                if ctx_lock.instruction_count >= limit {
                    println!("\nDebug limit of {} instructions reached. Stopping.", limit);
                    ctx_lock.running = false;
                    break;
                }
            }
        }
    }
}

// Main Emulator Startup Function
pub fn emu_run(args: Vec<String>) -> io::Result<()> {
    // Parse command line arguments
    let mut rom_path = None;
    let mut debug_limit = None;
    let mut i = 1;
    
    while i < args.len() {
        match args[i].as_str() {
            "--debug" => {
                if i + 1 < args.len() {
                    debug_limit = Some(args[i + 1].parse().expect("Debug limit must be a number"));
                    i += 2;
                    continue;
                }
            }
            path => {
                rom_path = Some(path);
                i += 1;
            }
        }
    }

    // Check if ROM path was provided
    let rom_path = rom_path.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Missing ROM file argument")
    })?;

    // Attempt to create Cartridge
    let mut cart = Cartridge::new();
    if let Err(e) = cart.load_cart(rom_path) {
        println!("Failed to load ROM file: {}", e);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to load ROM file: {}", e),
        ));
    }
    println!("Cart loaded..");

    // Initialize UI
    let ui_result = UI::new();
    if let Err(e) = &ui_result {
        println!("Failed to initialize UI: {}", e);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to initialize UI: {}", e),
        ));
    }
    let mut ui = ui_result.unwrap();

    // Initialize Bus and CPU
    let bus = Bus::new(cart);
    let cpu = Arc::new(Mutex::new(CPU::new(bus)));
    
    // Initialize context
    let ctx = Arc::new(Mutex::new(EmuContext::new(debug_limit)));
    
    // Store CPU reference in context
    ctx.lock().unwrap().cpu = Some(Arc::clone(&cpu));

    // Initialize the global context reference
    init_global_emu_context(Arc::clone(&ctx));

    // Spawn a new thread for CPU execution
    let cpu_thread_ctx = Arc::clone(&ctx);
    let cpu_thread_cpu = Arc::clone(&cpu);
    
    let cpu_thread = thread::spawn(move || {
        cpu_run(cpu_thread_cpu, cpu_thread_ctx);
    });

    // Main loop for UI and event handling
    while !ctx.lock().unwrap().die {
        // Handle UI events
        let mut cpu_lock = cpu.lock().unwrap();
        let continue_running = ui.handle_events(&mut cpu_lock);
        drop(cpu_lock);
        
        if !continue_running {
            println!("UI requested shutdown");
            ctx.lock().unwrap().die = true;
            ctx.lock().unwrap().running = false;
            break;
        }
        
        thread::sleep(Duration::from_millis(10));
    }

    // Wait for CPU thread to finish
    if let Err(e) = cpu_thread.join() {
        println!("Error joining CPU thread: {:?}", e);
    }

    // Make sure to properly signal the CPU thread to stop
    ctx.lock().unwrap().running = false;

    Ok(())
}

// Function to increment EmuContext ticks based on CPU M-cycles.
// Each M-cycle is typically 4 T-cycles (clock ticks).
pub fn emu_cycles(cpu_m_cycles: u8) {
    if let Some(ctx_arc) = EMU_CONTEXT.get() {
        let t_cycles_to_add = cpu_m_cycles as u64 * 4; // Calculate total T-cycles to add
        if let Ok(mut emu_ctx_lock) = ctx_arc.lock() {
            emu_ctx_lock.ticks += t_cycles_to_add;
        } else {
            // Failed to lock, this is a potential issue if it happens often
            eprintln!("emu_cycles: Failed to lock EmuContext.");
        }
    } else {
        // This means init_global_emu_context was not called before emu_cycles.
        // This is a programming error.
        eprintln!("emu_cycles: Global EmuContext not initialized. Call init_global_emu_context first.");
        // Depending on desired robustness, you might panic here.
        // panic!("emu_cycles: Global EmuContext not initialized."); 
    }
}
