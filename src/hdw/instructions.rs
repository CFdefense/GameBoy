/*

    File to contain all Enumerations for Instructions and their expected targets and target sources
    As well as all implementations of Instruction operations such as decoding and matching bytes to instructions

*/
use super::bus::Bus;

#[derive(Copy, Clone)]
pub struct Instruction {
    pub in_type: InType,
    pub mode: AddrMode,
    pub reg_1: RegType,
    pub reg_2: RegType,
    pub cond: CondType,
    pub param: u8,
}

#[derive(Copy, Clone)]
pub enum InType {
    IN_NONE,
    IN_NOP,
    IN_LD,
    IN_INC,
    IN_DEC,
    IN_RLCA,
    IN_ADD,
    IN_RRCA,
    IN_STOP,
    IN_RLA,
    IN_JR,
    IN_RRA,
    IN_DAA,
    IN_CPL,
    IN_SCF,
    IN_CCF,
    IN_HALT,
    IN_ADC,
    IN_SUB,
    IN_SBC,
    IN_AND,
    IN_XOR,
    IN_OR,
    IN_CP,
    IN_POP,
    IN_JP,
    IN_PUSH,
    IN_RET,
    IN_CB,
    IN_CALL,
    IN_RETI,
    IN_LDH,
    IN_JPHL,
    IN_DI,
    IN_EI,
    IN_RST,
    IN_ERR,
    //CB instructions...
    IN_RLC,
    IN_RRC,
    IN_RL,
    IN_RR,
    IN_SLA,
    IN_SRA,
    IN_SWAP,
    IN_SRL,
    IN_BIT,
    IN_RES,
    IN_SET,
}

#[derive(Copy, Clone)]
pub enum AddrMode {
    AM_IMP,
    AM_R_D16,
    AM_R_R,
    AM_MR_R,
    AM_R,
    AM_R_D8,
    AM_R_MR,
    AM_R_HLI,
    AM_R_HLD,
    AM_HLI_R,
    AM_HLD_R,
    AM_R_A8,
    AM_A8_R,
    AM_HL_SPR,
    AM_D16,
    AM_D8,
    AM_D16_R,
    AM_MR_D8,
    AM_MR,
    AM_A16_R,
    AM_R_A16,
}

#[derive(Copy, Clone)]
pub enum RegType {
    RT_NONE,
    RT_A,
    RT_F,
    RT_B,
    RT_C,
    RT_D,
    RT_E,
    RT_H,
    RT_L,
    RT_AF,
    RT_BC,
    RT_DE,
    RT_HL,
    RT_SP,
    RT_PC,
}

#[derive(Copy, Clone)]
pub enum CondType {
    CT_NONE,
    CT_NZ,
    CT_Z,
    CT_NC,
    CT_C,
}

impl Instruction {
    // Function to take opcode from cpu and match it to a corresponding Instruction
    pub fn decode_from_opcode(opcode: u8, cart: &Bus, pc: u16) -> Option<Instruction> {
        let prefixed = opcode == 0xCB;

        // determine if instruction is a PREFIX
        let instruction_opcode = if prefixed {
            cart.read_byte(None, pc + 1)
        } else {
            opcode
        };

        // Use enum to translate opcode and store next pc addr
        let instruction = if prefixed {
            Instruction::decode_prefixed_opcode(instruction_opcode)
        } else {
            Instruction::decode_opcode(instruction_opcode)
        };

        // Implicit Return
        instruction
    }

    // Reutrn
    fn decode_opcode(opcode: u8) -> Option<Instruction> {
        if (opcode as usize) < Self::INSTRUCTIONS.len() {
            Some(Self::INSTRUCTIONS[opcode as usize])
        } else {
            None
        }
    }

    fn decode_prefixed_opcode(opcode: u8) -> Option<Instruction> {}

