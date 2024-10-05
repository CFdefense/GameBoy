
struct CPU {
    registers: Registers,
    pc: u16,
    memory: Memory
}

struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8
}

struct Memory {
    memory: [u8; 0xFFFF]
}

// Enum for all instructions
enum Instruction {
    ADD(ArithmeticTarget),
}

// Target all except F register
enum ArithmeticTarget {
    A, B, C, D, E, H, L,
}

impl CPU {
    
}

impl Registers {
    // Get Virtual 16-Bit Register -> Rust Returns Last Expression
    fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }
    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }
    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    
    // Set Virtual 16-Bit Register mask bytes and shift
    fn set_af(&mut self, value: u16) {
        self.a = ((value && 0xFF00) >> 8) as u8;
        self.f = (value && 0xFF) as u8;
    }
    fn set_bc(&mut self, value: u16) {
        self.b = ((value && 0xFF00) >> 8) as u8;
        self.c = (value && 0xFF) as u8;
    }
    fn set_de(&mut self, value: u16) {
        self.d = ((value && 0xFF00) >> 8) as u8;
        self.e = (value && 0xFF) as u8;
    }
    fn set_hl(&mut self, value: u16) {
        self.h = ((value && 0xFF00) >> 8) as u8;
        self.l = (value && 0xFF) as u8;
    }
}

impl Memory {
    // Function to return a byte at an address
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    // Function to execute an opcode by matching Instruction type and target then calling its method
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(target) => {
                match(target) {
                    ArithmeticTarget::C => {
                        // ADD on C Register
                        
                    }
                }
                // Add more targets
            }
            // Add more Instructions
        }
    }
}

