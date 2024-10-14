use crate::hdw::memory::Memory;

use core::panic;

use super::cart::Cartridge;

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
    cartridge: Cartridge,
    is_halted: bool,
    curr_opcode: u8,
    curr_instruction: Option<Instruction>,
}

// Registers For Holding and Manipulating Data
#[derive(Debug)]
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
#[derive(Debug)]
struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

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
    ADC(OPType),
    SUB(OPType),
    SBC(OPType),
    AND(OPType),
    XOR(OPType),
    OR(OPType),
    CP(OPType),
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

// Target All Except F register
#[derive(Debug)]
pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
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

#[derive(Debug)]
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

// Target F Register
#[derive(Debug)]
pub enum FlagsTarget {
    Zero,
    Subtract,
    HalfCarry,
    Carry,
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

// Enum For Possible Byte Load Targets
#[derive(Debug)]
pub enum LoadByteTarget {
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
#[derive(Debug)]
pub enum LoadByteSource {
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

#[derive(Debug)]
pub enum LoadN16 {
    BC,
    DE,
    HLINC,
    HLDEC,
}

#[derive(Debug)]
pub enum AddN16Target {
    BC,
    DE,
    HL,
    SP,
}

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

// filter byte to instruction dependant on prefixes
impl Instruction {
    // Match Instruction to Prefixed Instruction Set
    fn from_prefixed_byte(byte: u8) -> Option<Instruction> {
        match byte {
            // RLC
            0x00..=0x07 => Some(Instruction::RLC(Self::hl_target_helper(byte))),
            // RRC
            0x08..=0x0F => Some(Instruction::RRC(Self::hl_target_helper(byte))),
            // RL
            0x10..=0x17 => Some(Instruction::RL(Self::hl_target_helper(byte))),
            // RR
            0x18..=0x1F => Some(Instruction::RR(Self::hl_target_helper(byte))),
            // SLA
            0x20..=0x27 => Some(Instruction::SLA(Self::hl_target_helper(byte))),
            // SRA
            0x28..=0x2F => Some(Instruction::SRA(Self::hl_target_helper(byte))),
            // SWAP
            0x30..=0x37 => Some(Instruction::SWAP(Self::hl_target_helper(byte))),
            // SRL
            0x38..=0x3F => Some(Instruction::SRL(Self::hl_target_helper(byte))),
            // BIT
            0x40..=0x7F => Some(Instruction::BIT(Self::byte_target_helper(byte))),
            //RES
            0x080..=0xBF => Some(Instruction::RES(Self::byte_target_helper(byte))),
            //SET
            0x0C0..=0xFF => Some(Instruction::SET(Self::byte_target_helper(byte))),
        }
    }

