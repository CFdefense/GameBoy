use crate::hdw::emu::emu_cycles;
use crate::hdw::bus::Bus;
use crate::hdw::cpu_ops::*;
use crate::hdw::instructions::*;
use crate::hdw::interrupts::*;
use crate::hdw::registers::*;
use core::panic;
use regex::Regex;

use std::thread;
use std::time::Duration;

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
    pub fn step(&mut self) -> bool {
        if !self.is_halted {
            // fetch next opcode from cartridge
            self.fetch();

            // Decode current opcode
            self.decode();

            // print information
            // Convert `curr_instruction` to a string
            let instruction_output = format!("{:#?}", self.curr_instruction);

            // Define a regex to capture the instruction name within `Some(...)`
            let re = Regex::new(r"Some\(\s*([A-Z]+)").unwrap();

            // Use regex to capture the instruction name
            let instruction_name = if let Some(cap) = re.captures(&instruction_output) {
                cap.get(1).map_or("Unknown", |m| m.as_str())
            } else {
                "Unknown"
            };

            // Print information, including the extracted instruction name
            print!(
                "\n{:04X}:\t {}\t({:02X} {:02X} {:02X}) A: {:02X} F: {}{}{}{} BC: {:04X} DE: {:04X} HL: {:04X}",
                self.pc,
                instruction_name,
                self.curr_opcode,
                self.bus.read_byte(None, self.pc + 1),
                self.bus.read_byte(None, self.pc + 2),
                self.registers.a,
                if self.registers.f.zero { 'Z' } else { '-' },
                if self.registers.f.subtract { 'N' } else { '-' },
                if self.registers.f.half_carry { 'H' } else { '-' },
                if self.registers.f.carry { 'C' } else { '-' }, 
                self.registers.get_bc(),
                self.registers.get_de(),
                self.registers.get_hl(),
            );

            // Execute the current instruction if it exists and reset it to none
            if let Some(instruction) = self.curr_instruction.take() {
                // Execute the current instruction
                let next_pc = self.execute(instruction);

                // Increment pc to returned pc
                self.pc = next_pc;
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
    fn execute(&mut self, instruction: Instruction) -> u16 {
        // return while halted
        if self.is_halted {
            return self.pc;
        }

        match instruction {
            Instruction::NOP => {
                // Do nothing -> increment pc
                self.pc.wrapping_add(1)
            }
            Instruction::STOP => {
                panic!("STOP");
            }
            Instruction::RLCA => {
                // Perform Operation & Implicit Return
                op_rlca(self)
            }
            Instruction::RRCA => {
                // Perform Operation & Implicit Return
                op_rrca(self)
            }
            Instruction::RLA => {
                // Perform Operation & Implicit Return
                op_rla(self)
            }
            Instruction::RRA => {
                // Perform Operation & Implicit Return
                op_rra(self)
            }
            Instruction::DAA => {
                // Perform Operation & Implicit Return
                op_daa(self)
            }
            Instruction::SCF => {
                self.registers.f.carry = true;
                self.pc.wrapping_add(1)
            }
            Instruction::CPL => {
                // Perform Operation & Implicit Return
                op_cpl(self)
            }
            Instruction::CCF => {
                self.registers.f.carry = !self.registers.f.carry;
                self.pc.wrapping_add(1)
            }
            Instruction::JR(target) => {
                // Perform Operation & Implicit Return
                let next_pc = op_jr(self, target);
                next_pc
            }
            Instruction::INC(target) => {
                // Perform Operation & Implicit Return
                op_inc(self, target)
            }
            Instruction::DEC(target) => {
                // Perform Operation & Implicit Return
                op_dec(self, target)
            }
            Instruction::LD(target) => {
                // Perform Operation & Implicit Return
                op_ld(self, target)
            }
            Instruction::HALT => {
                // Instruction For Halting CPU Cycle
                self.is_halted = true;
                self.pc.wrapping_add(1)
            }
            Instruction::ADD(target) => {
                // Perform Operation & Implicit Return
                op_add(self, target)
            }
            Instruction::ADC(target) => {
                // Perform Operation & Implicit Return
                op_adc(self, target)
            }
            Instruction::SUB(target) => {
                // Perform Operation & Implicit Return
                op_sub(self, target)
            }
            Instruction::SBC(target) => {
                // Perform Operation & Implicit Return
                op_sbc(self, target)
            }
            Instruction::AND(target) => {
                // Perform Operation & Implicit Return
                op_and(self, target)
            }
            Instruction::XOR(target) => {
                // Perform Operation & Implicit Return
                op_xor(self, target)
            }
            Instruction::OR(target) => {
                // Perform Operation & Implicit Return
                op_or(self, target)
            }
            Instruction::CP(target) => {
                // Perform Operation & Implicit Return
                op_cp(self, target)
            }
            Instruction::RET(target) => {
                // Perform Operation & Implicit Return
                let next_pc = op_ret(self, target);
                thread::sleep(Duration::from_secs(20));
                next_pc
            }
            Instruction::RETI => {
                // Perform Operation & Implicit Return
                let next_pc = op_reti(self);
                thread::sleep(Duration::from_secs(20));
                next_pc
            }
            Instruction::POP(target) => {
                // Perform Operation & Implicit Return
                op_pop(self, target)
            }
            Instruction::JP(target) => {
                // Perform Operation & Implicit Return
                op_jp(self, target)
            }
            Instruction::CALL(target) => {
                // Perform Operation & Implicit Return
                let next_pc = op_call(self, target);
                //thread::sleep(Duration::from_secs(20));
                next_pc
            }
            Instruction::PUSH(target) => {
                // Perform Operation & Implicit Return
                op_push(self, target)
            }
            Instruction::RST(target) => {
                // Perform Operation & Implicit Return
                let next_pc = op_rst(self, target);
                thread::sleep(Duration::from_secs(20));
                next_pc
            }
            Instruction::DI => {
                self.master_enabled = false;
                self.pc.wrapping_add(1) // unsure what to return here leaving this for now
            }
            Instruction::EI => {
                self.master_enabled = true;
                self.pc.wrapping_add(1) // unsure what to return here leavint his for now
            }

            // PREFIXED INSTRUCTIONS
            Instruction::RLC(target) => {
                // Perform Operation & Implicit Return
                op_rlc(self, target)
            }
            Instruction::RRC(target) => {
                // Perform Operation & Implicit Return
                op_rrc(self, target)
            }
            Instruction::RL(target) => {
                // Perform Operation & Implicit Return
                op_rl(self, target)
            }
            Instruction::RR(target) => {
                // Perform Operation & Implicit Return
                op_rr(self, target)
            }
            Instruction::SLA(target) => {
                // Perform Operation & Implicit Return
                op_sla(self, target)
            }
            Instruction::SRA(target) => {
                // Perform Operation & Implicit Return
                op_sra(self, target)
            }
            Instruction::SWAP(target) => {
                // Perform Operation & Implicit Return
                op_swap(self, target)
            }
            Instruction::SRL(target) => {
                // Perform Operation & Implicit Return
                op_srl(self, target)
            }
            Instruction::BIT(target) => {
                // Perform Operation & Implicit Return
                op_bit(self, target)
            }
            Instruction::RES(target) => {
                // Perform Operation & Implicit Return
                op_res(self, target)
            }
            Instruction::SET(target) => {
                // Perform Operation & Implicit Return
                op_set(self, target)
            }
        }
    }

    // IE Getter
    pub fn get_ie_register(&self) -> u8 {
        self.ie_register
    }

    // IE Setter
    pub fn set_ie_register(&mut self, value: u8) {
        self.ie_register = value;
    }
    // CPU ENDS HERE
}
