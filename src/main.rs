mod hdw;

use hdw::cart::{self, Cartridge};
use hdw::cpu::{self, CPU};
use std::rc::Rc; // Import Rc for reference countings

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

    // Wrap in RC to pass to CPU
    let gb_cart_rc = Rc::new(gb_cart);
    // Init Components
    let mut emu_cpu = CPU::new(gb_cart_rc.clone());

    // CPU Cycling
    loop {
        emu_cpu.step();
    }
}

// for synchronizing in future
fn emu_cycles(cpu_cycles: i32) {
    // TODO...
}
