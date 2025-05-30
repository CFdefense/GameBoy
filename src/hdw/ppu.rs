use crate::hdw::lcd::{LCD, LcdMode, StatSrc};
use crate::hdw::interrupts::Interrupts;
use crate::hdw::ui::{get_ticks, delay};

#[derive(Copy, Clone)]
pub struct OAMEntry {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub flags: u8,
}

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u32 = 456;
const YRES: u8 = 144;
const XRES: u8 = 160;

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
    pub ly: u8,           // Current scanline
    pub current_frame: u32, // Current frame number
    pub video_buffer: Vec<u32>, // Video buffer for frame (YRES * XRES * 32-bit pixels)
    pub line_ticks: u32,  // Ticks for current scanline
    pub lcd: LCD,         // LCD controller
    
    // Frame timing
    target_frame_time: u32,
    prev_frame_time: u64,
    start_timer: u64,
    frame_count: u32,
}

impl PPU {
    pub fn new() -> Self {
        let mut ppu = PPU {
            oam_ram: [OAMEntry::new(); 40],
            vram: [0; 0x2000],
            ly: 0,
            line_ticks: 0,
            current_frame: 0,
            video_buffer: vec![0; (YRES as usize) * (XRES as usize)], // Allocate frame buffer
            lcd: LCD::new(),
            
            // Frame timing (60 FPS)
            target_frame_time: 1000 / 60,
            prev_frame_time: 0,
            start_timer: 0,
            frame_count: 0,
        };

        // Set initial LCD mode to OAM
        ppu.lcd.lcds_mode_set(LcdMode::OAM);
        
        ppu
    }

    fn increment_ly(&mut self) -> Vec<Interrupts> {
        let mut interrupts = Vec::new();
        self.ly += 1;

        if self.ly == self.lcd.lyc {
            self.lcd.lcds_lyc_set(true);
            
            if self.lcd.lcds_stat_int(StatSrc::LYC) {
                interrupts.push(Interrupts::LCDSTAT);
            }
        } else {
            self.lcd.lcds_lyc_set(false);
        }
        
        interrupts
    }

    fn ppu_mode_oam(&mut self) -> Vec<Interrupts> {
        if self.line_ticks >= 80 {
            self.lcd.lcds_mode_set(LcdMode::Transfer);
        }
        Vec::new()
    }

    fn ppu_mode_xfer(&mut self) -> Vec<Interrupts> {
        if self.line_ticks >= 80 + 172 {
            self.lcd.lcds_mode_set(LcdMode::HBlank);
        }
        Vec::new()
    }

    fn ppu_mode_vblank(&mut self) -> Vec<Interrupts> {
        let mut interrupts = Vec::new();
        
        if self.line_ticks >= TICKS_PER_LINE {
            interrupts.extend(self.increment_ly());

            if self.ly >= LINES_PER_FRAME {
                self.lcd.lcds_mode_set(LcdMode::OAM);
                self.ly = 0;
            }

            self.line_ticks = 0;
        }
        
        interrupts
    }

    fn ppu_mode_hblank(&mut self) -> Vec<Interrupts> {
        let mut interrupts = Vec::new();
        
        if self.line_ticks >= TICKS_PER_LINE {
            interrupts.extend(self.increment_ly());

            if self.ly >= YRES {
                self.lcd.lcds_mode_set(LcdMode::VBlank);

                interrupts.push(Interrupts::VBLANK);

                if self.lcd.lcds_stat_int(StatSrc::VBlank) {
                    interrupts.push(Interrupts::LCDSTAT);
                }

                self.current_frame += 1;

                // Calculate FPS
                let end = get_ticks();
                let frame_time = end - self.prev_frame_time;

                if frame_time < self.target_frame_time as u64 {
                    delay((self.target_frame_time as u64 - frame_time) as u32);
                }

                if end - self.start_timer >= 1000 {
                    let fps = self.frame_count;
                    self.start_timer = end;
                    self.frame_count = 0;

                    println!("FPS: {}", fps);
                }

                self.frame_count += 1;
                self.prev_frame_time = get_ticks();
            } else {
                self.lcd.lcds_mode_set(LcdMode::OAM);
            }

            self.line_ticks = 0;
        }
        
        interrupts
    }

    pub fn ppu_tick(&mut self) -> Vec<Interrupts> {
        self.line_ticks += 1;
        
        match self.lcd.lcds_mode() {
            LcdMode::OAM => self.ppu_mode_oam(),
            LcdMode::Transfer => self.ppu_mode_xfer(),
            LcdMode::VBlank => self.ppu_mode_vblank(),
            LcdMode::HBlank => self.ppu_mode_hblank(),
        }
    }

    pub fn update_lcd_ly(&mut self) {
        self.lcd.ly = self.ly;
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
