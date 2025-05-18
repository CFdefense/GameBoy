// io.rs
use std::sync::Mutex;
use std::io::{self, Write};

use crate::hdw::cpu_util::{get_int_flags, set_int_flags};

// Thread-safe serial data using a Mutex
lazy_static::lazy_static! {
    static ref SERIAL_DATA: Mutex<[u8; 2]> = Mutex::new([0; 2]);
}

pub fn io_read(cpu: Option<&crate::hdw::cpu::CPU>, address: u16) -> u8 {
    match address {
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
            cpu.unwrap().timer.timer_read(address)
        },
        0xFF0F => {
            return get_int_flags(cpu.unwrap())
        }
        _ => {
            println!("IO READ NOT IMPLEMENTED for address: {:04X}", address);
            0
        }
    }
}

pub fn io_write(cpu: Option<&mut crate::hdw::cpu::CPU>, address: u16, value: u8) {
    match address {
        0xFF01 => { // SB - Serial Byte
            if let Ok(mut data) = SERIAL_DATA.lock() {
                data[0] = value;
            } else {
                println!("Failed to lock SERIAL_DATA for writing to SB");
            }
        },
        0xFF02 => { // SC - Serial Control
            if let Ok(mut data) = SERIAL_DATA.lock() {
                data[1] = value; // Store the new SC value immediately
                
                // Check if transfer is requested (bit 7) AND clock is internal (bit 0)
                // This check should be on the NEW state of SC (data[1])
                if (data[1] & 0x81) == 0x81 { 
                    // Character to be "sent" is in SB (data[0])
                    print!("{}", data[0] as char);
                    if io::stdout().flush().is_err() {
                        // Optionally handle flush error, though unwrap() was used before
                        println!("Error flushing stdout for serial output");
                    }
                    
                    // Transfer complete: Hardware clears bit 7 of SC
                    data[1] &= !0x80;
                    
                    // Request Serial Interrupt (IF register, bit 3)
                    if let Some(cpu_ref) = cpu {
                        cpu_ref.int_flags |= 1 << 3; // Set bit 3
                    }
                }
            } else {
                println!("Failed to lock SERIAL_DATA for writing to SC");
            }
        },
        0xFF04..=0xFF07 => {
            cpu.unwrap().timer.timer_write(address, value)
        },
        0xFF0F => {
            set_int_flags(cpu.unwrap(), value);
        }
        _ => {
            println!("IO WRITE NOT IMPLEMENTED for address: {:04X}", address);
        }
    }
}