// Debug to retrieve data from serial from blaarg tests

use std::sync::Mutex;
use crate::hdw::bus::Bus;

// Thread-safe debug message buffer
lazy_static::lazy_static! {
    static ref DBG_MSG: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(1024));
}

pub fn dbg_update(bus: &mut Bus) {
    if bus.read_byte(None, 0xFF02) == 0x01 {
        let c = bus.read_byte(None, 0xFF01); // get flag from serial
    
        if let Ok(mut msg) = DBG_MSG.lock() {
            msg.push(c); // add to debug vector
        } else {
            println!("Failed to lock DBG_MSG for updating");
        }
        
        bus.write_byte(None, 0xFF02, 0); // reset flag
    }
}

pub fn dbg_print() {
    if let Ok(msg) = DBG_MSG.lock() {
        if !msg.is_empty() { // parse vector 
            // Convert bytes to string, handling invalid UTF-8
            match std::str::from_utf8(&msg) {
                Ok(s) => println!("DBG: {}", s),
                Err(_) => {
                    // Fall back to printing individual bytes
                    print!("DBG (non-UTF8): ");
                    for &byte in msg.iter() {
                        print!("{:02X} ", byte);
                    }
                    println!();
                }
            }
        }
    } else {
        println!("Failed to lock DBG_MSG for printing");
    }
}

pub fn dbg_clear() {
    if let Ok(mut msg) = DBG_MSG.lock() {
        msg.clear();
    } else {
        println!("Failed to lock DBG_MSG for clearing");
    }
}