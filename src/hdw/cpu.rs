pub mod memory;

use memory::Memory;

// FLAG POSITIONS FOR FLAGS REGISTER
const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

// Our CPU to Call and Control
pub struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    memory: Memory,
    is_halted: bool,
    curr_opcode: u8,
    curr_instruction: Option<Instruction>,
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
    l: u8,
}

// Special Flags Register to act as u8 but be called as struct
struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

// Target For All Instructions
enum Instruction {
    NOP,
    LD(LoadType),
    INC(ArithmeticTarget),
    DEC(ArithmeticTarget),
    RLCA,
    ADDHL(ArithmeticTarget),
    ADD(ArithmeticTarget),
    RRCA,
    STOP,
    RLA,
    JR,
    RRA,
    DAA,
    CPL,
    SCF(FlagsTarget),
    CCF(FlagsTarget),
    HALT,
    ADC(ArithmeticTarget),
    SUB(ArithmeticTarget),
    SBC(ArithmeticTarget),
    AND(ArithmeticTarget),
    XOR(ArithmeticTarget),
    OR(ArithmeticTarget),
    CP(ArithmeticTarget),
    RET(JumpTest),
    POP(StackTarget),
    JP(JumpTest),
    CALL(JumpTest),
    PUSH(StackTarget),
    RST,

    // PREFIXED INSTRUCTIONS
    RLC(HLTarget),
    RRC(HLTarget),
    RR(HLTarget),
    RL(HLTarget),
    SRA(HLTarget),
    SLA(HLTarget),
    SRL(HLTarget),
    SWAP(HLTarget),
    BIT(ByteTarget),
    RES(ByteTarget),
    SET(ByteTarget),
}

// Target All Except F register
enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

// Enum For BIT/RES/SET Instruction Types
enum ByteTarget {
    Zero(HLTarget),
    One(HLTarget),
    Two(HLTarget),
    Three(HLTarget),
    Four(HLTarget),
    Five(HLTarget),
    Six(HLTarget),
    Seven(HLTarget),
}

enum HLTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}

// 16 Bit Targets For Stack
enum StackTarget {
    AF,
    BC,
    DE,
    HL,
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
    Always,
}

// Enum For Possible Byte Load Targets
enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}

// Enum For Possible Byte Load Sources
enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,
    HLI,
}

// Enum For Possible Word Load Targets
enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    N16,
}

// Enum For Possible Word Load Sources
enum LoadWordSource {
    SP,
    N16,
    HL,
    SPE8,
}

enum LoadN16 {
    BC,
    DE,
    HLINC,
    HLDEC,
}

// TODO IMPLEMENT
// Enum Describes Load Rule
enum LoadType {
    LoadByteTargetyte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource), // Like Byte but 16 bit values
    AStoreInN16(LoadN16),                 // Store A register in N16 register
    N16StoreInA(LoadN16),                 // Store N16 register into A register
    D8StoreInReg(HLTarget),
    RegInReg(HLTarget, HLTarget),
}

// filter byte to instruction dependant on prefixes
impl Instruction {
    // TODO Implement
    // Match Instruction to Prefixed Instruction Set
    fn from_prefixed_byte(byte: u8) -> Option<Instruction> {
        match byte {
            // RLC
            0x00..=0x07 => Some(Instruction::RLC(Self::arithmetic_target_helper(byte))),
            // RRC
            0x08..=0x0F => Some(Instruction::RRC(Self::arithmetic_target_helper(byte))),
            // RL
            0x10..=0x17 => Some(Instruction::RL(Self::arithmetic_target_helper(byte))),
            // RR
            0x18..=0x1F => Some(Instruction::RR(Self::arithmetic_target_helper(byte))),
            // SLA
            0x20..=0x27 => Some(Instruction::SLA(Self::arithmetic_target_helper(byte))),
            // SRA
            0x28..=0x2F => Some(Instruction::SRA(Self::arithmetic_target_helper(byte))),
            // SWAP
            0x30..=0x37 => Some(Instruction::SWAP(Self::arithmetic_target_helper(byte))),
            // SRL
            0x38..=0x3F => Some(Instruction::SRL(Self::arithmetic_target_helper(byte))),
            // BIT
            0x40..=0x7F => Some(Instruction::BIT(Self::byte_target_helper(byte))),
            //RES
            0x080..=0xBF => Some(Instruction::RES(Self::byte_target_helper(byte))),
            //SET
            0x0C0..=0xFF => Some(Instruction::SET(Self::byte_target_helper(byte))),
        }
    }

