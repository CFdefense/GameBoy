// io.rs
use std::sync::Mutex;
use std::io::{self, Write};

// Thread-safe serial data using a Mutex
lazy_static::lazy_static! {
    static ref SERIAL_DATA: Mutex<[u8; 2]> = Mutex::new([0; 2]);
}

pub fn io_read(address: u16) -> u8 {
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
        _ => {
            println!("IO READ NOT IMPLEMENTED for address: {:04X}", address);
            0
        }
    }
}

pub fn io_write(address: u16, value: u8) {
    match address {
        0xFF01 => {
            if let Ok(mut data) = SERIAL_DATA.lock() {
                data[0] = value;
            } else {
                println!("Failed to lock SERIAL_DATA for writing");
            }
            return
        },
        0xFF02 => {
            if let Ok(mut data) = SERIAL_DATA.lock() {
                data[1] = value;
                
                // If bit 7 is set, transfer is requested
                if value & 0x80 != 0 {
                    // Print the character that was sent
                    print!("{}", data[0] as char);
                    io::stdout().flush().unwrap();
                    
                    // Reset transfer bit
                    data[1] &= !0x80;
                }
            } else {
                println!("Failed to lock SERIAL_DATA for writing");
            }
            return
        },
        _ => {
            println!("IO WRITE NOT IMPLEMENTED for address: {:04X}", address);
        }
    }
}