    const INSTRUCTIONS: [Option<Instruction>; 256] = {
        let mut table: [Option<Instruction>; 256] = [None; 256];

        table[0x00] = Some(Instruction {
            in_type: InType::IN_NOP,
            mode: AddrMode::AM_IMP,
        });
        table[0x01] = Some(Instruction {
            in_type: IN_LD,
            mode: AddrMode::AM_R_D16, z
            reg_1: RegType::RT_BC,
        });
        table[0x02] = Some(Instruction 
            in_type =IN_LD, AM_MR_R, RT_BC, RT_A},
        table[0x03] = Some(Instruction 
            in_type =IN_INC, AM_R, RT_BC},
        table[0x04] = Some(Instruction 
            in_type =IN_INC, AM_R, RT_B},
        table[0x05] = Some(Instruction 
            in_type =IN_DEC, AM_R, RT_B},
        table[0x06] = Some(Instruction 
            in_type =IN_LD, AM_R_D8, RT_B},
        table[0x07] = Some(Instruction 
            in_type =IN_RLCA},
        table[0x08] = Some(Instruction 
            in_type =IN_LD, AM_A16_R, RT_NONE, RT_SP},
        table[0x09] = Some(Instruction 
            in_type =IN_ADD, AM_R_R, RT_HL, RT_BC},
        table[0x0A] = Some(Instruction 
            in_type =IN_LD, AM_R_MR, RT_A, RT_BC},
        table[0x0B] = Some(Instruction 
            in_type =IN_DEC, AM_R, RT_BC},
        table[0x0C] = Some(Instruction 
            in_type =IN_INC, AM_R, RT_C},
        table[0x0D] = Some(Instruction 
            in_type =IN_DEC, AM_R, RT_C},
        table[0x0E] = Some(Instruction 
            in_type =IN_LD, AM_R_D8, RT_C},
        table[0x0F] = Some(Instruction 
            in_type =IN_RRCA},

        //0x1X
        table[0x10] = Some(Instruction
            in_type = IN_STOP},
        table[0x11] = Some(Instruction
            in_type = IN_LD, AM_R_D16, RT_DE},
        table[0x12] = Some(Instruction
            in_type = IN_LD, AM_MR_R, RT_DE, RT_A},
        table[0x13] = Some(Instruction
            in_type = IN_INC, AM_R, RT_DE},
        table[0x14] = Some(Instruction
            in_type = IN_INC, AM_R, RT_D},
        table[0x15] = Some(Instruction
            in_type = IN_DEC, AM_R, RT_D},
        table[0x16] = Some(Instruction
            in_type = IN_LD, AM_R_D8, RT_D},
        table[0x17] = Some(Instruction
            in_type = IN_RLA},
        table[0x18] = Some(Instruction
            in_type = IN_JR, AM_D8},
        table[0x19] = Some(Instruction
            in_type = IN_ADD, AM_R_R, RT_HL, RT_DE},
        table[0x1A] = Some(Instruction
            in_type = IN_LD, AM_R_MR, RT_A, RT_DE},
        table[0x1B] = Some(Instruction
            in_type = IN_DEC, AM_R, RT_DE},
        table[0x1C] = Some(Instruction
            in_type = IN_INC, AM_R, RT_E},
        table[0x1D] = Some(Instruction
            in_type = IN_DEC, AM_R, RT_E},
        table[0x1E] = Some(Instruction
            in_type = IN_LD, AM_R_D8, RT_E},
        table[0x1F] = Some(Instruction
            in_type = IN_RRA},

        //0x2X
        table[0x20] = {IN_JR, AM_D8, RT_NONE, RT_NONE, CT_NZ},
        table[0x21] = {IN_LD, AM_R_D16, RT_HL},
        table[0x22] = {IN_LD, AM_HLI_R, RT_HL, RT_A},
        table[0x23] = {IN_INC, AM_R, RT_HL},
        table[0x24] = {IN_INC, AM_R, RT_H},
        table[0x25] = {IN_DEC, AM_R, RT_H},
        table[0x26] = {IN_LD, AM_R_D8, RT_H},
        table[0x28] = {IN_JR, AM_D8, RT_NONE, RT_NONE, CT_Z},
        table[0x29] = {IN_ADD, AM_R_R, RT_HL, RT_HL},
        table[0x2A] = {IN_LD, AM_R_HLI, RT_A, RT_HL},
        table[0x2B] = {IN_DEC, AM_R, RT_HL},
        table[0x2C] = {IN_INC, AM_R, RT_L},
        table[0x2D] = {IN_DEC, AM_R, RT_L},
        table[0x2E] = {IN_LD, AM_R_D8, RT_L},

        //0x3X
        table[0x30] = {IN_JR, AM_D8, RT_NONE, RT_NONE, CT_NC},
        table[0x31] = {IN_LD, AM_R_D16, RT_SP},
        table[0x32] = {IN_LD, AM_HLD_R, RT_HL, RT_A},
        table[0x33] = {IN_INC, AM_R, RT_SP},
        table[0x34] = {IN_INC, AM_MR, RT_HL},
        table[0x35] = {IN_DEC, AM_MR, RT_HL},
        table[0x36] = {IN_LD, AM_MR_D8, RT_HL},
        table[0x38] = {IN_JR, AM_D8, RT_NONE, RT_NONE, CT_C},
        table[0x39] = {IN_ADD, AM_R_R, RT_HL, RT_SP},
        table[0x3A] = {IN_LD, AM_R_HLD, RT_A, RT_HL},
        table[0x3B] = {IN_DEC, AM_R, RT_SP},
        table[0x3C] = {IN_INC, AM_R, RT_A},
        table[0x3D] = {IN_DEC, AM_R, RT_A},
        table[0x3E] = {IN_LD, AM_R_D8, RT_A},

        //0x4X
        table[0x40] = {IN_LD, AM_R_R, RT_B, RT_B},
        table[0x41] = {IN_LD, AM_R_R, RT_B, RT_C},
        table[0x42] = {IN_LD, AM_R_R, RT_B, RT_D},
        table[0x43] = {IN_LD, AM_R_R, RT_B, RT_E},
        table[0x44] = {IN_LD, AM_R_R, RT_B, RT_H},
        table[0x45] = {IN_LD, AM_R_R, RT_B, RT_L},
        table[0x46] = {IN_LD, AM_R_MR, RT_B, RT_HL},
        table[0x47] = {IN_LD, AM_R_R, RT_B, RT_A},
        table[0x48] = {IN_LD, AM_R_R, RT_C, RT_B},
        table[0x49] = {IN_LD, AM_R_R, RT_C, RT_C},
        table[0x4A] = {IN_LD, AM_R_R, RT_C, RT_D},
        table[0x4B] = {IN_LD, AM_R_R, RT_C, RT_E},
        table[0x4C] = {IN_LD, AM_R_R, RT_C, RT_H},
        table[0x4D] = {IN_LD, AM_R_R, RT_C, RT_L},
        table[0x4E] = {IN_LD, AM_R_MR, RT_C, RT_HL},
        table[0x4F] = {IN_LD, AM_R_R, RT_C, RT_A},

        //0x5X
        table[0x50] = {IN_LD, AM_R_R,  RT_D, RT_B},
        table[0x51] = {IN_LD, AM_R_R,  RT_D, RT_C},
        table[0x52] = {IN_LD, AM_R_R,  RT_D, RT_D},
        table[0x53] = {IN_LD, AM_R_R,  RT_D, RT_E},
        table[0x54] = {IN_LD, AM_R_R,  RT_D, RT_H},
        table[0x55] = {IN_LD, AM_R_R,  RT_D, RT_L},
        table[0x56] = {IN_LD, AM_R_MR, RT_D, RT_HL},
        table[0x57] = {IN_LD, AM_R_R,  RT_D, RT_A},
        table[0x58] = {IN_LD, AM_R_R,  RT_E, RT_B},
        table[0x59] = {IN_LD, AM_R_R,  RT_E, RT_C},
        table[0x5A] = {IN_LD, AM_R_R,  RT_E, RT_D},
        table[0x5B] = {IN_LD, AM_R_R,  RT_E, RT_E},
        table[0x5C] = {IN_LD, AM_R_R,  RT_E, RT_H},
        table[0x5D] = {IN_LD, AM_R_R,  RT_E, RT_L},
        table[0x5E] = {IN_LD, AM_R_MR, RT_E, RT_HL},
        table[0x5F] = {IN_LD, AM_R_R,  RT_E, RT_A},

        //0x6X
        table[0x60] = {IN_LD, AM_R_R,  RT_H, RT_B},
        table[0x61] = {IN_LD, AM_R_R,  RT_H, RT_C},
        table[0x62] = {IN_LD, AM_R_R,  RT_H, RT_D},
        table[0x63] = {IN_LD, AM_R_R,  RT_H, RT_E},
        table[0x64] = {IN_LD, AM_R_R,  RT_H, RT_H},
        table[0x65] = {IN_LD, AM_R_R,  RT_H, RT_L},
        table[0x66] = {IN_LD, AMtable_R_MR, RT_H, RT_HL},
        table[0x67] = {IN_LD, AM_R_R,  RT_H, RT_A},
        table[0x68] = {IN_LD, AM_R_R,  RT_L, RT_B},
        table[0x69] = {IN_LD, AM_R_R,  RT_L, RT_C},
        table[0x6A] = {IN_LD, AM_R_R,  RT_L, RT_D},
        table[0x6B] = {IN_LD, AM_R_R,  RT_L, RT_E},
        table[0x6C] = {IN_LD, AM_R_R,  RT_L, RT_H},
        table[0x6D] = {IN_LD, AM_R_R,  RT_L, RT_L},
        table[0x6E] = {IN_LD, AM_R_MR, RT_L, RT_HL},
        table[0x6F] = {IN_LD, AM_R_R,  RT_L, RT_A},

        //0x7X
        table[0x70] = {IN_LD, AM_MR_R,  RT_HL, RT_B},
        table[0x71] = {IN_LD, AM_MR_R,  RT_HL, RT_C},
        table[0x72] = {IN_LD, AM_MR_R,  RT_HL, RT_D},
        table[0x73] = {IN_LD, AM_MR_R,  RT_HL, RT_E},
        table[0x74] = {IN_LD, AM_MR_R,  RT_HL, RT_H},
        table[0x75] = {IN_LD, AM_MR_R,  RT_HL, RT_L},
        table[0x76] = {IN_HALT},
        table[0x77] = {IN_LD, AM_MR_R,  RT_HL, RT_A},
        table[0x78] = {IN_LD, AM_R_R,  RT_A, RT_B},
        table[0x79] = {IN_LD, AM_R_R,  RT_A, RT_C},
        table[0x7A] = {IN_LD, AM_R_R,  RT_A, RT_D},
        table[0x7B] = {IN_LD, AM_R_R,  RT_A, RT_E},
        table[0x7C] = {IN_LD, AM_R_R,  RT_A, RT_H},
        table[0x7D] = {IN_LD, AM_R_R,  RT_A, RT_L},
        table[0x7E] = {IN_LD, AM_R_MR, RT_A, RT_HL},
        table[0x7F] = {IN_LD, AM_R_R,  RT_A, RT_A},

        //0x8X
        table[0x80] = {IN_ADD, AM_R_R, RT_A, RT_B},
        table[0x81] = {IN_ADD, AM_R_R, RT_A, RT_C},
        table[0x82] = {IN_ADD, AM_R_R, RT_A, RT_D},
        table[0x83] = {IN_ADD, AM_R_R, RT_A, RT_E},
        table[0x84] = {IN_ADD, AM_R_R, RT_A, RT_H},
        table[0x85] = {IN_ADD, AM_R_R, RT_A, RT_L},
        table[0x86] = {IN_ADD, AM_R_MR, RT_A, RT_HL},
        table[0x87] = {IN_ADD, AM_R_R, RT_A, RT_A},
        table[0x88] = {IN_ADC, AM_R_R, RT_A, RT_B},
        table[0x89] = {IN_ADC, AM_R_R, RT_A, RT_C},
        table[0x8A] = {IN_ADC, AM_R_R, RT_A, RT_D},
        table[0x8B] = {IN_ADC, AM_R_R, RT_A, RT_E},
        table[0x8C] = {IN_ADC, AM_R_R, RT_A, RT_H},
        table[0x8D] = {IN_ADC, AM_R_R, RT_A, RT_L},
        table[0x8E] = {IN_ADC, AM_R_MR, RT_A, RT_HL},
        table[0x8F] = {IN_ADC, AM_R_R, RT_A, RT_A},

        //0x9X
        table[0x90] = {IN_SUB, AM_R_R, RT_A, RT_B},
        table[0x91] = {IN_SUB, AM_R_R, RT_A, RT_C},
        table[0x92] = {IN_SUB, AM_R_R, RT_A, RT_D},
        table[0x93] = {IN_SUB, AM_R_R, RT_A, RT_E},
        table[0x94] = {IN_SUB, AM_R_R, RT_A, RT_H},
        table[0x95] = {IN_SUB, AM_R_R, RT_A, RT_L},
        table[0x96] = {IN_SUB, AM_R_MR, RT_A, RT_HL},
        table[0x97] = {IN_SUB, AM_R_R, RT_A, RT_A},
        table[0x98] = {IN_SBC, AM_R_R, RT_A, RT_B},
        table[0x99] = {IN_SBC, AM_R_R, RT_A, RT_C},
        table[0x9A] = {IN_SBC, AM_R_R, RT_A, RT_D},
        table[0x9B] = {IN_SBC, AM_R_R, RT_A, RT_E},
        table[0x9C] = {IN_SBC, AM_R_R, RT_A, RT_H},
        table[0x9D] = {IN_SBC, AM_R_R, RT_A, RT_L},
        table[0x9E] = {IN_SBC, AM_R_MR, RT_A, RT_HL},
        table[0x9F] = {IN_SBC, AM_R_R, RT_A, RT_A},


        //0xAX
        table[0xA0] = {IN_AND, AM_R_R, RT_A, RT_B},
        table[0xA1] = {IN_AND, AM_R_R, RT_A, RT_C},
        table[0xA2] = {IN_AND, AM_R_R, RT_A, RT_D},
        table[0xA3] = {IN_AND, AM_R_R, RT_A, RT_E},
        table[0xA4] = {IN_AND, AM_R_R, RT_A, RT_H},
        table[0xA5] = {IN_AND, AM_R_R, RT_A, RT_L},
        table[0xA6] = {IN_AND, AM_R_MR, RT_A, RT_HL},
        table[0xA7] = {IN_AND, AM_R_R, RT_A, RT_A},
        table[0xA8] = {IN_XOR, AM_R_R, RT_A, RT_B},
        table[0xA9] = {IN_XOR, AM_R_R, RT_A, RT_C},
        table[0xAA] = {IN_XOR, AM_R_R, RT_A, RT_D},
        table[0xAB] = {IN_XOR, AM_R_R, RT_A, RT_E},
        table[0xAC] = {IN_XOR, AM_R_R, RT_A, RT_H},
        table[0xAD] = {IN_XOR, AM_R_R, RT_A, RT_L},
        table[0xAE] = {IN_XOR, AM_R_MR, RT_A, RT_HL},
        table[0xAF] = {IN_XOR, AM_R_R, RT_A, RT_A},

        //0xBX
        table[0xB0] = {IN_OR, AM_R_R, RT_A, RT_B},
        table[0xB1] = {IN_OR, AM_R_R, RT_A, RT_C},
        table[0xB2] = {IN_OR, AM_R_R, RT_A, RT_D},
        table[0xB3] = {IN_OR, AM_R_R, RT_A, RT_E},
        table[0xB4] = {IN_OR, AM_R_R, RT_A, RT_H},
        table[0xB5] = {IN_OR, AM_R_R, RT_A, RT_L},
        table[0xB6] = {IN_OR, AM_R_MR, RT_A, RT_HL},
        table[0xB7] = {IN_OR, AM_R_R, RT_A, RT_A},
        table[0xB8] = {IN_CP, AM_R_R, RT_A, RT_B},
        table[0xB9] = {IN_CP, AM_R_R, RT_A, RT_C},
        table[0xBA] = {IN_CP, AM_R_R, RT_A, RT_D},
        table[0xBB] = {IN_CP, AM_R_R, RT_A, RT_E},
        table[0xBC] = {IN_CP, AM_R_R, RT_A, RT_H},
        table[0xBD] = {IN_CP, AM_R_R, RT_A, RT_L},
        table[0xBE] = {IN_CP, AM_R_MR, RT_A, RT_HL},
        table[0xBF] = {IN_CP, AM_R_R, RT_A, RT_A},

        table[0xC0] = {IN_RET, AM_IMP, RT_NONE, RT_NONE, CT_NZ},
        table[0xC1] = {IN_POP, AM_R, RT_BC},
        table[0xC2] = {IN_JP, AM_D16, RT_NONE, RT_NONE, CT_NZ},
        table[0xC3] = {IN_JP, AM_D16},
        table[0xC4] = {IN_CALL, AM_D16, RT_NONE, RT_NONE, CT_NZ},
        table[0xC5] = {IN_PUSH, AM_R, RT_BC},
        table[0xC6] = {IN_ADD, AM_R_A8, RT_A},
        table[0xC7] = {IN_RST, AM_IMP, RT_NONE, RT_NONE, CT_NONE, 0x00},
        table[0xC8] = {IN_RET, AM_IMP, RT_NONE, RT_NONE, CT_Z},
        table[0xC9] = {IN_RET},
        table[0xCA] = {IN_JP, AM_D16, RT_NONE, RT_NONE, CT_Z},
        table[0xCB] = {IN_CB, AM_D8},
        table[0xCC] = {IN_CALL, AM_D16, RT_NONE, RT_NONE, CT_Z},
        table[0xCD] = {IN_CALL, AM_D16},
        table[0xCE] = {IN_ADC, AM_R_D8, RT_A},
        table[0xCF] = {IN_RST, AM_IMP, RT_NONE, RT_NONE, CT_NONE, 0x08},

        table[0xD0] = {IN_RET, AM_IMP, RT_NONE, RT_NONE, CT_NC},
        table[0xD1] = {IN_POP, AM_R, RT_DE},
        table[0xD2] = {IN_JP, AM_D16, RT_NONE, RT_NONE, CT_NC},
        table[0xD4] = {IN_CALL, AM_D16, RT_NONE, RT_NONE, CT_NC},
        table[0xD5] = {IN_PUSH, AM_R, RT_DE},
        table[0xD6] = {IN_SUB, AM_D8},
        table[0xD7] = {IN_RST, AM_IMP, RT_NONE, RT_NONE, CT_NONE, 0x10},
        table[0xD8] = {IN_RET, AM_IMP, RT_NONE, RT_NONE, CT_C},
        table[0xD9] = {IN_RETI},
        table[0xDA] = {IN_JP, AM_D16, RT_NONE, RT_NONE, CT_C},
        table[0xDC] = {IN_CALL, AM_D16, RT_NONE, RT_NONE, CT_C},
        table[0xDE] = {IN_SBC, AM_R_D8, RT_A},
        table[0xDF] = {IN_RST, AM_IMP, RT_NONE, RT_NONE, CT_NONE, 0x18},

        //0xEX
        table[0xE0] = {IN_LDH, AM_A8_R, RT_NONE, RT_A},
        table[0xE1] = {IN_POP, AM_R, RT_HL},
        table[0xE2] = {IN_LD, AM_MR_R, RT_C, RT_A},
        table[0xE5] = {IN_PUSH, AM_R, RT_HL},
        table[0xE6] = {IN_AND, AM_D8},
        table[0xE7] = {IN_RST, AM_IMP, RT_NONE, RT_NONE, CT_NONE, 0x20},
        table[0xE8] = {IN_ADD, AM_R_D8, RT_SP},
        table[0xE9] = {IN_JP, AM_MR, RT_HL},
        table[0xEA] = {IN_LD, AM_A16_R, RT_NONE, RT_A},
        table[0xEE] = {IN_XOR, AM_D8},
        table[0xEF] = {IN_RST, AM_IMP, RT_NONE, RT_NONE, CT_NONE, 0x28},


        //0xFX
        table[0xF0] = {IN_LDH, AM_R_A8, RT_A},
        table[0xF1] = {IN_POP, AM_R, RT_AF},
        table[0xF2] = {IN_LD, AM_R_MR, RT_A, RT_C},
        table[0xF3] = {IN_DI},
        table[0xF5] = {IN_PUSH, AM_R, RT_AF},
        table[0xF6] = {IN_OR, AM_D8},
        table[0xF7] = {IN_RST, AM_IMP, RT_NONE, RT_NONE, CT_NONE, 0x30},
        table[0xF8] = {IN_LD, AM_HL_SPR, RT_HL, RT_SP},
        table[0xF9] = {IN_LD, AM_R_R, RT_SP, RT_HL},
        table[0xFA] = {IN_LD, AM_R_A16, RT_A},
        table[0xFB] = {IN_EI},
        table[0xFE] = {IN_CP, AM_D8},
        table[0xFF] = {IN_RST, AM_IMP, RT_NONE, RT_NONE, CT_NONE, 0x38},

        table
    };
}