    // TODO IMPLEMENT
    // Match Instruction to Non Prefixed Instruction Set
    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            //NOP
            0x00 => Some(Instruction::NOP),
            // LD Word w Word
            0x01 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::BC,
                LoadWordSource::N16,
            ))),
            0x11 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::DE,
                LoadWordSource::N16,
            ))),
            0x21 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                LoadWordSource::N16,
            ))),
            0x31 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                LoadWordSource::N16,
            ))),
            0x08 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::N16,
                LoadWordSource::SP,
            ))),
            0xF8 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                LoadWordSource::SPE8,
            ))),
            0xF9 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                LoadWordSource::HL,
            ))),
            // LD N16 From A
            0x02 => Some(Instruction::LD(LoadType::AStoreInN16(LoadN16::BC))),
            0x12 => Some(Instruction::LD(LoadType::AStoreInN16(LoadN16::DE))),
            0x22 => Some(Instruction::LD(LoadType::AStoreInN16(LoadN16::HLINC))),
            0x32 => Some(Instruction::LD(LoadType::AStoreInN16(LoadN16::HLDEC))),
            // LD Reg From D8
            0x06 => Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::B))),
            0x16 => Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::D))),
            0x26 => Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::H))),
            0x36 => Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::HL))),
            0x0E => Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::C))),
            0x1E => Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::E))),
            0x2E => Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::L))),
            0x3E => Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::A))),
            // LD A From N16
            0x0A => Some(Instruction::LD(LoadType::N16StoreInA(LoadN16::BC))),
            0x1A => Some(Instruction::LD(LoadType::N16StoreInA(LoadN16::DE))),
            0x2A => Some(Instruction::LD(LoadType::N16StoreInA(LoadN16::HLINC))),
            0x3A => Some(Instruction::LD(LoadType::N16StoreInA(LoadN16::HLDEC))),
            // LD Register to Register

            // HALT
            _ => todo!("Implement more byte not prefixed"),
        }
    }

    fn arithmetic_target_helper(byte: u8) -> HLTarget {
        match byte % 8 {
            0 => HLTarget::B,
            1 => HLTarget::C,
            2 => HLTarget::D,
            3 => HLTarget::E,
            4 => HLTarget::H,
            5 => HLTarget::L,
            6 => HLTarget::HL,
            7 => HLTarget::A,
            _ => panic!("Math doesnt math"),
        }
    }

    // Determine Instruction # and Associated Register
    fn byte_target_helper(byte: u8) -> ByteTarget {
        let some_instruction = Self::arithmetic_target_helper(byte);
        match byte {
            // Zero
            0x40..=0x47 => ByteTarget::Zero(some_instruction),
            0x80..=0x87 => ByteTarget::Zero(some_instruction),
            0xC0..=0xC7 => ByteTarget::Zero(some_instruction),
            // One
            0x48..=0x4F => ByteTarget::One(some_instruction),
            0x88..=0x8F => ByteTarget::One(some_instruction),
            0xC8..=0xCF => ByteTarget::One(some_instruction),
            // Two
            0x50..=0x57 => ByteTarget::Two(some_instruction),
            0x90..=0x97 => ByteTarget::Two(some_instruction),
            0xD0..=0xD7 => ByteTarget::Two(some_instruction),
            // Three
            0x58..=0x5F => ByteTarget::Three(some_instruction),
            0x98..=0x9F => ByteTarget::Three(some_instruction),
            0xD8..=0xDF => ByteTarget::Three(some_instruction),
            // Four
            0x60..=0x67 => ByteTarget::Four(some_instruction),
            0xA0..=0xA7 => ByteTarget::Four(some_instruction),
            0xE0..=0xE7 => ByteTarget::Four(some_instruction),
            // Five
            0x68..=0x6F => ByteTarget::Five(some_instruction),
            0xA8..=0xAF => ByteTarget::Five(some_instruction),
            0xE8..=0xEF => ByteTarget::Five(some_instruction),
            // Six
            0x70..=0x77 => ByteTarget::Six(some_instruction),
            0xB0..=0xB7 => ByteTarget::Six(some_instruction),
            0xF0..=0xF7 => ByteTarget::Six(some_instruction),
            // Seven
            0x78..=0x7F => ByteTarget::Seven(some_instruction),
            0xB8..=0xBF => ByteTarget::Seven(some_instruction),
            0xF8..=0xFF => ByteTarget::Seven(some_instruction),
            _ => panic!("Bit doesnt bit"),
        }
    }

    fn load_register_helper(byte: u8) -> Option<Instruction> {
        match byte {
            0x76 => Some(Instruction::HALT),
            0x40..=0x47 => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::B,
                Self::arithmetic_target_helper(byte),
            ))),
            0x48..=0x4F => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::C,
                Self::arithmetic_target_helper(byte),
            ))),
            0x50..=0x57 => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::D,
                Self::arithmetic_target_helper(byte),
            ))),
            0x58..=0x5F => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::E,
                Self::arithmetic_target_helper(byte),
            ))),
            0x60..=0x67 => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::H,
                Self::arithmetic_target_helper(byte),
            ))),
            0x68..=0x6F => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::L,
                Self::arithmetic_target_helper(byte),
            ))),
            0x70..=0x77 => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::HL,
                Self::arithmetic_target_helper(byte),
            ))),
            0x78..=0x7F => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::A,
                Self::arithmetic_target_helper(byte),
            ))),
            _ => panic!("Register doesnt register"),
        }
    }
}

