use crate::hdw::bus::Bus;
use crate::hdw::cpu_ops::*;
use crate::hdw::emu::emu_cycles;
use crate::hdw::instructions::*;
use crate::hdw::interrupts::*;
use crate::hdw::registers::*;
use core::panic;

use super::cpu_util::print_step_info;
use super::debug;

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
                b: 0x00,
                c: 0x13,
                d: 0x00,
                e: 0xD8,
                f: FlagsRegister {
                    zero: true,
                    subtract: false,
                    half_carry: true,
                    carry: true,
                },
                h: 0x01,
                l: 0x4D,
            },
            pc: 0x0100,
            sp: 0xFFFE, 
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
            if let Some(instruction) = self.curr_instruction.take() {
                debug::dbg_update(&mut self.bus);
                debug::dbg_print();
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
            Instruction::NOP => {
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::STOP => {
                panic!("STOP");
            }
            Instruction::RLCA => {
                op_rlca(self);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RRCA => {
                op_rrca(self);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RLA => {
                op_rla(self);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RRA => {
                op_rra(self);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::DAA => {
                op_daa(self);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::SCF => {
                self.registers.f.carry = true;
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::CPL => {
                op_cpl(self);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::CCF => {
                self.registers.f.carry = !self.registers.f.carry;
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::JR(target) => {
                self.pc = op_jr(self, target);
                self.pc = self.pc.wrapping_add(2); // skip operand of JR
            }
            Instruction::INC(target) => {
                op_inc(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::DEC(target) => {
                op_dec(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::LD(target) => {
                op_ld(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::HALT => {
                self.is_halted = true;
            }
            Instruction::ADD(target) => {
                op_add(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::ADC(target) => {
                op_adc(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::SUB(target) => {
                op_sub(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::SBC(target) => {
                op_sbc(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::AND(target) => {
                let is_d8 = matches!(target, OPTarget::D8);
                op_and(self, target);
                if is_d8 {
                    self.pc = self.pc.wrapping_add(2);
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            Instruction::XOR(target) => {
                op_xor(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::OR(target) => {
                op_or(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::CP(target) => {
                op_cp(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RET(target) => {
                op_ret(self, target);
            }
            Instruction::RETI => {
                op_reti(self);
            }
            Instruction::POP(target) => {
                op_pop(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::JP(target) => {
                op_jp(self, target);
            }
            Instruction::CALL(target) => {
                op_call(self, target);
            }
            Instruction::PUSH(target) => {
                op_push(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::RST(target) => {
                op_rst(self, target);
            }
            Instruction::DI => {
                self.master_enabled = false;
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::EI => {
                self.master_enabled = true;
                self.pc = self.pc.wrapping_add(1);
            }

            // PREFIXED INSTRUCTIONS: INC PC BY 1 AFTER INSTRUCTION DUE TO CB PREFIX
            Instruction::RLC(target) => {
                op_rlc(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::RRC(target) => {
                op_rrc(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::RL(target) => {
                op_rl(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::RR(target) => {
                op_rr(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::SLA(target) => {
                op_sla(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::SRA(target) => {
                op_sra(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::SWAP(target) => {
                op_swap(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::SRL(target) => {
                op_srl(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::BIT(target) => {
                op_bit(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::RES(target) => {
                op_res(self, target);
                self.pc = self.pc.wrapping_add(2);
            }
            Instruction::SET(target) => {
                op_set(self, target);
                self.pc = self.pc.wrapping_add(2);
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