    // Match Instruction to Non Prefixed Instruction Set
    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
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
            0x37 => Some(Instruction::DAA),
            //SCF
            0x47 => Some(Instruction::SCF),
            //CPL
            0x2F => Some(Instruction::CPL),
            //CCF
            0x3F => Some(Instruction::CCF),
            //JR
            0x18 => Some(Instruction::JR(JumpTest::Always)),
            0x20 => Some(Instruction::JR(JumpTest::NotZero)),
            0x28 => Some(Instruction::JR(JumpTest::Zero)),
            0x30 => Some(Instruction::JR(JumpTest::NotCarry)),
            0x38 => Some(Instruction::JR(JumpTest::Carry)),
            // INC
            0x03 => Some(Instruction::INC(AllRegisters::BC)),
            0x13 => Some(Instruction::INC(AllRegisters::DE)),
            0x23 => Some(Instruction::INC(AllRegisters::HL)),
            0x43 => Some(Instruction::INC(AllRegisters::SP)),
            0x04 => Some(Instruction::INC(AllRegisters::B)),
            0x14 => Some(Instruction::INC(AllRegisters::D)),
            0x24 => Some(Instruction::INC(AllRegisters::H)),
            0x34 => Some(Instruction::INC(AllRegisters::HLMEM)),
            0x0C => Some(Instruction::INC(AllRegisters::C)),
            0x1C => Some(Instruction::INC(AllRegisters::E)),
            0x2C => Some(Instruction::INC(AllRegisters::L)),
            0x3C => Some(Instruction::INC(AllRegisters::A)),
            // DEC
            0x0B => Some(Instruction::DEC(AllRegisters::BC)),
            0x1B => Some(Instruction::DEC(AllRegisters::DE)),
            0x2B => Some(Instruction::DEC(AllRegisters::HL)),
            0x4B => Some(Instruction::DEC(AllRegisters::SP)),
            0x05 => Some(Instruction::DEC(AllRegisters::B)),
            0x15 => Some(Instruction::DEC(AllRegisters::D)),
            0x25 => Some(Instruction::DEC(AllRegisters::H)),
            0x35 => Some(Instruction::DEC(AllRegisters::HLMEM)),
            0x0D => Some(Instruction::DEC(AllRegisters::C)),
            0x1D => Some(Instruction::DEC(AllRegisters::E)),
            0x2D => Some(Instruction::DEC(AllRegisters::L)),
            0x3D => Some(Instruction::DEC(AllRegisters::A)),
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
            // LD Register to Register + HALT
            0x40..=0x7F => Self::load_register_helper(byte),
            // LD A and a8
            0xE0 => Some(Instruction::LD(LoadType::AWithA8(LoadA8Target::A8))),
            0xF0 => Some(Instruction::LD(LoadType::AWithA8(LoadA8Target::A))),
            // LD A and C
            0xE3 => Some(Instruction::LD(LoadType::AWithAC(LoadACTarget::C))),
            0xF3 => Some(Instruction::LD(LoadType::AWithAC(LoadACTarget::A))),
            // LD A and a16
            0xEA => Some(Instruction::LD(LoadType::AWithA16(LoadA16Target::A16))),
            0xFA => Some(Instruction::LD(LoadType::AWithA16(LoadA16Target::A))),
            // ADD Register to A
            0x80..=0x87 => Some(Instruction::ADD(OPType::LoadA(Self::hl_target_helper(
                byte,
            )))),
            0xC6 => Some(Instruction::ADD(OPType::LoadD8)), // ADD D8
            0xE8 => Some(Instruction::ADD(OPType::LoadSP)), // ADD s8 SP
            // ADD N16 Register to N16 Register
            0x09 => Some(Instruction::ADD(OPType::LoadHL(AddN16Target::BC))),
            0x19 => Some(Instruction::ADD(OPType::LoadHL(AddN16Target::DE))),
            0x29 => Some(Instruction::ADD(OPType::LoadHL(AddN16Target::HL))),
            0x39 => Some(Instruction::ADD(OPType::LoadHL(AddN16Target::SP))),
            // ADC Register to A
            0x88..=0x8F => Some(Instruction::ADC(OPType::LoadA(Self::hl_target_helper(
                byte,
            )))),
            0xCE => Some(Instruction::ADC(OPType::LoadD8)), // ADC D8
            // SUB
            0x90..=0x97 => Some(Instruction::SUB(OPType::LoadA(Self::hl_target_helper(
                byte,
            )))),
            0xD6 => Some(Instruction::SUB(OPType::LoadD8)), // SUB D8
            // SBC
            0x98..=0x9F => Some(Instruction::SBC(OPType::LoadA(Self::hl_target_helper(
                byte,
            )))),
            0xDE => Some(Instruction::SBC(OPType::LoadD8)), // SBC D8
            // AND
            0xA0..=0xA7 => Some(Instruction::AND(OPType::LoadA(Self::hl_target_helper(
                byte,
            )))),
            0xE6 => Some(Instruction::AND(OPType::LoadD8)), // AND D8
            // XOR
            0xA8..=0xAF => Some(Instruction::XOR(OPType::LoadA(Self::hl_target_helper(
                byte,
            )))),
            0xEE => Some(Instruction::XOR(OPType::LoadD8)), // XOR D8
            // OR
            0xB0..=0xB7 => Some(Instruction::OR(OPType::LoadA(Self::hl_target_helper(byte)))),
            0xF6 => Some(Instruction::OR(OPType::LoadD8)), // OR D8
            // CP
            0xB8..=0xBF => Some(Instruction::CP(OPType::LoadA(Self::hl_target_helper(byte)))),
            0xFE => Some(Instruction::CP(OPType::LoadD8)), // CP D8
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
            0xC2 => Some(Instruction::JP(JumpTest::NotZero)),
            0xCA => Some(Instruction::JP(JumpTest::Zero)),
            0xD2 => Some(Instruction::JP(JumpTest::NotCarry)),
            0xDA => Some(Instruction::JP(JumpTest::Carry)),
            0xC3 => Some(Instruction::JP(JumpTest::Always)),
            0xE9 => Some(Instruction::JP(JumpTest::HL)),
            // CALL
            0xC4 => Some(Instruction::CALL(JumpTest::NotZero)),
            0xCC => Some(Instruction::CALL(JumpTest::Zero)),
            0xD4 => Some(Instruction::CALL(JumpTest::NotCarry)),
            0xDC => Some(Instruction::CALL(JumpTest::Carry)),
            0xCD => Some(Instruction::CALL(JumpTest::Always)),
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
            0xF4 => Some(Instruction::DI),
            // EI
            0xFB => Some(Instruction::EI),
            _ => todo!("Implement more byte not prefixed for byte {:#02X}", byte),
        }
    }

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

impl CPU {
    // Contructor
    pub fn new(game_cart: Cartridge) -> Self {
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
            pc: 0x0100,
            sp: 0,
            memory: Memory::new(),
            cartridge: game_cart,
            is_halted: false,
            curr_opcode: 0,
            curr_instruction: None,
        }
    }

    // Function to 'step' through instructions
    pub fn step(&mut self) {
        // fetch next opcode from cartridge
        self.fetch();

        // Decode current opcode
        self.decode();

        // Execute the current instruction if it exists and reset it to none
        if let Some(instruction) = self.curr_instruction.take() {
            // Execute the current instruction
            let next_pc = self.execute(instruction);

            // Increment pc to returned pc
            self.pc += next_pc;
        } else {
            panic!("Decode Error: No Instruction")
        }
    }

