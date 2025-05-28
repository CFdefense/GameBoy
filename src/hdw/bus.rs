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
use crate::hdw::cpu::CPU;
use crate::hdw::ram::RAM;
use crate::hdw::io::{io_read,io_write};

pub struct Bus {
    cart: Cartridge,
    ram: RAM,
}

impl Bus {
    // Constructor
    pub fn new(cart: Cartridge) -> Self {
        Bus {
            cart,
            ram: RAM::new(),
        }
    }

    // Function to return a byte at an address
    pub fn read_byte(&self, cpu: Option<&CPU>, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cart.read_byte(address),  // ROM Banks
            0x8000..=0x9FFF => {  // Char/Map Data
                print!("\nMEM NOT IMPL <A");
                0
            },
            0xA000..=0xBFFF => self.cart.read_byte(address),  // Cartridge RAM
            0xC000..=0xDFFF => self.ram.wram_read(address),   // WRAM
            0xE000..=0xFDFF => 0,  // Reserved Echo RAM
            0xFE00..=0xFE9F => {   // OAM
                print!("\nMEM NOT IMPL <FEA0");
                0
            },
            0xFEA0..=0xFEFF => 0,  // Reserved Unusable
            0xFF00..=0xFF7F => io_read(cpu, address),  // IO Registers
            0xFFFF => match cpu {   // Interrupt Enable Register
                Some(cpu) => cpu.get_ie_register(),
                None => panic!("BUS: Attempted to read IE register without CPU reference")
            },
            _ => self.ram.hram_read(address)  // HRAM (Zero Page)
        }
    }

    // Function to write byte to correct place
    pub fn write_byte(&mut self, cpu: Option<&mut CPU>, address: u16, value: u8) {
        // Log IO writes
        if (0xFF00..=0xFF7F).contains(&address) {
            println!("BUS_WRITE_IO: Addr={:04X}, Val={:02X}", address, value);
        }

        match address {
            0x0000..=0x7FFF => self.cart.write_byte(address, value),  // ROM Banks
            0x8000..=0x9FFF => {  // Char/Map Data
                print!("\nMEM NOT IMPL <A000");
            },
            0xA000..=0xBFFF => self.cart.write_byte(address, value),  // Cartridge RAM
            0xC000..=0xDFFF => self.ram.wram_write(address, value),   // WRAM
            0xE000..=0xFDFF => (),  // Reserved Echo RAM
            0xFE00..=0xFE9F => {    // OAM RAM
                print!("\nMEM NOT IMPL <FEA0");
            },
            0xFEA0..=0xFEFF => (),  // Reserved Unusable
            0xFF00..=0xFF7F => {    // IO Registers
                println!("BUS_WRITE_IO: Dispatching to io_write for Addr={:04X}, Val={:02X}", address, value);
                io_write(cpu, address, value);
            },
            0xFFFF => match cpu {    // Interrupt Enable Register
                Some(cpu) => cpu.set_ie_register(value),
                None => panic!("BUS: Attempted to write IE register without CPU reference")
            },
            _ => self.ram.hram_write(address, value)  // HRAM
        }
    }
}
