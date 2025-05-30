use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Import your required modules
use crate::hdw::bus::BUS;
use crate::hdw::cart::Cartridge;
use crate::hdw::cpu::CPU;
use crate::hdw::timer::Timer;
use crate::hdw::ui::{UI, ui_handle_events};

use once_cell::sync::OnceCell;

// Global static EmuContext holder
pub static EMU_CONTEXT: OnceCell<Arc<Mutex<EmuContext>>> = OnceCell::new();

// Emulator context
pub struct EmuContext {
    pub running: bool,
    pub paused: bool,
    pub die: bool,
    pub ticks: u64,
    pub cpu: Option<Arc<Mutex<CPU>>>,
    pub bus: BUS,
    debug_limit: Option<u32>,
    instruction_count: u32,
    pub timer: Timer,
    pub debug: bool,
}

impl EmuContext {
    pub fn new(debug_limit: Option<u32>, debug: bool) -> Self {
        EmuContext {
            running: true,
            paused: false,
            die: false,
            ticks: 0,
            cpu: None,
            bus: BUS::new(Cartridge::new()),
            debug_limit,
            instruction_count: 0,
            timer: Timer::new(),
            debug,
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
        
        // Execute a CPU step
        let result = {
            let mut cpu_lock = cpu.lock().unwrap();
            cpu_lock.step(Arc::clone(&ctx)) // Pass a clone of the Arc to step
        };

        if !result {
            println!("CPU Stopped");
            ctx.lock().unwrap().running = false;
            break;
        }

        // Update instruction count and check debug limit
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
    let mut debug = false;
    let mut i = 1;
    
    while i < args.len() {
        match args[i].as_str() {
            "--debug-limit" => {
                if i + 1 < args.len() {
                    debug_limit = Some(args[i + 1].parse().expect("Debug limit must be a number"));
                    i += 2;
                    continue;
                }
            }
            "--debug" => {
                debug = true;
                i += 1;
                continue;
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

    // Initialize context first
    let ctx = Arc::new(Mutex::new(EmuContext::new(debug_limit, debug)));
    
    // Initialize Bus and CPU
    let bus = BUS::new(cart);
    let cpu = Arc::new(Mutex::new(CPU::new(bus, debug)));
    
    // Update context with CPU and bus
    {
        let mut ctx_lock = ctx.lock().unwrap();
        ctx_lock.cpu = Some(Arc::clone(&cpu));
        ctx_lock.bus = BUS::new(Cartridge::new()); // Create new bus instance for context
    }

    // Initialize the global context reference
    init_global_emu_context(Arc::clone(&ctx));

    // Spawn a new thread for CPU execution
    let cpu_thread_ctx = Arc::clone(&ctx);
    let cpu_thread_cpu = Arc::clone(&cpu);
    
    let cpu_thread = thread::spawn(move || {
        cpu_run(cpu_thread_cpu, cpu_thread_ctx);
    });

    // Main loop for UI and event handling
    let mut prev_frame = 0;
    
    while !ctx.lock().unwrap().die {
        // Small delay
        thread::sleep(Duration::from_millis(1));
        
        // Handle UI events
        let mut cpu_lock = cpu.lock().unwrap();
        let continue_running = ui.handle_events(&mut cpu_lock);
        
        // Check if frame has changed and update UI
        let current_frame = cpu_lock.bus.ppu.current_frame;
        if prev_frame != current_frame {
            // Frame has changed, update UI
            // Note: In the future, this could call a specific ui.update() method
            // For now, the handle_events already updates the UI
            prev_frame = current_frame;
        }
        
        drop(cpu_lock);
        
        if !continue_running {
            println!("UI requested shutdown");
            ctx.lock().unwrap().die = true;
            ctx.lock().unwrap().running = false;
            break;
        }
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
// CPU reference is passed directly to avoid double-locking issues.
pub fn emu_cycles(cpu: &mut CPU, cpu_m_cycles: u8) {
    if let Some(ctx_arc) = EMU_CONTEXT.get() {
        let t_cycles_to_add = cpu_m_cycles as u64 * 4; // Calculate total T-cycles to add
        if let Ok(mut emu_ctx_lock) = ctx_arc.lock() {
            for _ in 0..t_cycles_to_add {
                emu_ctx_lock.ticks += 1;
                // Call timer_tick with the passed CPU reference
                emu_ctx_lock.timer.timer_tick(cpu);
                // Tick PPU for every T-cycle and handle interrupts
                let ppu_interrupts = cpu.bus.ppu.ppu_tick();
                for interrupt in ppu_interrupts {
                    cpu.bus.interrupt_controller.request_interrupt(interrupt);
                }
            }
            // Update LCD LY register from PPU
            cpu.bus.ppu.update_lcd_ly();
            emu_ctx_lock.bus.tick_dma(); // tick once per 4 t-cycles
        } else {
            eprintln!("emu_cycles: Failed to lock EmuContext.");
        }
    } else {
        panic!("emu_cycles: Global EmuContext not initialized. Call init_global_emu_context first.");
    }
}

pub fn is_debug_enabled() -> bool {
    if let Some(ctx_arc) = EMU_CONTEXT.get() {
        if let Ok(ctx_lock) = ctx_arc.lock() {
            return ctx_lock.debug;
        }
    }
    false
}
