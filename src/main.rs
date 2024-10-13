mod hdw;

use hdw::cart::{self, cartridge};

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
    let mut gb_cart = cartridge::new();

    if let Err(e) = gb_cart.load_cart(file_path) {
        println!("Error loading cartridge: {}", e);
        return;
    }

    let mut pc: u16 = 0x0000;
    print!("{:#02X} ", gb_cart.read_byte(0x07FBC));
    pc += 1;

    /*
    // Init Components
    let mut emu_cpu = CPU::new();

    // CPU Cycling
    loop {
        emu_cpu.step();
    }
     */
}
