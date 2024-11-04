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
    pub const fn new(
        in_type: InType,
        mode: AddrMode,
        reg_1: RegType,
        reg_2: RegType,
        cond: CondType,
        param: u8,
    ) -> Instruction {
        Instruction {
            in_type,
            mode,
            reg_1,
            reg_2,
            cond,
            param,
        }
    }

    pub const fn default() -> Instruction {
        Instruction {
            in_type: InType::IN_NONE,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        }
    }

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
        Self::NP_INSTRUCTIONS[opcode as usize]
    }

    fn decode_prefixed_opcode(opcode: u8) -> Option<Instruction> {
        Self::PRE_INSTRUCTIONS[opcode as usize]
    }

    const NP_INSTRUCTIONS: [Option<Instruction>; 256] = {
        let mut table: [Option<Instruction>; 256] = [None; 256];

        table[0x00] = Some(Instruction {
            in_type: InType::IN_NOP,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x01] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D16,
            reg_1: RegType::RT_BC,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x02] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_BC,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x03] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_BC,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x04] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x05] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x06] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x07] = Some(Instruction {
            in_type: InType::IN_RLCA,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x08] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_A16_R,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_SP,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x09] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_BC,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x0A] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_BC,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x0B] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_BC,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x0C] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x0D] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x0E] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x0F] = Some(Instruction {
            in_type: InType::IN_RRCA,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0x1X
        table[0x10] = Some(Instruction {
            in_type: InType::IN_STOP,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x11] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D16,
            reg_1: RegType::RT_DE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x12] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_DE,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x13] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_DE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x14] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x15] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x16] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x17] = Some(Instruction {
            in_type: InType::IN_RLA,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x18] = Some(Instruction {
            in_type: InType::IN_JR,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x19] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_DE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x1A] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_DE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x1B] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_DE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x1C] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x1D] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x1E] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x1F] = Some(Instruction {
            in_type: InType::IN_RRA,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0x2X
        table[0x20] = Some(Instruction {
            in_type: InType::IN_JR,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NZ,
            param: 0,
        });
        table[0x21] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D16,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x22] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_HLI_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x23] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x24] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x25] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x26] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x27] = Some(Instruction {
            in_type: InType::IN_DAA,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x28] = Some(Instruction {
            in_type: InType::IN_JR,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_Z,
            param: 0,
        });
        table[0x29] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x2A] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_HLI,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x2B] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x2C] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x2D] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x2E] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        //0x3X
        // 0x3X
        table[0x30] = Some(Instruction {
            in_type: InType::IN_JR,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NC,
            param: 0,
        });

        table[0x31] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D16,
            reg_1: RegType::RT_SP,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x32] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_HLD_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x33] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_SP,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x34] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_MR,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x35] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_MR,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x36] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_D8,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x37] = Some(Instruction {
            in_type: InType::IN_SCF,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x38] = Some(Instruction {
            in_type: InType::IN_JR,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_C,
            param: 0,
        });

        table[0x39] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_SP,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x3A] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_HLD,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x3B] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_SP,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x3C] = Some(Instruction {
            in_type: InType::IN_INC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x3D] = Some(Instruction {
            in_type: InType::IN_DEC,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x3E] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0x4X
        table[0x40] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x41] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x42] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x43] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x44] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x45] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x46] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x47] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_B,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x48] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x49] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x4A] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x4B] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x4C] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x4D] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x4E] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x4F] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        //0x5X
        table[0x50] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x51] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x52] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x53] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x54] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x55] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x56] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x57] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_D,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        table[0x58] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x59] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x5A] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x5B] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x5C] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x5D] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x5E] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x5F] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_E,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        //0x6X
        table[0x60] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x61] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x62] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x63] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x64] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x65] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x66] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x67] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_H,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x68] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x69] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x6A] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x6B] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x6C] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x6D] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x6E] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x6F] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_L,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0x7X
        table[0x70] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x71] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x72] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x73] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x74] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x75] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x76] = Some(Instruction {
            in_type: InType::IN_HALT,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x77] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x78] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x79] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x7A] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x7B] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x7C] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x7D] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x7E] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x7F] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0x8X
        table[0x80] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x81] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x82] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x83] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x84] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x85] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x86] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x87] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x88] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x89] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x8A] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x8B] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x8C] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x8D] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x8E] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x8F] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0x9X
        table[0x90] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x91] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x92] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x93] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x94] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x95] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x96] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x97] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x98] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x99] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x9A] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x9B] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x9C] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x9D] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x9E] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0x9F] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0xA0 to 0xAF: AND Instructions
        table[0xA0] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xA1] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xA2] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xA3] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xA4] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xA5] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xA6] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xA7] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0xA8 to 0xAF: XOR Instructions
        table[0xA8] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xA9] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xAA] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xAB] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xAC] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xAD] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xAE] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xAF] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0xB0 to 0xBF: OR Instructions
        table[0xB0] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xB1] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xB2] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xB3] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xB4] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xB5] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xB6] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xB7] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        // 0xB8 to 0xBF: CP Instructions
        table[0xB8] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_B,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xB9] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xBA] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_D,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xBB] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_E,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xBC] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_H,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xBD] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_L,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xBE] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0,
        });
        table[0xBF] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0,
        });

        //0xCX
        table[0xC0] = Some(Instruction {
            in_type: InType::IN_RET,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NZ,
            param: 0, // No additional parameter
        });
        table[0xC1] = Some(Instruction {
            in_type: InType::IN_POP,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_BC,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xC2] = Some(Instruction {
            in_type: InType::IN_JP,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NZ,
            param: 0, // No additional parameter
        });
        table[0xC3] = Some(Instruction {
            in_type: InType::IN_JP,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xC4] = Some(Instruction {
            in_type: InType::IN_CALL,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NZ,
            param: 0, // No additional parameter
        });
        table[0xC5] = Some(Instruction {
            in_type: InType::IN_PUSH,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_BC,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xC6] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_A8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xC7] = Some(Instruction {
            in_type: InType::IN_RST,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0x00,
        });
        table[0xC8] = Some(Instruction {
            in_type: InType::IN_RET,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_Z,
            param: 0, // No additional parameter
        });
        table[0xC9] = Some(Instruction {
            in_type: InType::IN_RET,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xCA] = Some(Instruction {
            in_type: InType::IN_JP,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_Z,
            param: 0, // No additional parameter
        });
        table[0xCB] = Some(Instruction {
            in_type: InType::IN_CB,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xCC] = Some(Instruction {
            in_type: InType::IN_CALL,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_Z,
            param: 0, // No additional parameter
        });
        table[0xCD] = Some(Instruction {
            in_type: InType::IN_CALL,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xCE] = Some(Instruction {
            in_type: InType::IN_ADC,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xCF] = Some(Instruction {
            in_type: InType::IN_RST,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0x08,
        });

        //0xDX
        table[0xD0] = Some(Instruction {
            in_type: InType::IN_RET,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NC,
            param: 0, // No additional parameter
        });
        table[0xD1] = Some(Instruction {
            in_type: InType::IN_POP,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_DE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xD2] = Some(Instruction {
            in_type: InType::IN_JP,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NC,
            param: 0, // No additional parameter
        });
        table[0xD4] = Some(Instruction {
            in_type: InType::IN_CALL,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NC,
            param: 0, // No additional parameter
        });
        table[0xD5] = Some(Instruction {
            in_type: InType::IN_PUSH,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_DE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xD6] = Some(Instruction {
            in_type: InType::IN_SUB,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xD7] = Some(Instruction {
            in_type: InType::IN_RST,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0x10,
        });
        table[0xD8] = Some(Instruction {
            in_type: InType::IN_RET,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_C,
            param: 0, // No additional parameter
        });
        table[0xD9] = Some(Instruction {
            in_type: InType::IN_RETI,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xDA] = Some(Instruction {
            in_type: InType::IN_JP,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_C,
            param: 0, // No additional parameter
        });
        table[0xDC] = Some(Instruction {
            in_type: InType::IN_CALL,
            mode: AddrMode::AM_D16,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_C,
            param: 0, // No additional parameter
        });
        table[0xDE] = Some(Instruction {
            in_type: InType::IN_SBC,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xDF] = Some(Instruction {
            in_type: InType::IN_RST,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0x18,
        });

        //0xEX
        table[0xE0] = Some(Instruction {
            in_type: InType::IN_LDH,
            mode: AddrMode::AM_A8_R,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xE1] = Some(Instruction {
            in_type: InType::IN_POP,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xE2] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_MR_R,
            reg_1: RegType::RT_C,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xE5] = Some(Instruction {
            in_type: InType::IN_PUSH,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xE6] = Some(Instruction {
            in_type: InType::IN_AND,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xE7] = Some(Instruction {
            in_type: InType::IN_RST,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0x20,
        });
        table[0xE8] = Some(Instruction {
            in_type: InType::IN_ADD,
            mode: AddrMode::AM_R_D8,
            reg_1: RegType::RT_SP,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xE9] = Some(Instruction {
            in_type: InType::IN_JP,
            mode: AddrMode::AM_MR,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xEA] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_A16_R,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_A,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xEE] = Some(Instruction {
            in_type: InType::IN_XOR,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xEF] = Some(Instruction {
            in_type: InType::IN_RST,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0x28,
        });

        //0xFX
        table[0xF0] = Some(Instruction {
            in_type: InType::IN_LDH,
            mode: AddrMode::AM_R_A8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xF1] = Some(Instruction {
            in_type: InType::IN_POP,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_AF,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xF2] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_MR,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_C,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xF3] = Some(Instruction {
            in_type: InType::IN_DI,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xF5] = Some(Instruction {
            in_type: InType::IN_PUSH,
            mode: AddrMode::AM_R,
            reg_1: RegType::RT_AF,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xF6] = Some(Instruction {
            in_type: InType::IN_OR,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xF7] = Some(Instruction {
            in_type: InType::IN_RST,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0x30,
        });
        table[0xF8] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_HL_SPR,
            reg_1: RegType::RT_HL,
            reg_2: RegType::RT_SP,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xF9] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_R,
            reg_1: RegType::RT_SP,
            reg_2: RegType::RT_HL,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xFA] = Some(Instruction {
            in_type: InType::IN_LD,
            mode: AddrMode::AM_R_A16,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xFB] = Some(Instruction {
            in_type: InType::IN_EI,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xFE] = Some(Instruction {
            in_type: InType::IN_CP,
            mode: AddrMode::AM_D8,
            reg_1: RegType::RT_A,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0, // No additional parameter
        });
        table[0xFF] = Some(Instruction {
            in_type: InType::IN_RST,
            mode: AddrMode::AM_IMP,
            reg_1: RegType::RT_NONE,
            reg_2: RegType::RT_NONE,
            cond: CondType::CT_NONE,
            param: 0x38,
        });
        table
    };

    const NP_INSTRUCTIONS: [Option<Instruction>; 256] = {
        let mut table: [Option<Instruction>; 256] = [None; 256];
        
    }
}
