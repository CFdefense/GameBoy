#[path = "hdw/cpu.rs"] mod cpu;

use std::io::{stdin, stdout, Read, Write};
use cpu::CPU;


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
    Println!("Enter ROM Path");
    stdin().read_line(&mut file_path).expect("Didn't Get Input");

}
