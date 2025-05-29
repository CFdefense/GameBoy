use crate::hdw::bus::BUS;

#[derive(Default)]
pub struct DMA {
    pub active: bool,
    pub current_byte: u8,
    pub byte_value: u8,
    pub start_delay: u8,
}

impl DMA {

    pub fn new() -> Self {
        Default::default()
    }

    pub fn dma_start(&mut self, start: u8) {
        self.active = true;
        self.current_byte = 0;
        self.byte_value = start;
        self.start_delay = 2;
    }

    pub fn dma_tick(&mut self, bus: &mut BUS) -> bool {
        if !self.active {
            return false;
        }

        if self.start_delay > 0 {
            self.start_delay -= 1;
            return false;
        }
        
        let value = bus.read_byte(None, ((self.byte_value as u16) * 0x100) + self.current_byte as u16);
        bus.ppu.ppu_oam_write(self.current_byte as u16, value);
        
        self.current_byte = self.current_byte + 1;
        self.active = self.current_byte < 0xA0;
        self.active
    }

    pub fn dma_transferring(&self) -> bool {
        self.active
    }
}

