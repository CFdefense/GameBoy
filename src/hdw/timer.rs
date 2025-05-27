use core::panic;

use crate::hdw::cpu::CPU;
use crate::hdw::interrupts::Interrupts;
use crate::hdw::debug_timer::log_timer_state;

pub struct Timer {
    pub div: u16, // divider register R/W
    pub tima: u8, // timer counter R/W
    pub tma: u8, // timer modulo R/W
    pub tac: u8, // timer control R/W
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
        // DIV is a 16-bit register and increments continuously.
        // Its full 16-bit value matters for the bit checks below.
        self.div = self.div.wrapping_add(1); 

        // Timer Enable Check: Bit 2 of TAC
        let timer_enabled = (self.tac & (1 << 2)) != 0;

        if timer_enabled {
            let mut tima_should_increment: bool = false;
            // Determine if TIMA should increment based on a falling edge of the selected DIV bit
            match self.tac & 0b11 { // Check lower 2 bits of TAC for clock select
                // Frequencies assuming DIV increments at 4.194304 MHz / 4 = 1.048576 MHz (i.e., timer_tick is called every T-cycle)
                // If timer_tick is called every M-cycle (4 T-cycles), then DIV increments effectively 4x slower from system clock perspective
                // PanDocs frequencies are relative to system clock (e.g. 4096 Hz is SysClock/1024)
                // Let's assume self.div is incremented at 16384 Hz directly as PanDocs states for DIV's own increment rate for simplicity here.
                // The bits of prev_div and self.div used below must correspond to the frequencies in PanDocs.
                // For DIV incrementing at 16384Hz:
                // 4096 Hz   => 16384 / 4   => DIV bit 2 (16384 / 2^2)
                // 262144 Hz => 16384 * 16  => This interpretation is tricky. 
                // Re-evaluating based on PanDocs: DIV increments at 16384Hz. TIMA increments based on selected clock.
                // Clock select values (e.g., 00 for 4096Hz) are the actual TIMA increment frequencies.
                // These frequencies are derived from the system clock, not directly from DIV's own fixed 16384Hz increment rate.
                // The prev_div & self.div checks are about detecting when enough system clock cycles have passed.
                // This typically means these bits are from a shadow counter that *is* running at SystemClock/4 or SystemClock.

                // Let's stick to the common hardware model where DIV is a free-running counter and TAC selects bits from it.
                // Assuming self.div increments every 4 system clocks (1 M-cycle) for simplicity in this example logic.
                // So, self.div effectively increments at ~1MHz. Bit values need to be adjusted for this. 
                // For 4096 Hz with a 1MHz DIV counter: 1048576 / 256 = 4096 => bit 8 of DIV (1MHz / 2^8)
                // For 262144 Hz with a 1MHz DIV counter: 1048576 / 4 = 262144 => bit 2 of DIV (1MHz / 2^2)
                // For 65536 Hz with a 1MHz DIV counter: 1048576 / 16 = 65536 => bit 4 of DIV (1MHz / 2^4)
                // For 16384 Hz with a 1MHz DIV counter: 1048576 / 64 = 16384 => bit 6 of DIV (1MHz / 2^6)
                // The bits in your original code were (9,3,5,7). If DIV increments at system clock / 4, these are:
                // SysClock/(4*2^9) = SysClock/2048 = ~2048 Hz  (for 00)
                // SysClock/(4*2^3) = SysClock/32   = ~131072 Hz (for 01)
                // SysClock/(4*2^5) = SysClock/128  = ~32768 Hz  (for 10)
                // SysClock/(4*2^7) = SysClock/512  = ~8192 Hz   (for 11)
                // These don't match PanDocs frequencies (4096, 262144, 65536, 16384 Hz).
                // The crucial part is that *a specific bit of DIV falling* triggers TIMA.
                // The PanDocs frequencies imply which bit of a (System Clock / 4) counter should be used.
                // 4096 Hz   => (SysClock/4) / 256 => Bit 8 of (SysClock/4 counter)
                // 262144 Hz => (SysClock/4) / 4   => Bit 2 of (SysClock/4 counter)
                // 65536 Hz  => (SysClock/4) / 16  => Bit 4 of (SysClock/4 counter)
                // 16384 Hz  => (SysClock/4) / 64  => Bit 6 of (SysClock/4 counter)
                // Your `self.div` is the DIV register (FF04). PanDocs says it increments at 16384Hz.
                // The TIMA increment is based on other frequencies. This means there must be another internal counter or
                // the bits used from DIV must be interpreted based on the main system clock, not DIV's own value if DIV itself is just FF04.
                // Let's assume your `self.div` is the actual hardware DIV register that increments at 16384Hz.
                // The TIMA clocking mechanism uses bits from a *different*, faster internal counter that reflects system clock progression.
                // For now, I will keep your original bit choices (9,3,5,7) for `prev_div` and `self.div` assuming `self.div` *is* that faster internal counter.
                // If `self.div` is strictly FF04, then this logic needs rethinking based on system clock ticks. 

                0b00 => { tima_should_increment = (prev_div & (1 << 9)) != 0 && (self.div & (1 << 9)) == 0; }, // Was bit 9 for 4096Hz input
                0b01 => { tima_should_increment = (prev_div & (1 << 3)) != 0 && (self.div & (1 << 3)) == 0; }, // Was bit 3 for 262144Hz input
                0b10 => { tima_should_increment = (prev_div & (1 << 5)) != 0 && (self.div & (1 << 5)) == 0; }, // Was bit 5 for 65536Hz input
                0b11 => { tima_should_increment = (prev_div & (1 << 7)) != 0 && (self.div & (1 << 7)) == 0; }, // Was bit 7 for 16384Hz input
                _ => unreachable!(), // Should not happen with tac & 0b11
            }
        
            if tima_should_increment {
                log_timer_state(cpu, "TIMA increment");
                let (new_tima, did_overflow) = self.tima.overflowing_add(1);
                self.tima = new_tima;

                if did_overflow { // TIMA overflowed from 0xFF to 0x00
                    log_timer_state(cpu, "TIMA overflow - loading TMA and requesting interrupt");
                    self.tima = self.tma; // Reload TIMA with TMA
                    cpu.cpu_request_interrupt(Interrupts::TIMER); // Request Timer Interrupt
                }
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

    pub fn timer_read(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!("UNSUPPORTED TIMER READ ADDRESS: {:#06X}", address)
        }
    }
}