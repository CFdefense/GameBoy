mod hdw;

use hdw::bus::Bus;
use hdw::cart::Cartridge;
use hdw::cpu::CPU;

use std::io::stdin;

/* Implement Initialization of Gameboy Here


-- Components --
|  Cart |
|  CPU  |
|  MEM  |
|  GPU  |
|  TMR  |

Include Here
Recieve Cart
Load Cart
Init Components
CPU CYCLE

*/

fn main() {
    // Recieve Cart
    let mut file_path = String::new();
    println!("Enter ROM Path");
    stdin().read_line(&mut file_path).expect("Didn't Get Input");

    // Remove newline character from input
    let file_path = file_path.trim();

    // Load Cart
    let mut gb_cart = Cartridge::new();

    if let Err(e) = gb_cart.load_cart(file_path) {
        println!("Error loading cartridge: {}", e);
        return;
    }

    // Init Components
    let emu_bus = Bus::new(gb_cart);
    let mut emu_cpu = CPU::new(emu_bus);

    // CPU Cycling
    loop {
        emu_cpu.step();
    }
}

// for synchronizing in future
fn emu_cycles(cpu_cycles: i32) {
    // TODO...
}
