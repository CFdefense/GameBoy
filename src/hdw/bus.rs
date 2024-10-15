/*

    Gameboy Memory Bus

    Most of the function here is to reroute referencing of data


    0000	3FFF	16 KiB ROM bank 00	From cartridge, usually a fixed bank
    4000	7FFF	16 KiB ROM Bank 01–NN	From cartridge, switchable bank via mapper (if any)
    8000	9FFF	8 KiB Video RAM (VRAM)	In CGB mode, switchable bank 0/1
    A000	BFFF	8 KiB External RAM	From cartridge, switchable bank if any
    C000	CFFF	4 KiB Work RAM (WRAM)
    D000	DFFF	4 KiB Work RAM (WRAM)	In CGB mode, switchable bank 1–7
    E000	FDFF	Echo RAM (mirror of C000–DDFF)	Nintendo says use of this area is prohibited.
    FE00	FE9F	Object attribute memory (OAM)
    FEA0	FEFF	Not Usable	Nintendo says use of this area is prohibited.
    FF00	FF7F	I/O Registers
    FF80	FFFE	High RAM (HRAM)
    FFFF	FFFF	Interrupt Enable register (IE)

*/

use super::cart::Cartridge;
use std::rc::Rc;

pub struct Bus {
    pub bus: [u8; 0xFFFF],
    cart: Rc<Cartridge>,
}

impl Bus {
    // Consructor
    pub fn new(cart: Rc<Cartridge>) -> Self {
        Bus {
            // initialize vars
            bus: [0; 0xFFFF],
            cart,
        }
    }

    // Function to return a byte at an address
    pub fn read_byte(&self, address: u16) -> u8 {
        if address < 0x8000 {
            // ROM DATA
            self.cart.read_byte(address)
        } else {
            // Handle other memory areas or return 0 for now
            0
        }
    }

    // Function to write byte to correct place
    pub fn write_byte(&mut self, address: u16, value: u8) {
        // Need to filter destination of byte and write to there
    }

    // Function to read next byte
    pub fn read_next_byte(&self) -> u8 {
        todo!()
    }
}
