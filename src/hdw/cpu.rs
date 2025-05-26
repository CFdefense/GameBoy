use crate::hdw::bus::Bus;
use crate::hdw::cpu_ops::*;
use crate::hdw::instructions::*;
use crate::hdw::interrupts::*;
use crate::hdw::registers::*;
use crate::hdw::timer::Timer;
use core::panic;

use std::sync::{Arc, Mutex};
use crate::hdw::emu::EmuContext;

use super::cpu_util::{print_step_info, log_cpu_state};
use super::debug;
use super::emu::emu_cycles;

// Our CPU to Call and Control
pub struct CPU {
    pub registers: Registers,
    pub timer: Timer,
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
    pub fn new(new_bus: Bus, mut new_timer: Timer) -> Self {
        new_timer.div = 0xABCC;
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
            timer: new_timer, // Maybe need to set div to 0xABCC
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
    pub fn step(&mut self, ctx: Arc<Mutex<EmuContext>>) -> bool {
        if self.enabling_ime {
            self.master_enabled = true;
            self.enabling_ime = false; // Clear the flag once IME is enabled
        }

        if !self.is_halted {
            self.fetch();
            self.decode();
            
            print_step_info(self, &ctx, false);
            log_cpu_state(self, &ctx, false);
            debug::dbg_update(&mut self.bus);
            debug::dbg_print();

            let instruction_to_execute = self.curr_instruction.take();

            if let Some(instruction) = instruction_to_execute {
                self.execute(instruction); // Execute might modify PC and flagg

            } else {
                panic!("Decode Error: No Instruction")
            }
        } else {
            // is halted
            emu_cycles(self, 1);

            if (self.int_flags & self.ie_register) != 0 {
                self.is_halted = false;
            }
        }

        if self.master_enabled {
            cpu_handle_interrupts(self);
        }

        true
    }

    // Function to fetch next opcode
    fn fetch(&mut self) {
        self.curr_opcode = self.bus.read_byte(None, self.pc);
        emu_cycles(self, 1);
    }

    // Function to decode current opcode
    fn decode(&mut self) {
        // Try to decode curr opcode
        self.curr_instruction =
            Instruction::decode_from_opcode(self.curr_opcode, self.pc, self);

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
                self.registers.f.carry = true;     // C = 1
                self.registers.f.subtract = false; // N = 0
                self.registers.f.half_carry = false; // H = 0
                // Z flag is not affected
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::CPL => {
                op_cpl(self);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::CCF => {
                self.registers.f.carry = !self.registers.f.carry; // C = !C
                self.registers.f.subtract = false;             // N = 0
                self.registers.f.half_carry = false;             // H = 0
                // Z flag is not affected
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
                if !op_ret(self, target) {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            Instruction::RETI => {
                op_reti(self);
            }
            Instruction::POP(target) => {
                op_pop(self, target);
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::JP(target) => {
                if !op_jp(self, target) {
                    self.pc = self.pc.wrapping_add(3);
                }
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
                self.enabling_ime = false; // DI also cancels a pending EI
                self.pc = self.pc.wrapping_add(1);
            }
            Instruction::EI => {
                // EI enables interrupts AFTER the instruction FOLLOWING EI.
                // So, we set a flag to enable IME on the next cycle.
                self.enabling_ime = true; 
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
    
    pub fn cpu_request_interrupt(&mut self, interrupt: Interrupts) {
        self.int_flags |= interrupt as u8;
    }
}
