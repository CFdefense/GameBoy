// FLAG POSITIONS FOR FLAGS REGISTER
const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

// Our CPU to Call and Control
struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    memory: Memory,
    is_halted: bool
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
    RRA, RLA, RRCA, RRLA, CPL, NOP, HALT,
    JP(JumpTest), LD(LoadType), PUSH(StackTarget),
    POP(StackTarget), CALL(JumpTest), RET(JumpTest),
}

// Target All Except F register
enum ArithmeticTarget {
    A, B, C, D, E, H, L,
}

// 16 Bit Targets For Stack
enum StackTarget {
    AF, BC, DE, HL,
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

// TODO IMPLEMENT
// Enum Describes Load Rule
enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    Word, // Like Byte but 16 bit values
    AFromIndirect, //load the A register with the contents from a value from a memory location whose address is stored in some location
    IndirectFromA, // load a memory location whose address is stored in some location with the contents of the A register
    AFromByteAddress, // Just like AFromIndirect except the memory address is some address in the very last byte of memory.
    ByteAddressFromA, // Just like IndirectFromA except the memory address is some address in the very last byte of memory

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

    // TODO Implement
    // Match Instruction to Prefixed Instruction Set
    fn from_prefixed_byte(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::RLC(PrefixTarget::B)),
            // ^ ex syntax
        }
    }

    // TODO IMPLEMENT
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

        // return while halted
        if(self.is_halted) {
            return
        }

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
            Instruction::NOP => {
                // Stands for no-operation and it effectively does nothing except advance the program counter by 1.
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::HALT => {
                // Instruction For Halting CPU Cycle
                self.is_halted = true;
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
                            LoadByteSource::A => self.registers.a,
                            LoadByteSource::B => self.registers.b,
                            LoadByteSource::C => self.registers.c,
                            LoadByteSource::D => self.registers.d,
                            LoadByteSource::E => self.registers.e,
                            LoadByteSource::H => self.registers.h,
                            LoadByteSource::L => self.registers.l,
                            LoadByteSource::D8 => self.read_next_byte(), // direct 8 bytes -> read next bytes
                            LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()), // read byte of address stored in hl
                            _ =>   panic!("LD: Bad Source"),
                        };
                        match target {
                            LoadByteTarget::A => self.registers.a = source_value,
                            LoadByteTarget::B => self.registers.b = source_value,
                            LoadByteTarget::C => self.registers.c = source_value,
                            LoadByteTarget::D => self.registers.d = source_value,
                            LoadByteTarget::E => self.registers.e = source_value,
                            LoadByteTarget::H => self.registers.h = source_value,
                            LoadByteTarget::L => self.registers.l = source_value,
                            LoadByteTarget::HLI => self.bus.write_byte(self.registers.get_hl(), source_value),
                        _ =>   panic!("LD: Bad Target"),
                        };
                        
                        // Increment PC depending on source
                        match source {
                            LoadByteSource::D8  => self.pc.wrapping_add(2),
                            _                   => self.pc.wrapping_add(1),
                        }
                    }
                    LoadType::Word => {

                    }
                    LoadType::AFromIndirect => {

                    }
                    LoadType::IndirectFromA => {

                    }
                    LoadType::AFromByteAddress => {

                    }
                    LoadType::ByteAddressFromA => {

                    }
                    _ =>   panic!("LD: BAD LOAD TYPE"),
                }
            }
            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::AF => self.registers.get_af,
                    StackTarget::BC => self.registers.get.bc,
                    StackTarget::DE => self.registers.get.de,
                    StackTarget::HL => self.registers.get.hl,
                    _ => panic!("PUSH: Bad Target"),
                };
                // push value to stack
                self.push(value);
                
                // increment pc
                self.pc.wrapping_add(1);
            }
            Instruction::POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::AF => self.registers.set_af(result),
                    StackTarget::BC => self.registers.set.bc(result),
                    StackTarget::DE => self.registers.set.de(result),
                    StackTarget::HL => self.registers.set.hl(result),
                    _ => panic!("POP: Bad Target"),
                }
            }
            Instruction::CALL(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero =>    !self.registers.f.zero,
                    JumpTest::NotCarry =>   !self.registers.f.carry,
                    JumpTest::Zero =>       !self.registers.f.zero,
                    JumpTest::Carry =>      !self.registers.f.carry,
                    JumpTest::Always =>     true
                };
                self.call(jump_condition);
            }
            Instruction::RET(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero =>    !self.registers.f.zero,
                    JumpTest::NotCarry =>   !self.registers.f.carry,
                    JumpTest::Zero =>       !self.registers.f.zero,
                    JumpTest::Carry =>      !self.registers.f.carry,
                    JumpTest::Always =>     true
                };
                self.run_return(jump_condition);
            }
            _ =>   panic!("Implement more Instructions"),
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
            let most_significant = self.memory.read_byte(self.pc + 2) as u16;

            // combine and return 2 byte addr in lil endian
            (most_significant << 8) | least_significant
        } else {
            // return next pc
            self.pc.wrapping_add(3)
        }
    }

    // Push to stack and increment pointers
    fn push(&mut self, value: u16) {
        // increment stack pointer
        self.sp = self.sp.wrapping_add(1);

        // mask shift and write first byte to memory at SP
        self.memory.write_byte(self.sp, ((value && 0xFF00) >> 8) as u8);

        // increment stack pointer
        self.sp = self.sp.wrapping_add(1);

        // mask and write second byte to memory at SP
        self.memory.write_byte(self.sp, (value && 0xFF) as u8);
    }

    // Pop from stack and increment pointers
    fn pop(&mut self) -> u16 {
        // read least significant byte from memory at SP
        let least_significant_byte = self.memory.read_byte(self.sp) as u16;
        
        // increment stack pointer
        self.sp = self.sp.wrapping_add(1);

        // read most significan byte from memory at SP
        let most_significant_byte = self.memory.read_byte(self.sp) as u16;
 
        // increment stack pointer
        self.sp = self.sp.wrapping_add(1);

        // shift+OR to combine bytes and implicitly return
        (most_significant_byte << 8) | least_significant_byte
    }

    // Call function for call stack
    fn call(&mut self, should_jump: bool) -> u16 {
        let next_pc = self.pc.wrapping_add(3);
        if(should_jump) {
            self.push(next_pc);
            self.read_next_byte
        } else {
            next_pc
        }
    }

    // Return function for returning through call stack
    fn run_return(&mut self, jump_condition: bool) -> u16 {
        if(should_jump) {
            self.pop()
        } else {
            self.wrapping_add(1);
        }
    }

    // CPU ENDS HERE
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

