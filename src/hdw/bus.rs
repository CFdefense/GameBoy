/*

    Gameboy Memory Bus

    Most of the function here is to reroute referencing of data
    0x0000 - 0x3FFF : ROM Bank 0
    0x4000 - 0x7FFF : ROM Bank 1 - Switchable
    0x8000 - 0x97FF : CHR RAM
    0x9800 - 0x9BFF : BG Map 1
    0x9C00 - 0x9FFF : BG Map 2
    0xA000 - 0xBFFF : Cartridge RAM
    0xC000 - 0xCFFF : RAM Bank 0
    0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
    0xE000 - 0xFDFF : Reserved - Echo RAM
    0xFE00 - 0xFE9F : Object Attribute Memory
    0xFEA0 - 0xFEFF : Reserved - Unusable
    0xFF00 - 0xFF7F : I/O Registers
    0xFF80 - 0xFFFE : Zero Page

*/

use super::cart::Cartridge;
use crate::hdw::ram::RAM;

pub struct Bus {
    cart: Cartridge,
    ram: RAM,
}

impl Bus {
    // Consructor
    pub fn new(cart: Cartridge) -> Self {
        Bus {
            // initialize vars
            cart,
            ram: RAM::new(),
        }
    }

    // Function to return a byte at an address
    pub fn read_byte(&self, address: u16) -> u8 {
        if address < 0x8000 {
            // ROM DATA
            self.cart.read_byte(address)
        } else if address < 0xA000 {
            // Char/Map Data
            panic!("MEM NOT IMPL")
        } else if address < 0xC000 {
            // Cartridge RAM
            self.cart.read_byte(address)
        } else if address < 0xE000 {
            // WRAM
            self.ram.wram_read(address)
        } else if address < 0xFE00 {
            // Reserved Echo RAM
            0
        } else if address < 0xFEA0 {
            // OAM
            panic!("MEM NOT IMPL")
        } else if address < 0xFF00 {
            // Reserved Unusable
            0
        } else if address < 0xFF80 {
            // IO Registers
            panic!("MEM NOT IMPL")
        } else if address == 0xFFFF {
            // CPU ENABLE
            panic!("MEM NOT IMPL")
        } else {
            // HRAM (Zero Page)
            self.ram.hram_read(address)
        }
    }

    // Function to write byte to correct place
    pub fn write_byte(&mut self, address: u16, value: u8) {
        // Need to filter destination of byte and write to there
        if address < 0x8000 {
            // ROM DATA
            self.cart.write_byte(address, value);
        } else if address < 0xA000 {
            // Char/Map Data
            panic!("MEM NOT IMPL")
        } else if address < 0xC000 {
            // EXT RAM
            self.cart.write_byte(address, value);
        } else if address < 0xE000 {
            // WRAM
            self.ram.wram_write(address, value);
        } else if address < 0xFE00 {
            // Reserved ECHO RAM
        } else if address < 0xFEA0 {
            // OAM RAM
            panic!("MEM NOT IMPL")
        } else if address < 0xFF00 {
            // Reserved Unusuable
        } else if address < 0xFF80 {
            // IO Registers
            panic!("MEM NOT IMPL")
        } else if address == 0xFFFF {
            // CPU ENABLE
            panic!("MEM NOT IMPL")
        } else {
            // HRAM
            self.ram.hram_write(address, value);
        }
    }
}
