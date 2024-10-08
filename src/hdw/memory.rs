/*

    Gameboy Memory Bus

*/

pub struct Memory {
    pub memory: [u8; 0xFFFF]
}

impl Memory {

    // Consructor
    pub fn new() -> Self {
        Memory {
            // initialize vars
        
        }
    }
    
    // Function to return a byte at an address
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    
    pub fn write_byte(&mut self, address: u16, value: u8) {
        if address < 0xFFFF {
            self.memory[address as usize] = value; // Write the value at the specified address
        } else {
            println!("Attempted to write to an invalid address: {:#X}", address);
        }
    }
}