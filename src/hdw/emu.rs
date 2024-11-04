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
    cpu: CPU, // Add CPU instance to context
}

// Creating a static emulator context
impl EmuContext {
    fn new(bus: Bus) -> Self {
        EmuContext {
            running: true,
            paused: false,
            ticks: 0,
            cpu: CPU::new(bus), // Initialize CPU with a Bus
        }
    }

    fn execute_cpu_step(&mut self) -> bool {
        if !self.running || self.paused {
            return true; // Indicate that the step did not execute
        }

        // Execute a CPU step
        let result = self.cpu.step(self.ticks);

        if !result {
            println!("CPU Stopped");
            self.running = false; // Stop the emulator
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
    // Check Submitted Arugemnts
    if args.len() < 2 {
        println!("Usage: emu <rom_file>");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing ROM file argument",
        ));
    }

    // Attempt to create Cartridge
    let rom_path = &args[1];
    let mut cart = Cartridge::new();
    if let Err(e) = cart.load_cart(rom_path) {
        println!("Failed to load ROM file: {}", e);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to load ROM file: {}", e), // Convert the error into a string
        ));
    }
    println!("Cart loaded..");

    // Initialize Bus and CTX
    let bus = Bus::new(cart);
    let ctx = Arc::new(Mutex::new(EmuContext::new(bus)));

    // Spawn a new thread for CPU execution
    let cpu_ctx = Arc::clone(&ctx);
    thread::spawn(move || {
        cpu_run(cpu_ctx);
    });

    // Main loop for UI
    while ctx.lock().unwrap().running {
        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

pub fn emu_cycles(cpu_cycles: u8) {
    // TODO...
}
