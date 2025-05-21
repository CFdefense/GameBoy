use core::panic;

/*

    File to contain all Enumerations for Instructions and their expected targets and target sources
    As well as all implementations of Instruction operations such as decoding and matching bytes to instructions

*/
use super::{cpu::CPU, emu::{self, emu_cycles}};

// Target For All Instructions
#[derive(Debug)]
pub enum Instruction {
    NOP,
    LD(LoadType),
    INC(AllRegisters),
    DEC(AllRegisters),
    RLCA,
    ADD(OPType),
    RRCA,
    STOP,
    RLA,
    JR(JumpTest),
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,
    HALT,
    ADC(OPTarget),
    SUB(OPTarget),
    SBC(OPTarget),
    AND(OPTarget),
    XOR(OPTarget),
    OR(OPTarget),
    CP(OPTarget),
    RET(JumpTest),
    RETI,
    POP(StackTarget),
    JP(JumpTest),
    CALL(JumpTest),
    PUSH(StackTarget),
    RST(RestTarget),
    EI,
    DI,

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

// Target All 8 bit and 16 bit register except f
#[derive(Debug)]
pub enum AllRegisters {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLMEM,
    BC,
    DE,
    HL,
    SP,
}

// Enum For BIT/RES/SET Instruction Types
#[derive(Debug)]
pub enum ByteTarget {
    Zero(HLTarget),
    One(HLTarget),
    Two(HLTarget),
    Three(HLTarget),
    Four(HLTarget),
    Five(HLTarget),
    Six(HLTarget),
    Seven(HLTarget),
}

#[derive(PartialEq, Debug)]
pub enum HLTarget {
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
#[derive(Debug)]
pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

// Jump Test
#[derive(Debug)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
    HL,
}

// Enum For Possible Word Load Targets
#[derive(Debug)]
pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    N16,
}

// Enum For Possible Word Load Sources
#[derive(Debug)]
pub enum LoadWordSource {
    SP,
    N16,
    HL,
    SPE8,
}

// 16 bit addreses to be loaded
#[derive(Debug)]
pub enum LoadN16 {
    BC,
    DE,
    HLINC,
    HLDEC,
}

// 16 bit registers to be loaded
#[derive(Debug)]
pub enum AddN16Target {
    BC,
    DE,
    HL,
    SP,
}

// Some instructions require differing operations types with differing expected values ADD,ADC,SUB etc
#[derive(Debug)]
pub enum OPType {
    LoadA(HLTarget),
    LoadHL(AddN16Target),
    LoadSP,
    LoadD8,
}

// RST Targets
#[derive(Debug)]
pub enum RestTarget {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

// LD Targets For Edge Cases
#[derive(Debug)]
pub enum LoadA8Target {
    A8,
    A,
}

// LD Targets For Edge Cases
#[derive(Debug)]
pub enum LoadA16Target {
    A16,
    A,
}

// LD Targets For Edge Cases
#[derive(Debug)]
pub enum LoadACTarget {
    C,
    A,
}

#[derive(Debug)]
pub enum OPTarget {
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
    A,
    D8,
}

// Enum Describes Load Rule
#[derive(Debug)]
pub enum LoadType {
    RegInReg(HLTarget, HLTarget),         // Store one register into another
    Word(LoadWordTarget, LoadWordSource), // Like Byte but 16 bit values
    AStoreInN16(LoadN16),                 // Store A register in N16 register
    N16StoreInA(LoadN16),                 // Store N16 register into A register
    D8StoreInReg(HLTarget),               // Store D8 into a register
    AWithA8(LoadA8Target),                // Store A in a8 and reverse
    AWithA16(LoadA16Target),              // Store A in a16 and reverse
    AWithAC(LoadACTarget),                // Store A with C and reverse
}

impl Instruction {
    // Function to take opcode from cpu and match it to a corresponding Instruction
    pub fn decode_from_opcode(opcode: u8, pc: u16, cpu: &mut CPU) -> Option<Instruction> {
        let prefixed = opcode == 0xCB;

        if prefixed {
            emu_cycles(cpu, 1);
        }

        // determine if instruction is a PREFIX
        let instruction_opcode = if prefixed {
            cpu.bus.read_byte(None, pc + 1)
        } else {
            opcode
        };

        // Use enum to translate opcode and store next pc addr
        let instruction = if prefixed {
            Instruction::from_prefixed_byte(instruction_opcode, cpu)
        } else {
            Instruction::from_byte_not_prefixed(instruction_opcode, cpu)
        };

        // Implicit Return
        instruction
    }

