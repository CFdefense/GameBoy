// FLAG POSITIONS FOR FLAGS REGISTER
const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

// Our CPU to Call and Control
struct CPU {
    registers: Registers,
    pc: u16,
    memory: Memory
}

// Registers For Holding and Manipulating Data
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister,
    h: u8,
    l: u8
}

// Special Flags Register to act as u8 but be called as struct
struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool
}

// Our Gameboy's Memory
struct Memory {
    memory: [u8; 0xFFFF]
}

// Target For All Instructions
enum Instruction {
    ADD(ArithmeticTarget), ADDHL(ArithmeticTarget),
    ADC(ArithmeticTarget), SUB(ArithmeticTarget),
    SBC(ArithmeticTarget), AND(ArithmeticTarget),
    OR(ArithmeticTarget), XOR(ArithmeticTarget),
    CP(ArithmeticTarget), INC(ArithmeticTarget),
    DEC(ArithmeticTarget), CCF(FlagsTarget),
    SCF(FlagsTarget), BIT(ArithmeticTarget),
    RESET(ArithmeticTarget), SET(ArithmeticTarget),
    SRL(ArithmeticTarget), RR(ArithmeticTarget),
    RL(ArithmeticTarget), RRC(ArithmeticTarget),
    RLC(ArithmeticTarget), SRA(ArithmeticTarget),
    SLA(ArithmeticTarget), SWAP(ArithmeticTarget),
    RRA, RLA, RRCA, RRLA, CPL, 
    JP(JumpTest), LD(LoadType),
}

// Target All Except F register
enum ArithmeticTarget {
    A, B, C, D, E, H, L,
}

// Target F Register
enum FlagsTarget {
    Zero,
    Subtract,
    HalfCarry,
    Carry,
}

// Jump Test
enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always
}

// Enum For Possible Load Targets
enum LoadByteTarget {
    A, B, C, D, E, H, L, HLI
}

// Enum For Possible Load Sources
enum LoadByteSource {
    A, B, C, D, E, H, L, D8, HLI
}

// Enum Describes Load Rule
enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
}

// filter byte to instruction dependant on prefixes
impl Instruction {
    fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
       if(prefixed) {
        // returns result of this 
        Instruction::from_prefixed_byte(byte)
       } else {
        // returns result of this
        Instruction::from_byte_not_prefixed(byte)
       }
    }

    // Match Instruction to Prefixed Instruction Set
    fn from_prefixed_byte(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::RLC(PrefixTarget::B)),
            // ^ ex syntax
        }
    }

    // Match Instruction to Non Prefixed Instruction Set
    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte{
            0x02 => Some(Instruction::INC(IncDecTarget::BC)),
            // ^ ex syntax
        }
    }
}



impl CPU {
    // Function to 'step' through instructions
    fn step(&mut self) {
        // read next instruction from memory
        let mut instruction_opcode = self.memory.read_byte(self.pc);

        // Check if byte is the prefix indicator
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            // if prefixedd instead we read next byte 
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }

