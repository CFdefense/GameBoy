#[derive(Copy, Clone)]
pub struct OAMEntry {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub flags: u8,
}

impl OAMEntry {
    pub fn new() -> Self {
        OAMEntry {
            y: 0,
            x: 0,
            tile: 0,
            flags: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        [self.y, self.x, self.tile, self.flags]
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        OAMEntry {
            y: bytes[0],
            x: bytes[1],
            tile: bytes[2],
            flags: bytes[3],
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
            oam_ram: [OAMEntry::new(); 40],
            vram: [0; 0x2000],
        }
    }

    pub fn ppu_tick() {

    }

    pub fn ppu_oam_write(&mut self, mut address: u16, value: u8) {
        if address >= 0xFE00 {
            address -= 0xFE00;
        }
        let entry_index = (address / 4) as usize;
        let byte_index = (address % 4) as usize;
        let mut entry_bytes = self.oam_ram[entry_index].to_bytes();
        entry_bytes[byte_index] = value;
        self.oam_ram[entry_index] = OAMEntry::from_bytes(entry_bytes);
    }

    pub fn ppu_oam_read(&self, mut address: u16) -> u8 {
        if address >= 0xFE00 {
            address -= 0xFE00;
        }
        let entry_index = (address / 4) as usize;
        let byte_index = (address % 4) as usize;
        self.oam_ram[entry_index].to_bytes()[byte_index]
    }

    pub fn ppu_vram_write(&mut self, address: u16, value: u8) {
        self.vram[(address - 0x8000) as usize] = value;
    }

    pub fn ppu_vram_read(&self, address: u16) -> u8 {
        self.vram[(address - 0x8000) as usize]
    }
}