    // Match Instruction to Prefixed Instruction Set
    fn from_prefixed_byte(byte: u8, cpu: &mut CPU) -> Option<Instruction> {
        match byte {
            // RLC
            0x00..=0x07 => {
                if byte == 0x06 {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::RLC(Self::hl_target_helper(byte)))},
            // RRC
            0x08..=0x0F => {
                if byte == 0x0E {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::RRC(Self::hl_target_helper(byte)))},
            // RL
            0x10..=0x17 => {
                if byte == 0x16 {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::RL(Self::hl_target_helper(byte)))},
            // RR
            0x18..=0x1F => {
                if byte == 0x1E {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::RR(Self::hl_target_helper(byte)))},
            // SLA
            0x20..=0x27 => {
                if byte == 0x26 {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::SLA(Self::hl_target_helper(byte)))},
            // SRA
            0x28..=0x2F => {
                if byte == 0x2E {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::SRA(Self::hl_target_helper(byte)))},
            // SWAP
            0x30..=0x37 => {
                if byte == 0x36 {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::SWAP(Self::hl_target_helper(byte)))},
            // SRL
            0x38..=0x3F => {
                if byte == 0x3E {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::SRL(Self::hl_target_helper(byte)))},
            // BIT
            0x40..=0x7F => {
                if byte == 0x46 || byte == 0x4E || byte == 0x56 || byte == 0x5E || byte == 0x66 || byte == 0x6E || byte == 0x7E {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::BIT(Self::byte_target_helper(byte)))},
            //RES
            0x080..=0xBF => {
                if byte == 0x86 || byte == 0x8E || byte == 0x96 || byte == 0x9E || byte == 0xA6 || byte == 0xAE || byte == 0xB6 || byte == 0xBE {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::RES(Self::byte_target_helper(byte)))},
            //SET
            0x0C0..=0xFF => {
                if byte == 0xC6 || byte == 0xCE || byte == 0xD6 || byte == 0xDE || byte == 0xE6 || byte == 0xEE || byte == 0xF6 || byte == 0xFE {
                    emu_cycles(cpu, 2);
                }
                Some(Instruction::SET(Self::byte_target_helper(byte)))},
        }
    }

