mod hdw;

use crate::hdw::emu::emu_run;

fn main() {
    //let args: Vec<String> = std::env::args().collect();
    let args: Vec<String> = vec![
        String::from("/home/cfdefence/Documents/Github/GameBoy/src/roms/07-jr,jp,call,ret,rst.gb"),
        String::from("/home/cfdefence/Documents/Github/GameBoy/src/roms/07-jr,jp,call,ret,rst.gb"),
    ];
    if let Err(e) = emu_run(args) {
        eprintln!("Error: {}", e);
    }
}