        // Use enum to translate opcode and store next pc addr
        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_opcode, prefixed) {
            self.execute(instruction);
        } else {
            panic!("Unable to Read Opcode 0x{}", instruction_opcode, " was prefixed? {}", prefixed);
        };

        self.pc = next_pc;
    }


    // Function to execute an opcode by matching Instruction type and target then calling its method
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target) => {
                let target_register = match target {
                    ArithmeticTarget::A => &mut self.registers.a,
                    ArithmeticTarget::B => &mut self.registers.b,
                    ArithmeticTarget::C => &mut self.registers.c,
                    ArithmeticTarget::D => &mut self.registers.d,
                    ArithmeticTarget::E => &mut self.registers.e,
                    ArithmeticTarget::H => &mut self.registers.h,
                    ArithmeticTarget::L => &mut self.registers.l,
                };
                
                // Perform ADD and UPD flags
                let new_value = self.add(*target_register);

                // UPD Register
                self.registers.a = new_value;

                // return next pc
                self.pc.wrapping_add(1) 
            }
            Instruction::ADDHL(target) => {
                // Get mutable reference to the target register
                let target_register = match target {
                    ArithmeticTarget::A => &mut self.registers.a,
                    ArithmeticTarget::B => &mut self.registers.b,
                    ArithmeticTarget::C => &mut self.registers.c,
                    ArithmeticTarget::D => &mut self.registers.d,
                    ArithmeticTarget::E => &mut self.registers.e,
                    ArithmeticTarget::H => &mut self.registers.h,
                    ArithmeticTarget::L => &mut self.registers.l,
                };

                // Perform ADDHL and UPD flags
                let new_value = self.add_hl(*target_register as u16);

                // UPD Register
                self.set_hl = new_value;
            }
            Instruction::ADC(target) => {
                // Get mutable reference to the target register
                let target_register = match target {
                    ArithmeticTarget::A => &mut self.registers.a,
                    ArithmeticTarget::B => &mut self.registers.b,
                    ArithmeticTarget::C => &mut self.registers.c,
                    ArithmeticTarget::D => &mut self.registers.d,
                    ArithmeticTarget::E => &mut self.registers.e,
                    ArithmeticTarget::H => &mut self.registers.h,
                    ArithmeticTarget::L => &mut self.registers.l,
                };

                // Perfom ADC and UPD Flags
                let new_value = self.adc(*target_register);

                // UPD Register
                self.registers.a = new_value;
            }
            Instruction::SUB(target) => {

            }
            Instruction::SBC(target) => {

            }
            Instruction::AND(target) => {
                
            }
            Instruction::OR(target) => {

            }
            Instruction::XOR(target) => {

            }
            Instruction::CP(target) => {

            }
            Instruction::INC(target) => {

            }
            Instruction::DEC(target) => {

            }
            Instruction::CCF(target) => {

            }
            Instruction::SCF(target) => {

            }
            Instruction::RRA => {

            }
            Instruction::RLA => {

            }
            Instruction::RRCA => {

            }
            Instruction::RRLA => {

            }
            Instruction::CPL => {

            }
            Instruction::BIT(targt) => {

            }
            Instruction::RESET(target) => {

            }
            Instruction::SET(target) => {

            }
            Instruction::SRL(target) => {

            }
            Instruction::RR(target) => {

            }
            Instruction::RL(target) => {

            }
            Instruction::RRC(target) => {

            }
            Instruction::RLC(target) => {

            }
            Instruction::SRA(target) => {

            }
            Instruction::SLA(target) => {

            }
            Instruction::SWAP(target) => {

            }
            Instruction::JP(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero =>    !self.registers.f.zero,
                    JumpTest::NotCarry =>   !self.registers.f.carry,
                    JumpTest::Zero =>       !self.registers.f.zero,
                    JumpTest::Carry =>      !self.registers.f.carry,
                    JumpTest::Always =>     true
                };
                self.jump(jump_condition)
            }
            Instruction::LD(load_type) => {
                match load_type {
                    LoadType::Byte(target, source) => {
                        let source = match source {
                            // TODO match sources
                        };
                        match target {
                            // TODO match target and up
                        };
                        
                        // Increment PC depending on source
                        match source {
                            LoadByteSource::D8  => self.pc.wrapping_add(2),
                            _                   => self.pc.wrapping_add(1),
                        }
                    }
                    // TODO Implement other Load Types
                }
            }
            // TODO MORE INSTRUCTIONS
        }
    }

    // ADD -> Adds specific registers contents to the a registers contents
    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);

        // Upd flags
        self.registers.f.zero = new_value == 0; // zero flag updated if 0
        self.registers.f.subtract = false; // set true if operation was subtraction
        self.registers.f.carry = did_overflow; // set true if overflow occured

        // Half Carry set true if lower nibbles of value and a register added are > than 0xF
        // This would mean there was a carry from the lower nibble to the upper nibble
        self.registers.f.half_carry = ((self.registers.a & 0x0F) + (value & 0x0F)) > 0x0F;

        // Implicitly Returned
        new_value
    }

    // ADDHL -> Adds specific registers contents to hl 16-bit register contents
    fn add_hl(&mut self, value: u16) -> u16 {
        // Get Current hl register value
        let hl_value = self.registers.get_hl();

        // Perform the addition
        let (new_hl_value, did_overflow) = hl_value.overflowing_add(value);

        // Update flags
        self.registers.f.carry = did_overflow; // Set carry flag if overflow occurred
        self.registers.f.zero = false; // Zero flag is not relevant for HL addition
        self.registers.f.subtract = false; // This is not a subtraction operation
        self.registers.f.half_carry = ((hl_value & 0x0F) + (value & 0x0F)) > 0x0F;

        // Implicitly Return
        new_hl_value
    }

    // ADC -> just like ADD except that the value of the carry flag is also added to the number
    fn adc(&mut self, value: u8) -> u8 {
        // Get carry value from the carry flag
        let carry = if self.registers.f.carry { 1 } else { 0 };

        // Perform the addition including carry
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value + carry);

        // Update flags
        self.registers.f.carry = did_overflow; // Set carry flag if overflow occurred
        self.registers.f.zero = false; // Zero flag is not relevant for HL addition
        self.registers.f.subtract = false; // This is not a subtraction operation
        self.registers.f.half_carry = ((hl_value & 0x0F) + (value & 0x0F)) > 0x0F;

        // Implicitly Return
        new_value
    }

    // Jump to addr in memory or increment pc
    fn jump(&self, jump: bool) -> u16 {
        if(jump) {
            let least_significant = self.memory.read_byte(self.pc + 1) as u16;
            let most_significant = self.memory.read_bye(self.pc + 2) as u16;

            // combine and return 2 byte addr in lil endian
            (most_significant << 8) | least_significant
        } else {
            // return next pc
            self.pc.wrapping_add(3)
        }
    }


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
        self.f = FlagsRegister::from((value & 0x00FF) as u8);
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
}

// Method to Convert Flag Register Struct to u8
impl std::convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        // Set Flag Bits In u8 Depending on Status in FlagsRegister
        (if flag.zero {1} else {0}) << ZERO_FLAG_BYTE_POSITION |
        (if flag.subtract {1} else {0}) << SUBTRACT_FLAG_BYTE_POSITION |
        (if flag.half_carry {1} else {0}) << HALF_CARRY_FLAG_BYTE_POSITION |
        (if flag.carry {1} else {0}) << CARRY_FLAG_BYTE_POSITION
    }
}

// Method to Convert u8 to Flag Register Struct
impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        // Get Register Bitwise Values 
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0xb1) != 0;
        let subtract = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 0xb1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0xb1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0xb1) != 0;

        // Remake Register
        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry
        }
    }
}