    // Match Instruction to Non Prefixed Instruction Set
    fn from_byte_not_prefixed(byte: u8, cpu: &mut CPU) -> Option<Instruction> {
        match byte {
            //NOP
            0x00 => Some(Instruction::NOP),
            //SOP
            0x10 => Some(Instruction::STOP),
            //RLCA
            0x07 => Some(Instruction::RLCA),
            //RRCA
            0x0F => Some(Instruction::RRCA),
            //RLA
            0x17 => Some(Instruction::RLA),
            //RRA
            0x1F => Some(Instruction::RRA),
            //DAA
            0x27 => Some(Instruction::DAA),
            //SCF
            0x37 => Some(Instruction::SCF),
            //CPL
            0x2F => Some(Instruction::CPL),
            //CCF
            0x3F => Some(Instruction::CCF),
            //JR
            0x18 => {
                emu_cycles(cpu, 1);
                Some(Instruction::JR(JumpTest::Always))},
            0x20 => {
                emu_cycles(cpu, 1);
                Some(Instruction::JR(JumpTest::NotZero))},
            0x28 => {
                emu_cycles(cpu, 1);
                Some(Instruction::JR(JumpTest::Zero))},
            0x30 => {
                emu_cycles(cpu, 1);
                Some(Instruction::JR(JumpTest::NotCarry))},
            0x38 => {
                emu_cycles(cpu, 1);
                Some(Instruction::JR(JumpTest::Carry))},
            // INC
            0x03 => {
                emu_cycles(cpu, 1);
                Some(Instruction::INC(AllRegisters::BC))},
            0x13 => {
                emu_cycles(cpu, 1);
                Some(Instruction::INC(AllRegisters::DE))},
            0x23 => {
                emu_cycles(cpu, 1);
                Some(Instruction::INC(AllRegisters::HL))},
            0x33 => {
                emu_cycles(cpu, 1);
                Some(Instruction::INC(AllRegisters::SP))},
            0x04 => Some(Instruction::INC(AllRegisters::B)),
            0x14 => Some(Instruction::INC(AllRegisters::D)),
            0x24 => Some(Instruction::INC(AllRegisters::H)),
            0x34 => Some(Instruction::INC(AllRegisters::HLMEM)),
            0x0C => Some(Instruction::INC(AllRegisters::C)),
            0x1C => Some(Instruction::INC(AllRegisters::E)),
            0x2C => Some(Instruction::INC(AllRegisters::L)),
            0x3C => Some(Instruction::INC(AllRegisters::A)),
            // DEC
            0x0B => {
                emu_cycles(cpu, 1);
                Some(Instruction::DEC(AllRegisters::BC))},
            0x1B => {
                emu_cycles(cpu, 1);
                Some(Instruction::DEC(AllRegisters::DE))},
            0x2B => {
                emu_cycles(cpu, 1);
                Some(Instruction::DEC(AllRegisters::HL))},
            0x3B => {
                emu_cycles(cpu, 1);
                Some(Instruction::DEC(AllRegisters::SP))},
            0x05 => Some(Instruction::DEC(AllRegisters::B)),
            0x15 => Some(Instruction::DEC(AllRegisters::D)),
            0x25 => Some(Instruction::DEC(AllRegisters::H)),
            0x35 => Some(Instruction::DEC(AllRegisters::HLMEM)),
            0x0D => Some(Instruction::DEC(AllRegisters::C)),
            0x1D => Some(Instruction::DEC(AllRegisters::E)),
            0x2D => Some(Instruction::DEC(AllRegisters::L)),
            0x3D => Some(Instruction::DEC(AllRegisters::A)),
            // LD Word w Word
            0x01 => {
                emu_cycles(cpu, 2);
                Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::BC,
                LoadWordSource::N16,
            )))},
            0x11 => {
                emu_cycles(cpu, 2);
                Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::DE,
                LoadWordSource::N16,
            )))},
            0x21 => {
                emu_cycles(cpu, 2);
                Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                LoadWordSource::N16,
            )))},
            0x31 => {
                emu_cycles(cpu, 2);
                Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                LoadWordSource::N16,
            )))},
            0x08 => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::N16,
                LoadWordSource::SP,
            )))},
            0xF8 => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                LoadWordSource::SPE8,
            )))},
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
            0x06 => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::B)))},
            0x16 => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::D)))},
            0x26 => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::H)))},
            0x36 => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::HL)))},
            0x0E => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::C)))},
            0x1E => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::E)))},
            0x2E => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::L)))},
            0x3E => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::D8StoreInReg(HLTarget::A)))},
            // LD A From N16
            0x0A => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::N16StoreInA(LoadN16::BC)))},
            0x1A => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::N16StoreInA(LoadN16::DE)))},
            0x2A => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::N16StoreInA(LoadN16::HLINC)))},
            0x3A => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::N16StoreInA(LoadN16::HLDEC)))},
            // LD Register to Register + HALT
            0x40..=0x7F => {
                if byte == 0x46 || byte == 0x4E || byte == 0x56 || byte == 0x5E || byte == 0x66 || byte == 0x6E || byte == 0x7E {
                    emu_cycles(cpu, 1);
                }
                Self::load_register_helper(byte)},
            // LD A and a8
            0xE0 => {
                emu_cycles(cpu,1);
                Some(Instruction::LD(LoadType::AWithA8(LoadA8Target::A8)))},
            0xF0 => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::AWithA8(LoadA8Target::A)))},
            // LD A and C
            0xE2 => Some(Instruction::LD(LoadType::AWithAC(LoadACTarget::C))),
            0xF2 => {
                emu_cycles(cpu, 1);
                Some(Instruction::LD(LoadType::AWithAC(LoadACTarget::A)))},
            // LD A and a16
            0xEA => {
                emu_cycles(cpu, 2);
                Some(Instruction::LD(LoadType::AWithA16(LoadA16Target::A16)))},
            0xFA => {
                emu_cycles(cpu, 3);
                Some(Instruction::LD(LoadType::AWithA16(LoadA16Target::A)))},
            // ADD Register to A
            0x80..=0x87 => {
                if byte == 0x86 {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::ADD(OPType::LoadA(Self::hl_target_helper(
                byte,
            ))))},
            0xC6 => {
                emu_cycles(cpu, 2);
                Some(Instruction::ADD(OPType::LoadD8))}, // ADD D8
            0xE8 => {
                emu_cycles(cpu, 1);
                Some(Instruction::ADD(OPType::LoadSP))}, // ADD s8 SP
            // ADD N16 Register to N16 Register
            0x09 => Some(Instruction::ADD(OPType::LoadHL(AddN16Target::BC))),
            0x19 => Some(Instruction::ADD(OPType::LoadHL(AddN16Target::DE))),
            0x29 => Some(Instruction::ADD(OPType::LoadHL(AddN16Target::HL))),
            0x39 => Some(Instruction::ADD(OPType::LoadHL(AddN16Target::SP))),
            // ADC
            0x88..=0x8F => {
                if byte == 0x8E {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::ADC(Self::op_target_helper(byte)))},
            0xCE => {
                emu_cycles(cpu, 1);
                Some(Instruction::ADC(OPTarget::D8))},
            // SUB
            0x90..=0x97 => {
                if byte == 0x96 {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::SUB(Self::op_target_helper(byte)))},
            0xD6 => {
                emu_cycles(cpu, 1);                
                Some(Instruction::SUB(OPTarget::D8))},
            // SBC
            0x98..=0x9F => {
                if byte == 0x9E {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::SBC(Self::op_target_helper(byte)))},
            0xDE => {
                emu_cycles(cpu, 1);
                Some(Instruction::SBC(OPTarget::D8))},
            // AND
            0xA0..=0xA7 => {
                if byte == 0xA6 {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::AND(Self::op_target_helper(byte)))},
            0xE6 => {
                emu_cycles(cpu, 1);
                Some(Instruction::AND(OPTarget::D8))},
            // XOR
            0xA8..=0xAF => {
                if byte == 0xAE {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::XOR(Self::op_target_helper(byte)))},
            0xEE => {
                emu_cycles(cpu, 1);
                Some(Instruction::XOR(OPTarget::D8))},
            // OR
            0xB0..=0xB7 => {
                if byte == 0xB6 {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::OR(Self::op_target_helper(byte)))},
            0xF6 => {
                emu_cycles(cpu, 1);
                Some(Instruction::OR(OPTarget::D8))},
            // CP
            0xB8..=0xBF => {
                if byte == 0xBE {
                    emu_cycles(cpu, 1);
                }
                Some(Instruction::CP(Self::op_target_helper(byte)))},
            0xFE => {
                emu_cycles(cpu, 1);
                Some(Instruction::CP(OPTarget::D8))},
            // RET
            0xC0 => Some(Instruction::RET(JumpTest::NotZero)),
            0xC8 => Some(Instruction::RET(JumpTest::Zero)),
            0xD0 => Some(Instruction::RET(JumpTest::NotCarry)),
            0xD8 => Some(Instruction::RET(JumpTest::Carry)),
            0xC9 => Some(Instruction::RET(JumpTest::Always)),
            // RETI
            0xD9 => Some(Instruction::RETI),
            // POP
            0xC1 => Some(Instruction::POP(StackTarget::BC)),
            0xD1 => Some(Instruction::POP(StackTarget::DE)),
            0xE1 => Some(Instruction::POP(StackTarget::HL)),
            0xF1 => Some(Instruction::POP(StackTarget::AF)),
            // JP
            0xC2 => {
                emu_cycles(cpu, 2);
                Some(Instruction::JP(JumpTest::NotZero))},
            0xCA => {
                emu_cycles(cpu, 2);
                Some(Instruction::JP(JumpTest::Zero))},
            0xD2 => {
                emu_cycles(cpu, 2);
                Some(Instruction::JP(JumpTest::NotCarry))},
            0xDA => {
                emu_cycles(cpu, 2);
                Some(Instruction::JP(JumpTest::Carry))},
            0xC3 => {
                emu_cycles(cpu, 2);
                Some(Instruction::JP(JumpTest::Always))},
            0xE9 => Some(Instruction::JP(JumpTest::HL)),
            // CALL
            0xC4 => {
                emu_cycles(cpu, 2);
                Some(Instruction::CALL(JumpTest::NotZero))},
            0xCC => {
                emu_cycles(cpu, 2);
                Some(Instruction::CALL(JumpTest::Zero))},
            0xD4 => {
                emu_cycles(cpu, 2);
                Some(Instruction::CALL(JumpTest::NotCarry))},
            0xDC => {
                emu_cycles(cpu, 2);
                Some(Instruction::CALL(JumpTest::Carry))},
            0xCD => {
                emu_cycles(cpu, 2);
                Some(Instruction::CALL(JumpTest::Always))},
            // PUSH
            0xC5 => Some(Instruction::PUSH(StackTarget::BC)),
            0xD5 => Some(Instruction::PUSH(StackTarget::DE)),
            0xE5 => Some(Instruction::PUSH(StackTarget::HL)),
            0xF5 => Some(Instruction::PUSH(StackTarget::AF)),
            // RST
            0xC7 => Some(Instruction::RST(RestTarget::Zero)),
            0xCF => Some(Instruction::RST(RestTarget::One)),
            0xD7 => Some(Instruction::RST(RestTarget::Two)),
            0xDF => Some(Instruction::RST(RestTarget::Three)),
            0xE7 => Some(Instruction::RST(RestTarget::Four)),
            0xEF => Some(Instruction::RST(RestTarget::Five)),
            0xF7 => Some(Instruction::RST(RestTarget::Six)),
            0xFF => Some(Instruction::RST(RestTarget::Seven)),
            // DI
            0xF3 => Some(Instruction::DI),
            // EI
            0xFB => Some(Instruction::EI),
            0xD3 | 0xE3 | 0xE4 | 0xF4 | 0xCB | 0xDB | 0xEB | 0xEC | 0xFC | 0xDD | 0xED | 0xFD => {
                panic!("NULL INSTRUCTION READ: {:02X}", byte)
            }
            _ => panic!("NOT AN INSTRUCTION: {:02X}", byte),
        }
    }

    // Function to help quickly match bytes to their associated HL Target
    fn hl_target_helper(byte: u8) -> HLTarget {
        match byte % 8 {
            0 => Some(HLTarget::B),
            1 => Some(HLTarget::C),
            2 => Some(HLTarget::D),
            3 => Some(HLTarget::E),
            4 => Some(HLTarget::H),
            5 => Some(HLTarget::L),
            6 => Some(HLTarget::HL),
            7 => Some(HLTarget::A),
            _ => None,
        }
        .expect("Math doesn't math") // Unwrap and panic if None
    }

    // Function for OP Targets
    fn op_target_helper(byte: u8) -> OPTarget {
        match byte % 8 {
            0 => Some(OPTarget::B),
            1 => Some(OPTarget::C),
            2 => Some(OPTarget::D),
            3 => Some(OPTarget::E),
            4 => Some(OPTarget::H),
            5 => Some(OPTarget::L),
            6 => Some(OPTarget::HL),
            7 => Some(OPTarget::A),
            _ => Some(OPTarget::D8),
        }
        .expect("Math doesn't math") // Unwrap and panic if None
    }

    // Determine Instruction # and Associated Register
    fn byte_target_helper(byte: u8) -> ByteTarget {
        let some_instruction = Self::hl_target_helper(byte);
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

    // Function to help match large set of LD instructions by first matching their target then their associated source
    fn load_register_helper(byte: u8) -> Option<Instruction> {
        match byte {
            0x76 => Some(Instruction::HALT),
            0x40..=0x47 => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::B,
                Self::hl_target_helper(byte),
            ))),
            0x48..=0x4F => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::C,
                Self::hl_target_helper(byte),
            ))),
            0x50..=0x57 => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::D,
                Self::hl_target_helper(byte),
            ))),
            0x58..=0x5F => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::E,
                Self::hl_target_helper(byte),
            ))),
            0x60..=0x67 => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::H,
                Self::hl_target_helper(byte),
            ))),
            0x68..=0x6F => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::L,
                Self::hl_target_helper(byte),
            ))),
            0x70..=0x77 => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::HL,
                Self::hl_target_helper(byte),
            ))),
            0x78..=0x7F => Some(Instruction::LD(LoadType::RegInReg(
                HLTarget::A,
                Self::hl_target_helper(byte),
            ))),
            _ => panic!("Register doesnt register"),
        }
    }
}
