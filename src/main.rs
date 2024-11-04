mod hdw;

use crate::hdw::emu::emu_run;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = emu_run(args) {
        eprintln!("Error: {}", e);
    }
}
