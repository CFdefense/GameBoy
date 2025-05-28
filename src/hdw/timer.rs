use core::panic;

use crate::hdw::cpu::CPU;
use crate::hdw::interrupts::Interrupts;

pub struct Timer {
    pub div: u16, // divider register R/W - This is the 16-bit internal counter
    pub tima: u8, // timer counter R/W
    pub tma: u8, // timer modulo R/W
    pub tac: u8, // timer control R/W
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0xAC00, // Initial value often seen in logs, but BIOS sets it. Let's use a common starting point.
            tima: 0,
            tma: 0,
            tac: 0
        }
    }

    pub fn timer_tick(&mut self, cpu: &mut CPU) {
        let prev_div: u16 = self.div;
        self.div = self.div.wrapping_add(1); 

        let tima_should_increment: bool;
            
        // Match reference implementation's bit selection
        match self.tac & 0b11 {
            0b00 => { tima_should_increment = (prev_div & (1 << 9)) != 0 && (self.div & (1 << 9)) == 0; },
            0b01 => { tima_should_increment = (prev_div & (1 << 3)) != 0 && (self.div & (1 << 3)) == 0; },
            0b10 => { tima_should_increment = (prev_div & (1 << 5)) != 0 && (self.div & (1 << 5)) == 0; },
            0b11 => { tima_should_increment = (prev_div & (1 << 7)) != 0 && (self.div & (1 << 7)) == 0; },
            _ => unreachable!(), 
        }
    
        // Check if timer is enabled and we should increment
        if tima_should_increment && (self.tac & (1 << 2)) != 0 {
            self.tima = self.tima.wrapping_add(1);
            
            // Check for overflow ie timer is done
            if self.tima == 0xFF {
                self.tima = self.tma;
                cpu.cpu_request_interrupt(Interrupts::TIMER);
            }
        }
    }

    pub fn timer_write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => { // Writing to DIV (0xFF04) resets the *entire* 16-bit internal counter
                self.div = 0;
            }
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            _ => panic!("UNSUPPORTED TIMER WRITE ADDRESS: {:#06X}", address)
        }
    }

    pub fn timer_read(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8, // Reading DIV (0xFF04) returns the upper 8 bits of the 16-bit internal counter
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!("UNSUPPORTED TIMER READ ADDRESS: {:#06X}", address)
        }
    }
}