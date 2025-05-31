// io.rs
use std::sync::Mutex;
use crate::hdw::debug_timer::log_timer_state;
use crate::hdw::dma::DMA;
use crate::hdw::cpu::CPU;
use crate::hdw::interrupts::InterruptController;
use crate::hdw::ppu::PPU;
use crate::hdw::gamepad::GamePad;
use crate::hdw::apu::AudioSystem;

// Use the EMU_CONTEXT from the emu module
use crate::hdw::emu::EMU_CONTEXT;

// Thread-safe serial data using a Mutex
lazy_static::lazy_static! {
    static ref SERIAL_DATA: Mutex<[u8; 2]> = Mutex::new([0; 2]);
}

pub fn io_read(cpu: Option<&CPU>, address: u16, interrupt_controller: &InterruptController, ppu: &PPU, gamepad: &GamePad, apu: &AudioSystem) -> u8 {
    let value = match address {
        0xFF00 => {
            gamepad.get_gamepad_output()
        },
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
            let val = interrupt_controller.get_int_flags();
            if let Some(c) = cpu {
                if let Some(ctx_arc) = EMU_CONTEXT.get() {
                    if crate::hdw::emu::is_debug_enabled() {
                        log_timer_state(c, ctx_arc, &format!("Reading INT_FLAGS from FF0F = {:02X}", val));
                    }
                }
            }
            val
        },
        0xFF10..=0xFF3F => {
            // Sound registers
            apu.read_register(address)
        },
        0xFF40..=0xFF4B => {
            ppu.lcd.lcd_read(address)
        },
        _ => {
            println!("IO READ NOT IMPLEMENTED for address: {:04X}", address);
            0
        }
    };
    
    value
}

pub fn io_write(address: u16, value: u8, dma: &mut DMA, interrupt_controller: &mut InterruptController, ppu: &mut PPU, gamepad: &mut GamePad, apu: &mut AudioSystem) {
    match address {
        0xFF00 => {
            gamepad.gamepad_set_selection(value);
        },
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
                    if address == 0xFF07 { emu_ctx_lock.timer.tac } else { 0 };
                    
                    // Do the actual timer write
                    emu_ctx_lock.timer.timer_write(address, value);
                    
                    // Release the lock before logging
                    drop(emu_ctx_lock);
                }
            } else {
                eprintln!("io_write (timer): Global EmuContext not initialized");
            }
            return;
        },
        0xFF0F => {
            interrupt_controller.set_int_flags(value);
            return;
        },
        0xFF10..=0xFF3F => {
            // Sound registers
            apu.write_register(address, value);
        },
        0xFF40..=0xFF4B => {
            let result = ppu.lcd.lcd_write(address, value);
            
            // if lcd write returns a value we know to initiate a dma transfer
            if let Some(dma_value) = result {
                dma.dma_start(dma_value);

                if crate::hdw::emu::is_debug_enabled() {
                    println!("DMA STARTED");
                }
            }
        },
        _ => {
            println!("IO WRITE NOT IMPLEMENTED for address: {:04X}", address);
        }
    }
}