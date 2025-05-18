use core::panic;

use crate::hdw::cpu::CPU;
use crate::hdw::interrupts::Interrupts;

pub struct Timer {
    div: u16, // divider register R/W
    tima: u8, // timer counter R/W
    tma: u8, // timer modulo R/W
    tac: u8, // timer control R/W
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0xAC00,
            tima: 0,
            tma: 0,
            tac: 0
        }
    }

    pub fn timer_tick(&mut self, cpu: &mut CPU) {
        let prev_div: u16 = self.div;
        let mut timer_update: bool = false;

        self.div = self.div.wrapping_add(1);

        // Check if timer is enabled (TAC bit 2)
        if (self.tac & (1 << 2)) != 0 {
            match self.tac & 0b11 { // Check lower 2 bits of TAC for clock select
                0b00 => { timer_update = (prev_div & (1 << 9)) != 0 && (self.div & (1 << 9)) == 0; },
                0b01 => { timer_update = (prev_div & (1 << 3)) != 0 && (self.div & (1 << 3)) == 0; },
                0b10 => { timer_update = (prev_div & (1 << 5)) != 0 && (self.div & (1 << 5)) == 0; },
                0b11 => { timer_update = (prev_div & (1 << 7)) != 0 && (self.div & (1 << 7)) == 0; },
                _ => unreachable!(),
            }
        }

        // Check if timer is enabled (TAC bit 2) AND if a TIMA increment is due from DIV logic
        if timer_update && (self.tac & (1 << 2)) != 0 {
            self.tima = self.tima.wrapping_add(1);

            // If TIMA overflows (becomes 00 after increment from FF), reload TMA and request interrupt.
            if self.tima == 0x00 {
                self.tima = self.tma;
                cpu.cpu_request_interrupt(Interrupts::TIMER);
            }
        }
    }

    pub fn timer_write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.div = 0, 
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            _ => panic!("UNSUPPORTED TIMER WRITE ADDRESS: {:#06X}", address)
        }
    }

    pub fn timer_read(&self, address: u16) -> u8 { // Changed to &self
        match address {
            0xFF04 => (self.div >> 8) as u8, // Read upper byte of DIV
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!("UNSUPPORTED TIMER READ ADDRESS: {:#06X}", address)
        }
    }
}