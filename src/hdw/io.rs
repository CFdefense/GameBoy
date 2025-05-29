// io.rs
use std::sync::Mutex;
use crate::hdw::debug_timer::log_timer_state;
use crate::hdw::dma::DMA;
use crate::hdw::cpu::CPU;

// Use the EMU_CONTEXT from the emu module
use crate::hdw::emu::EMU_CONTEXT;

// Thread-safe serial data using a Mutex
lazy_static::lazy_static! {
    static ref SERIAL_DATA: Mutex<[u8; 2]> = Mutex::new([0; 2]);
    static ref LY: Mutex<u8> = Mutex::new(0);
}

pub fn io_read(cpu: Option<&CPU>, address: u16) -> u8 {
    let value = match address {
        0xFF01 => {
            if let Ok(data) = SERIAL_DATA.lock() {
                data[0]
            } else {
                println!("Failed to lock SERIAL_DATA for reading");
                0
            }
        },
        0xFF02 => {
            if let Ok(data) = SERIAL_DATA.lock() {
                data[1]
            } else {
                println!("Failed to lock SERIAL_DATA for reading");
                0
            }
        },
        0xFF04..=0xFF07 => {
            if let Some(ctx_arc) = EMU_CONTEXT.get() {
                if let Ok(emu_ctx_lock) = ctx_arc.lock() {
                    let val = emu_ctx_lock.timer.timer_read(address);
                    val
                } else {
                    eprintln!("io_read (timer): Failed to lock EmuContext");
                    0
                }
            } else {
                eprintln!("io_read (timer): Global EmuContext not initialized");
                0
            }
        },
        0xFF0F => {
            if let Some(c) = cpu {
                let val = c.get_int_flags();
                if let Some(ctx_arc) = EMU_CONTEXT.get() {
                    log_timer_state(c, ctx_arc, &format!("Reading INT_FLAGS from FF0F = {:02X}", val));
                }
                val
            } else {
                0
            }
        },
        0xFF44 => {
            if let Ok(mut ly) = LY.lock() {
                *ly += 1;
                *ly
            } else {
                println!("Failed to lock LY for reading");
                0
            }
        }
        _ => {
            println!("IO READ NOT IMPLEMENTED for address: {:04X}", address);
            0
        }
    };
    
    value
}

pub fn io_write(cpu_opt: Option<&mut CPU>, address: u16, value: u8, dma: &mut DMA) {
    match address {
        0xFF01 => {
            if let Ok(mut data) = SERIAL_DATA.lock() {
                data[0] = value;
                return;
            } else {
                println!("Failed to lock SERIAL_DATA for writing to SB");
            }
        },
        0xFF02 => {
            if let Ok(mut data) = SERIAL_DATA.lock() {
                data[1] = value;
                return;
            } else {
                println!("Failed to lock SERIAL_DATA for writing to SC");
            }
        },
        0xFF04..=0xFF07 => {
            if let Some(ctx_arc) = EMU_CONTEXT.get() {
                if let Ok(mut emu_ctx_lock) = ctx_arc.lock() {
                    // Store values we need for logging before modifying timer
                    let old_tac = if address == 0xFF07 { emu_ctx_lock.timer.tac } else { 0 };
                    
                    // Do the actual timer write
                    emu_ctx_lock.timer.timer_write(address, value);
                    
                    // Release the lock before logging
                    drop(emu_ctx_lock);
                    
                    // Now do logging if needed (without holding the lock)
                    if address == 0xFF07 {
                        if let Some(ref cpu) = cpu_opt {
                            println!("TAC Write: Value={:02X}, Old={:02X}, PC={:04X}", value, old_tac, cpu.pc);
                        }
                    }
                } else {
                    eprintln!("io_write (timer): Failed to lock EmuContext");
                }
            } else {
                eprintln!("io_write (timer): Global EmuContext not initialized");
            }
            return;
        },
        0xFF0F => {
            if let Some(c) = cpu_opt {
                // For IF writes, just print directly instead of using log_timer_state
                println!("Writing INT_FLAGS = {:02X}", value);
                c.set_int_flags(value);
            }
            return;
        },
        0xFF46 => {
            dma.dma_start(value);
            println!("DMA STARTED \n");
        }
        _ => {
            println!("IO WRITE NOT IMPLEMENTED for address: {:04X}", address);
        }
    }
}