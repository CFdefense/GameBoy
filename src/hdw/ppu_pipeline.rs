#[derive(Copy, Clone)]
pub enum FIFOState {
    TILE,
    DATA0,
    DATA1,
    IDLE,
    PUSH,
}

pub struct FIFO {
    pub entries: Vec<u32>,
    pub max_size: usize,
}

impl FIFO {
    pub fn new() -> Self {
        FIFO {
            entries: Vec::new(),
            max_size: 10,
        }
    }
}

pub struct PixelFIFO {
    pub state: FIFOState,
    pub fifo: FIFO,
    pub line_x: u8,
    pub pushed_x: u8,
    pub fetch_x: u8,
    pub bgw_fetch_data: [u8; 3],
    pub fetch_entry_data: [u8; 6],
    pub map_x: u8,
    pub map_y: u8,
    pub tile_y: u8,
    pub fifo_x: u8,
}

impl PixelFIFO {
    pub fn new() -> Self {
        PixelFIFO {
            state: FIFOState::TILE,
            fifo: FIFO::new(),
            line_x: 0,
            pushed_x: 0,
            fetch_x: 0,
            bgw_fetch_data: [0; 3],
            fetch_entry_data: [0; 6],
            map_x: 0,
            map_y: 0,
            tile_y: 0,
            fifo_x: 0,
        }
    }

    pub fn pixel_fifo_push(&mut self, value: u32) {
        if self.fifo.entries.len() < self.fifo.max_size {
            self.fifo.entries.push(value);
        }
    }

    pub fn pixel_fifo_pop(&mut self) -> Option<u32> {
        if self.fifo.entries.is_empty() {
            None
        } else {
            Some(self.fifo.entries.remove(0))
        }
    }

    pub fn fifo_size(&self) -> usize {
        self.fifo.entries.len()
    }

    pub fn pipeline_fifo_reset(&mut self) {
        // Pop all entries from the FIFO
        while self.fifo_size() > 0 {
            self.pixel_fifo_pop();
        }
        self.fifo.entries.clear();
    }
}