    // Function to fetch next opcode
    fn fetch(&mut self) {
        self.curr_opcode = self.cartridge.read_byte(self.pc);
    }

    // Function to decode current opcode
    fn decode(&mut self) {
        let prefixed = self.curr_opcode == 0xCB;

        // Determine Instruction Byte
        let instruction_opcode = if prefixed {
            self.cartridge.read_byte(self.pc + 1)
        } else {
            self.curr_opcode
        };

        // Use enum to translate opcode and store next pc addr
        if prefixed {
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
    }

    // Function to execute an opcode by matching Instruction type and target then calling its method
    fn execute(&mut self, instruction: Instruction) -> u16 {
        // return while halted
        if self.is_halted {
            return self.pc;
        }

        match instruction {
            Instruction::NOP => {
                // Stands for no-operation and it effectively does nothing except advance the program counter by 1.
                print!("NOP");
                self.pc = self.pc.wrapping_add(1);
                self.pc
            }
            Instruction::STOP => {
                print!("STOP");
                self.pc + 1
            }
            Instruction::RLCA => {
                print!("RLCA");
                self.pc + 1
            }
            Instruction::RRCA => {
                print!("RRCA");
                self.pc + 1
            }
            Instruction::RLA => {
                print!("RLA");
                self.pc + 1
            }
            Instruction::RRA => {
                print!("RRA");
                self.pc + 1
            }
            Instruction::DAA => {
                print!("DAA");
                self.pc + 1
            }
            Instruction::SCF => {
                self.registers.f.carry = true;
                self.pc + 1
            }
            Instruction::CPL => {
                print!("CPL");
                self.pc + 1
            }
            Instruction::CCF => {
                self.registers.f.carry = !self.registers.f.carry;
                self.pc + 1
            }
            Instruction::JR(test) => {
                let jump_distance = self.cartridge.read_byte(self.pc + 1) as i8;
                match test {
                    JumpTest::NotZero => {
                        if !self.registers.f.zero {
                            self.pc = self.pc.wrapping_add(jump_distance as u16)
                        }
                    }
                    JumpTest::NotCarry => {
                        if !self.registers.f.carry {
                            self.pc = self.pc.wrapping_add(jump_distance as u16)
                        }
                    }
                    JumpTest::Always => self.pc = self.pc.wrapping_add(jump_distance as u16),
                    JumpTest::Zero => {
                        if self.registers.f.zero {
                            self.pc = self.pc.wrapping_add(jump_distance as u16)
                        }
                    }
                    JumpTest::Carry => {
                        if self.registers.f.carry {
                            self.pc = self.pc.wrapping_add(jump_distance as u16)
                        }
                    }
                    JumpTest::HL => {
                        panic!("BAD JR REQUEST");
                    }
                }
                self.pc + 1
            }
            Instruction::INC(target) => {
                match target {
                    // Increment 8-bit registers and update flags
                    AllRegisters::A => {
                        self.registers.a = self.registers.a.wrapping_add(1);
                        self.update_flags_after_inc(self.registers.a);
                    }
                    AllRegisters::B => {
                        self.registers.b = self.registers.b.wrapping_add(1);
                        self.update_flags_after_inc(self.registers.b);
                    }
                    AllRegisters::C => {
                        self.registers.c = self.registers.c.wrapping_add(1);
                        self.update_flags_after_inc(self.registers.c);
                    }
                    AllRegisters::D => {
                        self.registers.d = self.registers.d.wrapping_add(1);
                        self.update_flags_after_inc(self.registers.d);
                    }
                    AllRegisters::E => {
                        self.registers.e = self.registers.e.wrapping_add(1);
                        self.update_flags_after_inc(self.registers.e);
                    }
                    AllRegisters::H => {
                        self.registers.h = self.registers.h.wrapping_add(1);
                        self.update_flags_after_inc(self.registers.h);
                    }
                    AllRegisters::L => {
                        self.registers.l = self.registers.l.wrapping_add(1);
                        self.update_flags_after_inc(self.registers.l);
                    }
                    // Increment value at memory location HL
                    AllRegisters::HLMEM => {
                        let hl_addr = self.registers.get_hl();
                        let value = self.cartridge.read_byte(hl_addr).wrapping_add(1);
                        self.memory.write_byte(hl_addr, value);
                        self.update_flags_after_inc(value);
                    }
                    // 16-bit register increments (don't need to update flags for these)
                    AllRegisters::BC => {
                        let new_bc = self.registers.get_bc().wrapping_add(1);
                        self.registers.set_bc(new_bc);
                    }
                    AllRegisters::DE => {
                        let new_de = self.registers.get_de().wrapping_add(1);
                        self.registers.set_de(new_de);
                    }
                    AllRegisters::HL => {
                        let new_hl = self.registers.get_hl().wrapping_add(1);
                        self.registers.set_hl(new_hl);
                    }
                    AllRegisters::SP => {
                        self.sp = self.sp.wrapping_add(1);
                    }
                }
                self.pc + 1
            }
            Instruction::DEC(target) => {
                match target {
                    // Increment 8-bit registers and update flags
                    AllRegisters::A => {
                        let original_value = self.registers.a;
                        self.registers.a = self.registers.a.wrapping_sub(1);
                        self.update_flags_after_dec(self.registers.a, original_value);
                    }
                    AllRegisters::B => {
                        let original_value = self.registers.b;
                        self.registers.b = self.registers.b.wrapping_sub(1);
                        self.update_flags_after_dec(self.registers.b, original_value);
                    }
                    AllRegisters::C => {
                        let original_value = self.registers.c;
                        self.registers.c = self.registers.c.wrapping_sub(1);
                        self.update_flags_after_dec(self.registers.c, original_value);
                    }
                    AllRegisters::D => {
                        let original_value = self.registers.d;
                        self.registers.d = self.registers.d.wrapping_sub(1);
                        self.update_flags_after_dec(self.registers.d, original_value);
                    }
                    AllRegisters::E => {
                        let original_value = self.registers.e;
                        self.registers.e = self.registers.e.wrapping_sub(1);
                        self.update_flags_after_dec(self.registers.e, original_value);
                    }
                    AllRegisters::H => {
                        let original_value = self.registers.h;
                        self.registers.h = self.registers.h.wrapping_sub(1);
                        self.update_flags_after_dec(self.registers.h, original_value);
                    }
                    AllRegisters::L => {
                        let original_value = self.registers.l;
                        self.registers.l = self.registers.l.wrapping_sub(1);
                        self.update_flags_after_dec(self.registers.l, original_value);
                    }
                    // Increment value at memory location HL
                    AllRegisters::HLMEM => {
                        let hl_addr = self.registers.get_hl();
                        let original_value = self.cartridge.read_byte(hl_addr);
                        let value = self.cartridge.read_byte(hl_addr).wrapping_sub(1);
                        self.memory.write_byte(hl_addr, value);
                        self.update_flags_after_dec(value, original_value);
                    }
                    // 16-bit register increments (don't need to update flags for these)
                    AllRegisters::BC => {
                        let new_bc = self.registers.get_bc().wrapping_sub(1);
                        self.registers.set_bc(new_bc);
                    }
                    AllRegisters::DE => {
                        let new_de = self.registers.get_de().wrapping_sub(1);
                        self.registers.set_de(new_de);
                    }
                    AllRegisters::HL => {
                        let new_hl = self.registers.get_hl().wrapping_sub(1);
                        self.registers.set_hl(new_hl);
                    }
                    AllRegisters::SP => {
                        self.sp = self.sp.wrapping_sub(1);
                    }
                }
                self.pc + 1
            }
            Instruction::LD(target) => match target {
                LoadType::RegInReg(target, source) => match target {
                    HLTarget::B => match target {
                        HLTarget::B => {
                            self.registers.b = self.registers.b;
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.registers.b = self.registers.c;
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.registers.b = self.registers.d;
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.registers.b = self.registers.e;
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.registers.b = self.registers.h;
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.registers.b = self.registers.l;
                            self.pc + 1
                        }
                        HLTarget::HL => {
                            self.registers.b = self.memory.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.b = self.registers.a;
                            self.pc + 1
                        }
                    },
                    HLTarget::C => match target {
                        HLTarget::B => {
                            self.registers.c = self.registers.b;
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.registers.c = self.registers.c;
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.registers.c = self.registers.d;
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.registers.c = self.registers.e;
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.registers.c = self.registers.h;
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.registers.c = self.registers.l;
                            self.pc + 1
                        }
                        HLTarget::HL => {
                            self.registers.c = self.memory.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.c = self.registers.a;
                            self.pc + 1
                        }
                    },
                    HLTarget::D => match target {
                        HLTarget::B => {
                            self.registers.d = self.registers.b;
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.registers.d = self.registers.c;
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.registers.d = self.registers.d;
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.registers.d = self.registers.e;
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.registers.d = self.registers.h;
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.registers.d = self.registers.l;
                            self.pc + 1
                        }
                        HLTarget::HL => {
                            self.registers.d = self.memory.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.d = self.registers.a;
                            self.pc + 1
                        }
                    },
                    HLTarget::E => match target {
                        HLTarget::B => {
                            self.registers.e = self.registers.b;
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.registers.e = self.registers.c;
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.registers.e = self.registers.d;
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.registers.e = self.registers.e;
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.registers.e = self.registers.h;
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.registers.e = self.registers.l;
                            self.pc + 1
                        }
                        HLTarget::HL => {
                            self.registers.e = self.memory.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.e = self.registers.a;
                            self.pc + 1
                        }
                    },
                    HLTarget::H => match target {
                        HLTarget::B => {
                            self.registers.h = self.registers.b;
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.registers.h = self.registers.c;
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.registers.h = self.registers.d;
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.registers.h = self.registers.e;
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.registers.h = self.registers.h;
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.registers.h = self.registers.l;
                            self.pc + 1
                        }
                        HLTarget::HL => {
                            self.registers.h = self.memory.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.h = self.registers.a;
                            self.pc + 1
                        }
                    },
                    HLTarget::L => match target {
                        HLTarget::B => {
                            self.registers.l = self.registers.b;
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.registers.l = self.registers.c;
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.registers.l = self.registers.d;
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.registers.l = self.registers.e;
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.registers.l = self.registers.h;
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.registers.l = self.registers.l;
                            self.pc + 1
                        }
                        HLTarget::HL => {
                            self.registers.l = self.memory.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.l = self.registers.a;
                            self.pc + 1
                        }
                    },
                    HLTarget::HL => match target {
                        HLTarget::B => {
                            self.memory
                                .write_byte(self.registers.get_hl(), self.registers.b);
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.memory
                                .write_byte(self.registers.get_hl(), self.registers.c);
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.memory
                                .write_byte(self.registers.get_hl(), self.registers.d);
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.memory
                                .write_byte(self.registers.get_hl(), self.registers.e);
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.memory
                                .write_byte(self.registers.get_hl(), self.registers.h);
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.memory
                                .write_byte(self.registers.get_hl(), self.registers.l);
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.memory
                                .write_byte(self.registers.get_hl(), self.registers.a);
                            self.pc + 1
                        }
                        _ => panic!("Getting LD HL HL Should be HALT"),
                    },
                    HLTarget::A => match target {
                        HLTarget::B => {
                            self.registers.a = self.registers.b;
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.registers.a = self.registers.c;
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.registers.a = self.registers.d;
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.registers.a = self.registers.e;
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.registers.a = self.registers.h;
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.registers.a = self.registers.l;
                            self.pc + 1
                        }
                        HLTarget::HL => {
                            self.registers.a = self.memory.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.a = self.registers.a;
                            self.pc + 1
                        }
                    },
                },
                LoadType::Word(target, source) => {
                    // Read the next two bytes from memory at the current PC
                    let low_byte = self.cartridge.read_byte(self.pc + 1); // Read the low byte
                    let high_byte = self.cartridge.read_byte(self.pc + 2); // Read the high byte

                    // Combine the low and high bytes into a 16-bit value
                    let word_value = ((high_byte as u16) << 8) | (low_byte as u16);

                    match target {
                        LoadWordTarget::BC => match source {
                            LoadWordSource::N16 => {
                                self.registers
                                    .set_bc(self.cartridge.read_byte(word_value) as u16);
                                self.pc + 3
                            }
                            _ => panic!("BAD MATCH"),
                        },
                        LoadWordTarget::HL => match source {
                            LoadWordSource::N16 => {
                                self.registers
                                    .set_hl(self.cartridge.read_byte(word_value) as u16);
                                self.pc + 3
                            }
                            LoadWordSource::SPE8 => {
                                self.registers.set_hl(
                                    ((self.sp as i16).wrapping_add(
                                        (self.cartridge.read_byte(self.pc + 1) as i8) as i16,
                                    )) as u16,
                                );

                                // UPD FLAGS
                                self.registers.f.subtract = false;

                                self.registers.f.half_carry = ((self.sp & 0x0F)
                                    + (self.cartridge.read_byte(self.pc + 1) as u16 & 0x0F))
                                    > 0x0F;

                                self.registers.f.carry = ((self.sp & 0xFF)
                                    + (self.cartridge.read_byte(self.pc + 1) as u16 & 0xFF))
                                    > 0xFF;

                                self.pc + 2
                            }
                            _ => panic!("BAD MATCH"),
                        },
                        LoadWordTarget::DE => match source {
                            LoadWordSource::N16 => {
                                self.registers
                                    .set_de(self.cartridge.read_byte(word_value) as u16);
                                self.pc + 3
                            }
                            _ => panic!("BAD MATCH"),
                        },
                        LoadWordTarget::N16 => match source {
                            LoadWordSource::SP => {
                                self.memory.write_byte(word_value, (self.sp & 0x00FF) as u8);
                                self.memory.write_byte(word_value + 1, (self.sp >> 8) as u8);
                                self.pc + 3
                            }
                            _ => panic!("BAD MATCH"),
                        },
                        LoadWordTarget::SP => match source {
                            LoadWordSource::HL => {
                                self.registers.set_hl(self.sp);
                                self.pc + 1
                            }
                            LoadWordSource::N16 => {
                                self.sp = word_value;
                                self.pc + 3
                            }
                            _ => panic!("BAD MATCH"),
                        },
                    }
                }
                LoadType::AStoreInN16(target) => match target {
                    LoadN16::BC => {
                        self.memory
                            .write_byte(self.registers.get_bc(), self.registers.a);
                        self.pc + 1
                    }
                    LoadN16::DE => {
                        self.memory
                            .write_byte(self.registers.get_de(), self.registers.a);
                        self.pc + 1
                    }
                    LoadN16::HLDEC => {
                        self.memory
                            .write_byte(self.registers.get_hl(), self.registers.a);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_sub(1));
                        self.pc + 1
                    }
                    LoadN16::HLINC => {
                        self.memory
                            .write_byte(self.registers.get_hl(), self.registers.a);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_add(1));
                        self.pc + 1
                    }
                },
                LoadType::N16StoreInA(source) => match source {
                    LoadN16::BC => {
                        self.registers.a = self.memory.read_byte(self.registers.get_bc());
                        self.pc + 1
                    }
                    LoadN16::DE => {
                        self.registers.a = self.memory.read_byte(self.registers.get_de());
                        self.pc + 1
                    }
                    LoadN16::HLDEC => {
                        self.registers.a = self.memory.read_byte(self.registers.get_hl());
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_sub(1));
                        self.pc + 1
                    }
                    LoadN16::HLINC => {
                        self.registers.a = self.memory.read_byte(self.registers.get_hl());
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_add(1));
                        self.pc + 1
                    }
                },
                LoadType::D8StoreInReg(target) => match target {
                    HLTarget::B => {
                        self.registers.b = self.cartridge.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::C => {
                        self.registers.c = self.cartridge.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::D => {
                        self.registers.d = self.cartridge.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::E => {
                        self.registers.e = self.cartridge.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::H => {
                        self.registers.h = self.cartridge.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::L => {
                        self.registers.l = self.cartridge.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::HL => {
                        self.memory.write_byte(
                            self.registers.get_hl(),
                            self.cartridge.read_byte(self.pc + 1),
                        );
                        self.pc + 2
                    }
                    HLTarget::A => {
                        self.registers.a = self.cartridge.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                },
                LoadType::AWithA8(target) => match target {
                    LoadA8Target::A => {
                        self.registers.a = self
                            .memory
                            .read_byte(0xFF00 + self.cartridge.read_byte(self.pc + 1) as u16);
                        self.pc + 2
                    }
                    LoadA8Target::A8 => {
                        self.memory.write_byte(
                            0xFF00 + self.cartridge.read_byte(self.pc + 1) as u16,
                            self.registers.a,
                        );
                        self.pc + 2
                    }
                },
                LoadType::AWithA16(target) => {
                    let low_byte = self.cartridge.read_byte(self.pc + 1); // Read the low byte
                    let high_byte = self.cartridge.read_byte(self.pc + 2); // Read the high byte

                    // Combine the low and high bytes into a 16-bit value
                    let address = ((high_byte as u16) << 8) | (low_byte as u16);

                    match target {
                        LoadA16Target::A => {
                            self.registers.a = self.memory.read_byte(address);
                            self.pc + 3
                        }
                        LoadA16Target::A16 => {
                            self.memory.write_byte(address, self.registers.a);
                            self.pc + 3
                        }
                    }
                }
                LoadType::AWithAC(target) => match target {
                    LoadACTarget::A => {
                        self.memory
                            .write_byte(0xFF00 + self.registers.c as u16, self.registers.a);
                        self.pc + 2
                    }
                    LoadACTarget::C => {
                        self.registers.a = self.memory.read_byte(0xFF00 + self.registers.c as u16);
                        self.pc + 2
                    }
                },
            },
            Instruction::HALT => {
                // Instruction For Halting CPU Cycle
                self.is_halted = true;
                self.pc + 1
            }
            Instruction::ADD(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = self.match_hl(target);
                    // Store the original value of A
                    let original = self.registers.a;

                    // Update register A by adding the target value
                    self.registers.a = original.wrapping_add(reg_target);

                    // Set Flags
                    // Zero Flag: Set if the result is zero
                    self.registers.f.zero = self.registers.a == 0;

                    // Subtract Flag: Not set for ADD operations
                    self.registers.f.subtract = false;

                    // Half-Carry Flag: Set if there was a carry from bit 3 to bit 4
                    self.registers.f.half_carry = (original & 0x0F) + (reg_target & 0x0F) > 0x0F;

                    // Carry Flag: Set if the addition overflowed an 8-bit value
                    self.registers.f.carry = self.registers.a < original; // Check if an overflow occurred

                    self.pc + 1
                }
                OPType::LoadHL(target) => {
                    let reg_target = self.match_n16(target);
                    self.registers
                        .set_hl(self.registers.get_hl().wrapping_add(reg_target));

                    // Set Flags

                    // Carry Flag: Check for carry from the addition
                    self.registers.f.carry =
                        ((self.registers.get_hl() as u32) + (reg_target as u32)) > 0xFFFF;

                    // Half-Carry Flag: Check if there was a carry from bit 11 to bit 12
                    let half_carry =
                        ((self.registers.get_hl() & 0x0FFF) + (reg_target & 0x0FFF)) > 0x0FFF;
                    self.registers.f.half_carry = half_carry;

                    // Subtract Flag: Not set for ADD operations
                    self.registers.f.subtract = false;

                    // Zero Flag: Not affected, but set to false
                    self.registers.f.zero = false;

                    self.pc + 1
                }
                OPType::LoadSP => {
                    let immediate_operand: i8 = self.cartridge.read_byte(self.pc + 1) as i8;

                    // Sign-extend the immediate operand to 16 bits
                    let signed_value = immediate_operand as i16;

                    self.sp = self.sp.wrapping_add(signed_value as u16);

                    // Set Flags
                    self.registers.f.zero = self.sp == 0;

                    // Carry Flag: Check if there's a carry out (would occur if SP > 0xFFFF)
                    self.registers.f.carry = (self.sp as i16) < (signed_value as i16);

                    // Half-Carry Flag: Check if there's a carry from bit 11 to bit 12 this check is done based on the lower 4 bits
                    let half_carry =
                        ((self.sp & 0x0F) as i16 + (signed_value & 0x0F) as i16) > 0x0F;
                    self.registers.f.half_carry = half_carry;

                    self.registers.f.subtract = false;

                    self.pc + 2
                }
                OPType::LoadD8 => {
                    let immediate_operand: u8 = self.cartridge.read_byte(self.pc + 1);
                    let original = self.registers.a;
                    self.registers.a = self.registers.a.wrapping_add(immediate_operand);

                    // Set Flags
                    self.registers.f.zero = self.registers.a == 0;
                    self.registers.f.subtract = false;
                    // Half-Carry Flag: Set if there was a carry from bit 3 to bit 4
                    self.registers.f.half_carry =
                        ((original & 0x0F) + (self.registers.a & 0x0F)) > 0x0F;
                    // Carry Flag: Set if there was a carry out from the most significant bit
                    self.registers.f.carry =
                        (self.registers.a < original) || (self.registers.a < immediate_operand);

                    self.pc + 2
                }
            },
            Instruction::ADC(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = self.match_hl(target);
                    todo!()
                }
                OPType::LoadHL(target) => {
                    let reg_target = self.match_n16(target);
                    todo!()
                }
                OPType::LoadSP => {
                    todo!()
                }
                OPType::LoadD8 => {
                    todo!()
                }
            },
            Instruction::SUB(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = self.match_hl(target);
                    todo!()
                }
                OPType::LoadHL(target) => {
                    let reg_target = self.match_n16(target);
                    todo!()
                }
                OPType::LoadSP => {
                    todo!()
                }
                OPType::LoadD8 => {
                    todo!()
                }
            },
            Instruction::SBC(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = self.match_hl(target);
                    todo!()
                }
                OPType::LoadHL(target) => {
                    let reg_target = self.match_n16(target);
                    todo!()
                }
                OPType::LoadSP => {
                    todo!()
                }
                OPType::LoadD8 => {
                    todo!()
                }
            },
            Instruction::AND(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = self.match_hl(target);
                    todo!()
                }
                OPType::LoadHL(target) => {
                    let reg_target = self.match_n16(target);
                    todo!()
                }
                OPType::LoadSP => {
                    todo!()
                }
                OPType::LoadD8 => {
                    todo!()
                }
            },
            Instruction::XOR(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = self.match_hl(target);
                    todo!()
                }
                OPType::LoadHL(target) => {
                    let reg_target = self.match_n16(target);
                    todo!()
                }
                OPType::LoadSP => {
                    todo!()
                }
                OPType::LoadD8 => {
                    todo!()
                }
            },
            Instruction::OR(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = self.match_hl(target);
                    todo!()
                }
                OPType::LoadHL(target) => {
                    let reg_target = self.match_n16(target);
                    todo!()
                }
                OPType::LoadSP => {
                    todo!()
                }
                OPType::LoadD8 => {
                    todo!()
                }
            },
            Instruction::CP(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = self.match_hl(target);
                    todo!()
                }
                OPType::LoadHL(target) => {
                    let reg_target = self.match_n16(target);
                    todo!()
                }
                OPType::LoadSP => {
                    todo!()
                }
                OPType::LoadD8 => {
                    todo!()
                }
            },
            Instruction::RET(test) => {
                let jump_condition = self.match_jump(test);
                self.run_return(jump_condition);
                todo!()
            }
            Instruction::RETI => {
                todo!()
            }
            Instruction::POP(target) => {
                let result = self.pop();
                match target {
                    StackTarget::AF => self.registers.set_af(result),
                    StackTarget::BC => self.registers.set_bc(result),
                    StackTarget::DE => self.registers.set_de(result),
                    StackTarget::HL => self.registers.set_hl(result),
                }
                todo!()
            }
            Instruction::JP(test) => {
                let jump_condition = self.match_jump(test);
                self.jump(jump_condition)
            }
            Instruction::CALL(test) => {
                let jump_condition = self.match_jump(test);
                self.call(jump_condition);
                todo!()
            }
            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::AF => self.registers.get_af(),
                    StackTarget::BC => self.registers.get_bc(),
                    StackTarget::DE => self.registers.get_de(),
                    StackTarget::HL => self.registers.get_hl(),
                };
                // push value to stack
                self.push(value);

                // increment pc
                self.pc.wrapping_add(1)
            }
            Instruction::RST(target) => {
                todo!()
            }
            Instruction::DI => {
                todo!()
            }
            Instruction::EI => {
                todo!()
            }

            // PREFIXED INSTRUCTIONS
            Instruction::RLC(target) => {
                let reg_target = self.match_hl(target);
                todo!();
            }
            Instruction::RRC(target) => {
                let reg_target = self.match_hl(target);
                todo!();
            }
            Instruction::RL(target) => {
                let reg_target = self.match_hl(target);
                todo!();
            }
            Instruction::RR(target) => {
                let reg_target = self.match_hl(target);
                todo!();
            }
            Instruction::SLA(target) => {
                let reg_target = self.match_hl(target);
                todo!();
            }
            Instruction::SRA(target) => {
                let reg_target = self.match_hl(target);
                todo!();
            }
            Instruction::SWAP(target) => {
                let reg_target = self.match_hl(target);
                todo!();
            }
            Instruction::SRL(target) => {
                let reg_target = self.match_hl(target);
                todo!();
            }
            Instruction::BIT(target) => match target {
                ByteTarget::Zero(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::One(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Two(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Three(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Four(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Five(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Six(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Seven(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
            },
            Instruction::RES(target) => match target {
                ByteTarget::Zero(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::One(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Two(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Three(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Four(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Five(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Six(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Seven(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
            },
            Instruction::SET(target) => match target {
                ByteTarget::Zero(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::One(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Two(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Three(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Four(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Five(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Six(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
                ByteTarget::Seven(hl_target) => {
                    let reg_target = self.match_hl(hl_target);
                    todo!()
                }
            },

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
        if jump {
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
        if should_jump {
            self.push(next_pc);
            self.memory.read_next_byte();
            todo!()
        } else {
            next_pc
        }
    }

    // Return function for returning through call stack
    fn run_return(&mut self, jump_condition: bool) -> u16 {
        if jump_condition {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }

    // Method to match a N16 Target
    fn match_n16(&self, target: AddN16Target) -> u16 {
        let reg_target = match target {
            AddN16Target::BC => self.registers.get_bc(),
            AddN16Target::DE => self.registers.get_de(),
            AddN16Target::HL => self.registers.get_hl(),
            AddN16Target::SP => self.sp,
        };
        reg_target
    }

    // Method to match Jump Condition
    fn match_jump(&self, test: JumpTest) -> bool {
        let jump_condition = match test {
            JumpTest::NotZero => !self.registers.f.zero,
            JumpTest::NotCarry => !self.registers.f.carry,
            JumpTest::Zero => !self.registers.f.zero,
            JumpTest::Carry => !self.registers.f.carry,
            JumpTest::Always => true,
            JumpTest::HL => panic!("HL BAD"),
        };
        jump_condition
    }

    // Method to match to All Registers as u16
    fn match_all_registers(&self, target: AllRegisters) -> u16 {
        let reg_target = match target {
            AllRegisters::A => self.registers.a as u16,
            AllRegisters::B => self.registers.b as u16,
            AllRegisters::C => self.registers.c as u16,
            AllRegisters::D => self.registers.d as u16,
            AllRegisters::E => self.registers.e as u16,
            AllRegisters::H => self.registers.h as u16,
            AllRegisters::L => self.registers.l as u16,
            AllRegisters::HLMEM => self.cartridge.read_byte(self.registers.get_hl()) as u16,
            AllRegisters::BC => self.registers.get_bc(),
            AllRegisters::DE => self.registers.get_de(),
            AllRegisters::HL => self.registers.get_hl(),
            AllRegisters::SP => self.sp,
        };
        reg_target
    }

    // Method to match a hl target to its register
    fn match_hl(&self, target: HLTarget) -> u8 {
        let reg_source = match target {
            HLTarget::A => self.registers.a,
            HLTarget::B => self.registers.b,
            HLTarget::C => self.registers.c,
            HLTarget::D => self.registers.d,
            HLTarget::E => self.registers.e,
            HLTarget::H => self.registers.h,
            HLTarget::L => self.registers.l,
            HLTarget::HL => self.memory.read_byte(self.registers.get_hl()),
        };
        reg_source
    }

    // Method to update relevant flags after INC operation
    fn update_flags_after_inc(&mut self, result: u8) {
        // Zero Flag: Set if the result is zero
        self.registers.f.zero = result == 0;

        // Subtract Flag: Reset (INC is an addition)
        self.registers.f.subtract = false;

        // Half-Carry Flag: Set if there was a carry from bit 3 to bit 4
        let half_carry = (result & 0x0F) == 0;
        self.registers.f.half_carry = half_carry;
    }

    // Method to update relevant flags after DEC operation
    fn update_flags_after_dec(&mut self, result: u8, original_value: u8) {
        // Zero Flag: Set if the result is zero
        self.registers.f.zero = result == 0;

        // Subtract Flag: SET (DEC is a subtraction)
        self.registers.f.subtract = true;

        // Half-Carry Flag: Set if there was a borrow from bit 4 to bit 3
        let half_carry = (original_value & 0x0F) == 0x00; // Borrow occurs if lower nibble was 0 before decrement
        self.registers.f.half_carry = half_carry;
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
