mod hdw;

use crate::hdw::emu::emu_run;

fn main() {

    // remove our logging file
    let _ = std::fs::remove_file("cpu_log.txt");

    // get the arguments the user passes
    let args: Vec<String> = std::env::args().collect();

    // start the emulator with user arguments
    if let Err(e) = emu_run(args) {
        eprintln!("Error: {}", e);
    }
}
