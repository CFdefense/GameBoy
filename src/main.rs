#[path = "hdw/cpu.rs"]
mod cpu;

use cpu::CPU;
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

    // Load Cart

    // Init Components
    let mut emu_cpu = CPU::new();

    // CPU Cycling
    loop {
        emu_cpu.step();
    }
}