impl CPU {
    // Contructor
    pub fn new(memory: Memory) -> Self {
        CPU {
            registers: Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                f: FlagsRegister {
                    zero: false,
                    subtract: false,
                    half_carry: false,
                    carry: false,
                },
                h: 0,
                l: 0,
            },
            pc: 0,
            sp: 0,
            memory,
            is_halted: false,
            curr_opcode: 0,
            curr_instruction: None,
        }
    }

    // Function to 'step' through instructions
    fn step(&mut self) {
        // Get Next Opcode
        self.fetch();

        // Check if byte is the prefix indicator
        self.decode();

        let mut next_pc = 0;
        // Execute the current instruction if it exists and reset it to none
        if let Some(instruction) = self.curr_instruction.take() {
            next_pc = self.execute(instruction);
        }

        // Increment pc to returned pc
        self.pc = next_pc;
    }

    // Function to fetch next opcode
    fn fetch(&mut self) {
        self.curr_opcode = self.memory.read_byte(self.pc);
    }

    // Function to decode current opcode
    fn decode(&mut self) {
        let prefixed = self.curr_opcode == 0xCB;

        // Determine Instruction Byte
        let instruction_opcode = if prefixed {
            self.memory.read_byte(self.pc + 1)
        } else {
            self.curr_opcode
        };

        // Use enum to translate opcode and store next pc addr
        if (prefixed) {
            self.curr_instruction = Instruction::from_prefixed_byte(instruction_opcode);
        } else {
            self.curr_instruction = Instruction::from_byte_not_prefixed(instruction_opcode);
        }

        // Error handling
        if self.curr_instruction.is_none() {
            panic!(
                "Unable to Read Opcode 0x{:02X}, was prefixed? {}",
                instruction_opcode, prefixed
            );
        }

        // Update PC (if needed) based on instruction
        self.pc = self.pc.wrapping_add(if prefixed { 2 } else { 1 });
    }

    // Function to execute an opcode by matching Instruction type and target then calling its method
    fn execute(&mut self, instruction: Instruction) -> u16 {
        // return while halted
        if (self.is_halted) {
            return self.pc;
        }

        match instruction {
            Instruction::ADD(target) => {
                let target_register = match target {
                    ArithmeticTarget::A => self.registers.a,
                    ArithmeticTarget::B => self.registers.b,
                    ArithmeticTarget::C => self.registers.c,
                    ArithmeticTarget::D => self.registers.d,
                    ArithmeticTarget::E => self.registers.e,
                    ArithmeticTarget::H => self.registers.h,
                    ArithmeticTarget::L => self.registers.l,
                };

                // Perform ADD and UPD flags
                let new_value = self.add(target_register);

                // UPD Register
                self.registers.a = new_value;

                // return next pc
                self.pc.wrapping_add(1)
            }
            Instruction::ADDHL(target) => {
                // Get mutable reference to the target register
                let target_register = match target {
                    ArithmeticTarget::A => self.registers.a,
                    ArithmeticTarget::B => self.registers.b,
                    ArithmeticTarget::C => self.registers.c,
                    ArithmeticTarget::D => self.registers.d,
                    ArithmeticTarget::E => self.registers.e,
                    ArithmeticTarget::H => self.registers.h,
                    ArithmeticTarget::L => self.registers.l,
                };

                // Perform ADDHL and UPD flags
                let new_value = self.add_hl(target_register as u16);

                // UPD Register
                self.registers.set_hl(new_value);

                todo!()
            }
            Instruction::ADC(target) => {
                // Get mutable reference to the target register
                let target_register = match target {
                    ArithmeticTarget::A => self.registers.a,
                    ArithmeticTarget::B => self.registers.b,
                    ArithmeticTarget::C => self.registers.c,
                    ArithmeticTarget::D => self.registers.d,
                    ArithmeticTarget::E => self.registers.e,
                    ArithmeticTarget::H => self.registers.h,
                    ArithmeticTarget::L => self.registers.l,
                };

                // Perfom ADC and UPD Flags
                let new_value = self.adc(target_register);

                // UPD Register
                self.registers.a = new_value;

                todo!()
            }
            Instruction::SUB(target) => {
                todo!();
            }
            Instruction::SBC(target) => {
                todo!();
            }
            Instruction::AND(target) => {
                todo!();
            }
            Instruction::OR(target) => {
                todo!();
            }
            Instruction::XOR(target) => {
                todo!();
            }
            Instruction::CP(target) => {
                todo!();
            }
            Instruction::INC(target) => {
                todo!();
            }
            Instruction::DEC(target) => {
                todo!();
            }
            Instruction::CCF(target) => {
                todo!();
            }
            Instruction::SCF(target) => {
                todo!();
            }
            Instruction::RRA => {
                todo!();
            }
            Instruction::RLA => {
                todo!();
            }
            Instruction::RRCA => {
                todo!();
            }
            Instruction::RRLA => {
                todo!();
            }
            Instruction::CPL => {
                todo!();
            }
            Instruction::NOP => {
                // Stands for no-operation and it effectively does nothing except advance the program counter by 1.
                self.pc = self.pc.wrapping_add(1);
                todo!()
            }
            Instruction::HALT => {
                // Instruction For Halting CPU Cycle
                self.is_halted = true;
                todo!()
            }
            Instruction::BIT(targt) => {
                todo!();
            }
            Instruction::RESET(target) => {
                todo!();
            }
            Instruction::SET(target) => {
                todo!();
            }
            Instruction::SRL(target) => {
                todo!();
            }
            Instruction::RR(target) => {
                todo!();
            }
            Instruction::RL(target) => {
                todo!();
            }
            Instruction::RRC(target) => {
                todo!();
            }
            Instruction::RLC(target) => {
                todo!();
            }
            Instruction::SRA(target) => {
                todo!();
            }
            Instruction::SLA(target) => {
                todo!();
            }
            Instruction::SWAP(target) => {
                todo!();
            }
            Instruction::JP(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Zero => !self.registers.f.zero,
                    JumpTest::Carry => !self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.jump(jump_condition)
            }
            Instruction::LD(load_type) => {
                match load_type {
                    LoadType::Byte(target, source) => {
                        let source_value = match source {
                            LoadByteSource::A => self.registers.a,
                            LoadByteSource::B => self.registers.b,
                            LoadByteSource::C => self.registers.c,
                            LoadByteSource::D => self.registers.d,
                            LoadByteSource::E => self.registers.e,
                            LoadByteSource::H => self.registers.h,
                            LoadByteSource::L => self.registers.l,
                            LoadByteSource::D8 => self.memory.read_next_byte(), // direct 8 bytes -> read next bytes
                            LoadByteSource::HLI => self.memory.read_byte(self.registers.get_hl()), // read byte of address stored in hl
                            _ => panic!("LD: Bad Source"),
                        };
                        match target {
                            LoadByteTarget::A => self.registers.a = source_value,
                            LoadByteTarget::B => self.registers.b = source_value,
                            LoadByteTarget::C => self.registers.c = source_value,
                            LoadByteTarget::D => self.registers.d = source_value,
                            LoadByteTarget::E => self.registers.e = source_value,
                            LoadByteTarget::H => self.registers.h = source_value,
                            LoadByteTarget::L => self.registers.l = source_value,
                            LoadByteTarget::HLI => self
                                .memory
                                .write_byte(self.registers.get_hl(), source_value),
                            _ => panic!("LD: Bad Target"),
                        };

                        // Increment PC depending on source
                        match source {
                            LoadByteSource::D8 => self.pc.wrapping_add(2),
                            _ => self.pc.wrapping_add(1),
                        }
                    }
                    LoadType::Word => {
                        todo!();
                    }
                    LoadType::AFromIndirect => {
                        todo!();
                    }
                    LoadType::IndirectFromA => {
                        todo!();
                    }
                    LoadType::AFromByteAddress => {
                        todo!();
                    }
                    LoadType::ByteAddressFromA => {
                        todo!();
                    }
                    _ => panic!("LD: BAD LOAD TYPE"),
                }
            }
            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::AF => self.registers.get_af(),
                    StackTarget::BC => self.registers.get_bc(),
                    StackTarget::DE => self.registers.get_de(),
                    StackTarget::HL => self.registers.get_hl(),
                    _ => panic!("PUSH: Bad Target"),
                };
                // push value to stack
                self.push(value);

                // increment pc
                self.pc.wrapping_add(1)
            }
            Instruction::POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::AF => self.registers.set_af(result),
                    StackTarget::BC => self.registers.set_bc(result),
                    StackTarget::DE => self.registers.set_de(result),
                    StackTarget::HL => self.registers.set_hl(result),
                    _ => panic!("POP: Bad Target"),
                }
                todo!()
            }
            Instruction::CALL(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Zero => !self.registers.f.zero,
                    JumpTest::Carry => !self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.call(jump_condition);
                todo!()
            }
            Instruction::RET(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Zero => !self.registers.f.zero,
                    JumpTest::Carry => !self.registers.f.carry,
                    JumpTest::Always => true,
                };
                self.run_return(jump_condition);
                todo!()
            }
            _ => panic!("Implement more Instructions"),
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
        self.registers.f.half_carry = ((new_value & 0x0F) + (value & 0x0F)) > 0x0F;

        // Implicitly Return
        new_value
    }

    // Jump to addr in memory or increment pc
    fn jump(&self, jump: bool) -> u16 {
        if (jump) {
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
        self.memory
            .write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        // increment stack pointer
        self.sp = self.sp.wrapping_add(1);

        // mask and write second byte to memory at SP
        self.memory.write_byte(self.sp, (value & 0xFF) as u8);
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
        if (should_jump) {
            self.push(next_pc);
            self.memory.read_next_byte();
            todo!()
        } else {
            next_pc
        }
    }

    // Return function for returning through call stack
    fn run_return(&mut self, jump_condition: bool) -> u16 {
        if (jump_condition) {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }

    // CPU ENDS HERE
}

impl Registers {
    // Get Virtual 16-Bit Register -> Rust Returns Last Expression
    fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | u8::from(&self.f) as u16
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
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsRegister::from((value & 0x00FF) as u8);
    }
    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }
    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }
    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }
}

// Method to Convert Flag Register Struct to u8
impl std::convert::From<&FlagsRegister> for u8 {
    fn from(flag: &FlagsRegister) -> u8 {
        // Set Flag Bits In u8 Depending on Status in FlagsRegister
        (if flag.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flag.subtract { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION
            | (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flag.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
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
            carry,
        }
    }
}
