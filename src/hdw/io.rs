// io.rs
use std::sync::Mutex;

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
            get_int_flags(cpu.unwrap())
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
                return;
            } else {
                println!("Failed to lock SERIAL_DATA for writing to SB");
            }
        },
        0xFF02 => { // SC - Serial Control
            if let Ok(mut data) = SERIAL_DATA.lock() {
                data[1] = value; // Store the new SC value immediately 
                return;
            } else {
                println!("Failed to lock SERIAL_DATA for writing to SC");
            }
        },
        0xFF04..=0xFF07 => {
            cpu.unwrap().timer.timer_write(address, value);
            return;
        },
        0xFF0F => {
            set_int_flags(cpu.unwrap(), value);
            return;
        }
        _ => {
            println!("IO WRITE NOT IMPLEMENTED for address: {:04X}", address);
        }
    }
}