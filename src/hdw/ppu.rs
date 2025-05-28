pub struct OAMEntry {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub flags: u8,
}

impl OAMEntry {
    pub fn new() -> Self {
        OAMEntry {
        }
    }
}

pub struct PPU {
    pub oam_ram: [OAMEntry; 40],
    pub vram: [u8; 0x2000],
}

impl PPU {
    pub fn new() -> Self {
        PPU {

        }
    }

    pub fn ppu_tick() {

    }

    pub fn ppu_oam_write(address: u16, value: u8) {
        if address >= 0xFE00 {
            address = address - 0xFE00
        }
    }

    pub fn ppu_oam_read(address: u16) -> u8 {

    }

    pub fn ppu_vram_write(address: u16, value: u8) {

    }

    pub fn ppu_vram_read(address: u16) -> u8 {

    }

}
