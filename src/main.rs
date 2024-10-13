mod hdw;

use hdw::cart::{self, Cartridge};
use hdw::cpu::{self, CPU};

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
    let mut emu_cpu = CPU::new(gb_cart);

    // CPU Cycling
    loop {
        emu_cpu.step();
    }
}
