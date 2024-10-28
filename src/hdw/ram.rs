use core::panic;

pub struct RAM {
    wram: [u8; 0x2000],
    hram: [u8; 0x80],
}

impl RAM {
    // Constructor
    pub fn new() -> Self {
        RAM {
            wram: [0; 0x2000],
            hram: [0; 0x80],
        }
    }

    // Method to read from wram
    pub fn wram_read(&self, address: u16) -> u8 {
        let offset_address = address - 0xC000;

        if offset_address >= 0x2000 {
            panic!("INVALID WRAM ADDRESS")
        }

        self.wram[offset_address as usize]
    }

    // Method to write to wram
    pub fn wram_write(&mut self, address: u16, value: u8) {
        let offset_address = address - 0xC000;

        self.wram[offset_address as usize] = value;
    }

    // Method to read from hram
    pub fn hram_read(&self, address: u16) -> u8 {
        let offset_address = address - 0xFF80;

        self.hram[offset_address as usize]
    }

    // Method to write to hram
    pub fn hram_write(&mut self, address: u16, value: u8) {
        let offset_address = address - 0xFF80;

        self.hram[offset_address as usize] = value;
    }
}
