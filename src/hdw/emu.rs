use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Import your required modules
use crate::hdw::bus::Bus;
use crate::hdw::cart::Cartridge;
use crate::hdw::cpu::CPU;

// Emulator context
pub struct EmuContext {
    running: bool,
    paused: bool,
    pub ticks: u64,
    cpu: CPU,
    debug_limit: Option<u32>,
    instruction_count: u32,
}

// Creating a static emulator context
impl EmuContext {
    fn new(bus: Bus, debug_limit: Option<u32>) -> Self {
        EmuContext {
            running: true,
            paused: false,
            ticks: 0,
            cpu: CPU::new(bus),
            debug_limit,
            instruction_count: 0,
        }
    }

    fn execute_cpu_step(&mut self) -> bool {
        if !self.running || self.paused {
            return true;
        }

        // Execute a CPU step
        let result = self.cpu.step(self.ticks);
        
        // Increment instruction count and check debug limit
        self.instruction_count += 1;
        if let Some(limit) = self.debug_limit {
            if self.instruction_count >= limit {
                println!("\nDebug limit of {} instructions reached. Stopping.", limit);
                self.running = false;
                return false;
            }
        }

        if !result {
            println!("CPU Stopped");
            self.running = false;
        }

        self.ticks += 1;
        result
    }
}

// CPU thread function
fn cpu_run(ctx: Arc<Mutex<EmuContext>>) {
    loop {
        let mut ctx_lock = ctx.lock().unwrap();

        if !ctx_lock.running {
            break;
        }

        // Execute a CPU step
        ctx_lock.execute_cpu_step();
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

    // Initialize Bus and CTX with debug limit
    let bus = Bus::new(cart);
    let ctx = Arc::new(Mutex::new(EmuContext::new(bus, debug_limit)));

    // Spawn a new thread for CPU execution
    let cpu_ctx = Arc::clone(&ctx);
    thread::spawn(move || {
        cpu_run(cpu_ctx);
    });

    // Main loop for UI
    while ctx.lock().unwrap().running {
        thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

pub fn emu_cycles(cpu_cycles: u8) {
    // TODO...
}
