use crate::hdw::bus::Bus;
use crate::hdw::cpu_ops::*;
use crate::hdw::emu::emu_cycles;
use crate::hdw::instructions::*;
use crate::hdw::interrupts::*;
use crate::hdw::registers::*;
use core::panic;

use super::cpu_util::print_step_info;

// Our CPU to Call and Control
pub struct CPU {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub bus: Bus,

    pub curr_opcode: u8,
    pub curr_instruction: Option<Instruction>,

    pub is_halted: bool,
    pub is_stepping: bool,

    pub ie_register: u8,
    pub int_flags: u8,
    pub enabling_ime: bool,
    pub master_enabled: bool,
}
impl CPU {
    // Contructor
    pub fn new(new_bus: Bus) -> Self {
        CPU {
            registers: Registers {
                a: 0x01,
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
            bus: new_bus,

            curr_opcode: 0,
            curr_instruction: None,

            is_halted: false,
            is_stepping: true,

            int_flags: 0,
            ie_register: 0,
            enabling_ime: false,
            master_enabled: false,
        }
    }

    // Function to 'step' through instructions
    pub fn step(&mut self, ticks: u64) -> bool {

        // Check if CPU is halted
        if !self.is_halted {
            self.fetch(); // fetch next opcode from cartridge
            self.decode(); // Decode current opcode
            print_step_info(self, ticks); // print step info

            // Execute the current instruction if it exists and reset it to none
            if let Some(instruction) = self.curr_instruction.take()  {
                self.execute(instruction); // Execute the current instruction
            } else {
                panic!("Decode Error: No Instruction")
            }
        } else {
            // is halted
            emu_cycles(1);

            if self.int_flags != 0 {
                self.is_halted = false;
            }
        }

        if self.master_enabled {
            cpu_handle_interrupts(self);
            self.enabling_ime = false;
        }

        if self.enabling_ime {
            self.master_enabled = true;
        }
        true
    }

    // Function to fetch next opcode
    fn fetch(&mut self) {
        // Increment PC
        self.pc = self.pc.wrapping_add(1);

        // Get Next Opcode
        self.curr_opcode = self.bus.read_byte(None, self.pc);
    }

    // Function to decode current opcode
    fn decode(&mut self) {
        // Try to decode curr opcode
        self.curr_instruction =
            Instruction::decode_from_opcode(self.curr_opcode, &self.bus, self.pc);

        // Error handling
        if self.curr_instruction.is_none() {
            panic!(
                "Unable to Read Opcode 0x{:02X}, instruction: {:#?}",
                self.curr_opcode, self.curr_instruction
            );
        }
    }

    // Function to execute an opcode by matching Instruction type and target then calling its method
    fn execute(&mut self, instruction: Instruction) {
        /* maybe dont need -> return while halted
        if self.is_halted {
            return self.pc;
        }
        */
        match instruction {
            Instruction::NOP => {/* Do Nothing */}
            Instruction::STOP => {panic!("STOP");}
            Instruction::RLCA => {op_rlca(self)}
            Instruction::RRCA => {op_rrca(self)}
            Instruction::RLA => {op_rla(self)}
            Instruction::RRA => {op_rra(self)}
            Instruction::DAA => {op_daa(self)}
            Instruction::SCF => {self.registers.f.carry = true;}
            Instruction::CPL => {op_cpl(self)}
            Instruction::CCF => {self.registers.f.carry = !self.registers.f.carry;}
            Instruction::JR(target) => {self.pc = op_jr(self, target);}
            Instruction::INC(target) => {op_inc(self, target)}
            Instruction::DEC(target) => {op_dec(self, target)}
            Instruction::LD(target) => {op_ld(self, target)}
            Instruction::HALT => {self.is_halted = true;}
            Instruction::ADD(target) => {op_add(self, target)}
            Instruction::ADC(target) => {op_adc(self, target)}
            Instruction::SUB(target) => {op_sub(self, target)}
            Instruction::SBC(target) => {op_sbc(self, target)}
            Instruction::AND(target) => {op_and(self, target)}
            Instruction::XOR(target) => {op_xor(self, target)}
            Instruction::OR(target) => {op_or(self, target)}
            Instruction::CP(target) => {op_cp(self, target)}
            Instruction::RET(target) => {op_ret(self, target);}
            Instruction::RETI => {op_reti(self);}
            Instruction::POP(target) => {op_pop(self, target);}
            Instruction::JP(target) => {op_jp(self, target);}
            Instruction::CALL(target) => {op_call(self, target);}
            Instruction::PUSH(target) => {op_push(self, target);}
            Instruction::RST(target) => {op_rst(self, target);}
            Instruction::DI => {self.master_enabled = false;}
            Instruction::EI => {self.master_enabled = true;}

            // PREFIXED INSTRUCTIONS: INC PC BY 1 AFTER INSTRUCTION DUE TO CB PREFIX
            Instruction::RLC(target) => {
                op_rlc(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RRC(target) => {
                op_rrc(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RL(target) => {
                op_rl(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RR(target) => {
                op_rr(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::SLA(target) => {
                op_sla(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::SRA(target) => {
                op_sra(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::SWAP(target) => {
                op_swap(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::SRL(target) => {
                op_srl(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::BIT(target) => {
                op_bit(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RES(target) => {
                op_res(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::SET(target) => {
                op_set(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
        }
    }

    // IE Getter
    pub fn get_ie_register(&self) -> u8 {
        self.ie_register
    }

    // IE Setter                // Perform Operation & Implicit Return
    pub fn set_ie_register(&mut self, value: u8) {
        self.ie_register = value;
    }
    // CPU ENDS HERE
}
