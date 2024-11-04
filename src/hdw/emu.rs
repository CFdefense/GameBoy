use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Import your required modules
use crate::hdw::bus::Bus;
use crate::hdw::cart::Cartridge;
use crate::hdw::cpu::CPU;

// Emulator context
struct EmuContext {
    running: bool,
    paused: bool,
    ticks: u64,
    cpu: CPU, // Add CPU instance to context
}

// Creating a static emulator context
impl EmuContext {
    fn new(bus: Bus) -> Self {
        EmuContext {
            running: false,
            paused: false,
            ticks: 0,
            cpu: CPU::new(bus), // Initialize CPU with a Bus
        }
    }
}

// CPU thread function
fn cpu_run(ctx: Arc<Mutex<EmuContext>>) {
    let mut ctx = ctx.lock().unwrap();
    ctx.running = true;
    println!("CPU thread started...");

    while ctx.running {
        if ctx.paused {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        // Execute a CPU step and check if it succeeded
        if !ctx.cpu.step() {
            // Assuming `step` is a method in `CPU`
            println!("CPU Stopped");
            ctx.running = false; // Stop the emulator
        }

        ctx.ticks += 1;
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
