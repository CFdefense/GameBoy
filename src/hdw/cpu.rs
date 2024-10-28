use crate::hdw::bus::Bus;
use crate::hdw::instructions::*;
use crate::hdw::registers::*;
use crate::hdw::cpu_util::*;
use crate::hdw::cpu_ops::*;
use core::panic;
use regex::Regex;

// Our CPU to Call and Control
pub struct CPU {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub bus: Bus,
    pub is_halted: bool,
    pub master_enabled: bool,
    pub curr_opcode: u8,
    pub curr_instruction: Option<Instruction>,
    pub ie_register: u8,
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
            is_halted: false,
            master_enabled: false,

            curr_opcode: 0,
            curr_instruction: None,
            ie_register: 0,
        }
    }

    // Function to 'step' through instructions
    pub fn step(&mut self) {
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
            "{:04X}:\t {} ({:02X} {:02X} {:02X})\nA:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} AF:{:04X} BC:{:04X} DE:{:04X} HL:{:04X}\nZ:{:02X} N:{:02X} H:{:02X} C:{:02X} \n\n",
            self.pc,
            instruction_name,
            self.curr_opcode,
            self.bus.read_byte(None, self.pc + 1),
            self.bus.read_byte(None, self.pc + 2),
            self.registers.a,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.h,
            self.registers.l,
            self.registers.get_af(),
            self.registers.get_bc(),
            self.registers.get_de(),
            self.registers.get_hl(),
            self.registers.f.zero as u8,
            self.registers.f.subtract as u8,    
            self.registers.f.half_carry as u8,
            self.registers.f.carry as u8
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
    }

    // Function to fetch next opcode
    fn fetch(&mut self) {
        self.curr_opcode = self.bus.read_byte(None, self.pc);
    }

    // Function to decode current opcode
    fn decode(&mut self) {
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
                // Store the original bit 7 to set the Carry flag and bit 0
                let bit_7 = (self.registers.a >> 7) & 1;

                // Rotate left: shift left by 1 and add bit 7 to bit 0
                self.registers.a = (self.registers.a << 1) | bit_7;

                // Set Carry Flag to the value of bit 7
                self.registers.f.carry = bit_7 != 0;

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::RRCA => {
                // Store the original bit 0 to set the carry flag and bit 7
                let bit_0 = self.registers.a & 1;

                // Rotate right: shift right by 1 and add bit 0 to bit 7
                self.registers.a = (self.registers.a >> 1) | (bit_0 << 7);

                // Set Carry Flag to the value of bit 0
                self.registers.f.carry = bit_0 != 0;

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::RLA => {
                // Store the original bit 7 to set the carry flag
                let bit_7 = (self.registers.a & 0x80) >> 7;

                // Rotate left: shift left by 1 and add carry to bit 0
                self.registers.a = (self.registers.a << 1) | (self.registers.f.carry as u8);

                // Set Carry Flag to the value of bit 7
                self.registers.f.carry = bit_7 != 0;

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::RRA => {
                // Store the original bit 0 to set the carry flag
                let bit_0 = self.registers.a & 1;

                // Rotate right: shift right by 1 and add carry to bit 7
                self.registers.a = (self.registers.a >> 1) | (self.registers.f.carry as u8) << 7;

                // Set Carry Flag to the value of bit 0
                self.registers.f.carry = bit_0 != 0;

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::DAA => {
                let mut adjustment = 0;
                let mut carry = false;

                // If the subtract flag is clear, this is an addition
                if !self.registers.f.subtract {
                    if self.registers.f.half_carry || self.registers.a & 0x0F > 9 {
                        adjustment += 0x06;
                    }
                    if self.registers.f.carry || self.registers.a > 0x99 {
                        adjustment += 0x60;
                        carry = true;
                    }
                } else {
                    // If subtract is set, it's a subtraction
                    if self.registers.f.half_carry {
                        adjustment -= 0x06;
                    }
                    if self.registers.f.carry {
                        adjustment -= 0x60;
                    }
                }

                // Apply the adjustment to the accumulator
                self.registers.a = self.registers.a.wrapping_add(adjustment);

                // Clear H flag and set C flag if carry occurred
                self.registers.f.half_carry = false;
                self.registers.f.carry = carry;

                // Set the zero flag if the result is 0
                self.registers.f.zero = self.registers.a == 0;

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::SCF => {
                self.registers.f.carry = true;
                self.pc.wrapping_add(1)
            }
            Instruction::CPL => {
                // Flip all bits of register A
                self.registers.a = !self.registers.a;

                // Set flags
                self.registers.f.zero = self.registers.a == 0; // might not need
                self.registers.f.subtract = true;
                self.registers.f.half_carry = true;

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::CCF => {
                self.registers.f.carry = !self.registers.f.carry;
                self.pc.wrapping_add(1)
            }
            Instruction::JR(test) => {
                let jump_distance = self.bus.read_byte(None, self.pc + 1) as i8;
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
                self.pc.wrapping_add(1)
            }
            Instruction::INC(target) => {
                match target {
                    // Increment 8-bit registers and Set Flags
                    AllRegisters::A => {
                        self.registers.a = self.registers.a.wrapping_add(1);
                        set_flags_after_inc(self, self.registers.a);
                    }
                    AllRegisters::B => {
                        self.registers.b = self.registers.b.wrapping_add(1);
                        set_flags_after_inc(self,self.registers.b);
                    }
                    AllRegisters::C => {
                        self.registers.c = self.registers.c.wrapping_add(1);
                        set_flags_after_inc(self,self.registers.c);
                    }
                    AllRegisters::D => {
                        self.registers.d = self.registers.d.wrapping_add(1);
                        set_flags_after_inc(self,self.registers.d);
                    }
                    AllRegisters::E => {
                        self.registers.e = self.registers.e.wrapping_add(1);
                        set_flags_after_inc(self,self.registers.e);
                    }
                    AllRegisters::H => {
                        self.registers.h = self.registers.h.wrapping_add(1);
                        set_flags_after_inc(self,self.registers.h);
                    }
                    AllRegisters::L => {
                        self.registers.l = self.registers.l.wrapping_add(1);
                        set_flags_after_inc(self,self.registers.l);
                    }
                    // Increment value at bus location HL
                    AllRegisters::HLMEM => {
                        let hl_addr = self.registers.get_hl();
                        let value = self.bus.read_byte(None, hl_addr).wrapping_add(1);
                        self.bus.write_byte(None, hl_addr, value);
                        set_flags_after_inc(self,value);
                    }
                    // 16-bit register increments (don't need to Set Flags for these)
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
                self.pc.wrapping_add(1)
            }
            Instruction::DEC(target) => {
                match target {
                    // Increment 8-bit registers and Set Flags
                    AllRegisters::A => {
                        let original_value = self.registers.a;
                        self.registers.a = self.registers.a.wrapping_sub(1);
                        set_flags_after_dec(self,self.registers.a, original_value);
                    }
                    AllRegisters::B => {
                        let original_value = self.registers.b;
                        self.registers.b = self.registers.b.wrapping_sub(1);
                        set_flags_after_dec(self,self.registers.b, original_value);
                    }
                    AllRegisters::C => {
                        let original_value = self.registers.c;
                        self.registers.c = self.registers.c.wrapping_sub(1);
                        set_flags_after_dec(self,self.registers.c, original_value);
                    }
                    AllRegisters::D => {
                        let original_value = self.registers.d;
                        self.registers.d = self.registers.d.wrapping_sub(1);
                        set_flags_after_dec(self,self.registers.d, original_value);
                    }
                    AllRegisters::E => {
                        let original_value = self.registers.e;
                        self.registers.e = self.registers.e.wrapping_sub(1);
                        set_flags_after_dec(self,self.registers.e, original_value);
                    }
                    AllRegisters::H => {
                        let original_value = self.registers.h;
                        self.registers.h = self.registers.h.wrapping_sub(1);
                        set_flags_after_dec(self,self.registers.h, original_value);
                    }
                    AllRegisters::L => {
                        let original_value = self.registers.l;
                        self.registers.l = self.registers.l.wrapping_sub(1);
                        set_flags_after_dec(self,self.registers.l, original_value);
                    }
                    // Increment value at bus location HL
                    AllRegisters::HLMEM => {
                        let hl_addr = self.registers.get_hl();
                        let original_value = self.bus.read_byte(None, hl_addr);
                        let value = self.bus.read_byte(None, hl_addr).wrapping_sub(1);
                        self.bus.write_byte(None, hl_addr, value);
                        set_flags_after_dec(self,value, original_value);
                    }
                    // 16-bit register increments (don't need to Set Flags for these)
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
                self.pc.wrapping_add(1)
            }
            Instruction::LD(target) => match target {
                LoadType::RegInReg(target, source) => match target {
                    HLTarget::B => match source {
                        HLTarget::B => {
                            self.registers.b = self.registers.b;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::C => {
                            self.registers.b = self.registers.c;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::D => {
                            self.registers.b = self.registers.d;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::E => {
                            self.registers.b = self.registers.e;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::H => {
                            self.registers.b = self.registers.h;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::L => {
                            self.registers.b = self.registers.l;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::HL => {
                            self.registers.b = self.bus.read_byte(None, self.registers.get_hl());
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::A => {
                            self.registers.b = self.registers.a;
                            self.pc.wrapping_add(1)
                        }
                    },
                    HLTarget::C => match target {
                        HLTarget::B => {
                            self.registers.c = self.registers.b;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::C => {
                            self.registers.c = self.registers.c;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::D => {
                            self.registers.c = self.registers.d;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::E => {
                            self.registers.c = self.registers.e;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::H => {
                            self.registers.c = self.registers.h;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::L => {
                            self.registers.c = self.registers.l;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::HL => {
                            self.registers.c = self.bus.read_byte(None, self.registers.get_hl());
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::A => {
                            self.registers.c = self.registers.a;
                            self.pc.wrapping_add(1)
                        }
                    },
                    HLTarget::D => match target {
                        HLTarget::B => {
                            self.registers.d = self.registers.b;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::C => {
                            self.registers.d = self.registers.c;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::D => {
                            self.registers.d = self.registers.d;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::E => {
                            self.registers.d = self.registers.e;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::H => {
                            self.registers.d = self.registers.h;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::L => {
                            self.registers.d = self.registers.l;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::HL => {
                            self.registers.d = self.bus.read_byte(None, self.registers.get_hl());
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::A => {
                            self.registers.d = self.registers.a;
                            self.pc.wrapping_add(1)
                        }
                    },
                    HLTarget::E => match target {
                        HLTarget::B => {
                            self.registers.e = self.registers.b;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::C => {
                            self.registers.e = self.registers.c;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::D => {
                            self.registers.e = self.registers.d;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::E => {
                            self.registers.e = self.registers.e;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::H => {
                            self.registers.e = self.registers.h;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::L => {
                            self.registers.e = self.registers.l;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::HL => {
                            self.registers.e = self.bus.read_byte(None, self.registers.get_hl());
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::A => {
                            self.registers.e = self.registers.a;
                            self.pc.wrapping_add(1)
                        }
                    },
                    HLTarget::H => match target {
                        HLTarget::B => {
                            self.registers.h = self.registers.b;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::C => {
                            self.registers.h = self.registers.c;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::D => {
                            self.registers.h = self.registers.d;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::E => {
                            self.registers.h = self.registers.e;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::H => {
                            self.registers.h = self.registers.h;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::L => {
                            self.registers.h = self.registers.l;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::HL => {
                            self.registers.h = self.bus.read_byte(None, self.registers.get_hl());
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::A => {
                            self.registers.h = self.registers.a;
                            self.pc.wrapping_add(1)
                        }
                    },
                    HLTarget::L => match target {
                        HLTarget::B => {
                            self.registers.l = self.registers.b;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::C => {
                            self.registers.l = self.registers.c;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::D => {
                            self.registers.l = self.registers.d;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::E => {
                            self.registers.l = self.registers.e;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::H => {
                            self.registers.l = self.registers.h;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::L => {
                            self.registers.l = self.registers.l;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::HL => {
                            self.registers.l = self.bus.read_byte(None, self.registers.get_hl());
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::A => {
                            self.registers.l = self.registers.a;
                            self.pc.wrapping_add(1)
                        }
                    },
                    HLTarget::HL => match target {
                        HLTarget::B => {
                            self.bus
                                .write_byte(None, self.registers.get_hl(), self.registers.b);
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::C => {
                            self.bus
                                .write_byte(None, self.registers.get_hl(), self.registers.c);
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::D => {
                            self.bus
                                .write_byte(None, self.registers.get_hl(), self.registers.d);
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::E => {
                            self.bus
                                .write_byte(None, self.registers.get_hl(), self.registers.e);
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::H => {
                            self.bus
                                .write_byte(None, self.registers.get_hl(), self.registers.h);
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::L => {
                            self.bus
                                .write_byte(None, self.registers.get_hl(), self.registers.l);
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::A => {
                            self.bus
                                .write_byte(None, self.registers.get_hl(), self.registers.a);
                            self.pc.wrapping_add(1)
                        }
                        _ => panic!("Getting LD HL HL Should be HALT"),
                    },
                    HLTarget::A => match target {
                        HLTarget::B => {
                            self.registers.a = self.registers.b;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::C => {
                            self.registers.a = self.registers.c;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::D => {
                            self.registers.a = self.registers.d;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::E => {
                            self.registers.a = self.registers.e;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::H => {
                            self.registers.a = self.registers.h;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::L => {
                            self.registers.a = self.registers.l;
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::HL => {
                            self.registers.a = self.bus.read_byte(None, self.registers.get_hl());
                            self.pc.wrapping_add(1)
                        }
                        HLTarget::A => {
                            self.registers.a = self.registers.a;
                            self.pc.wrapping_add(1)
                        }
                    },
                },
                LoadType::Word(target, source) => {
                    // Read the next two bytes from bus at the current PC
                    let low_byte = self.bus.read_byte(None, self.pc + 1); // Read the low byte
                    let high_byte = self.bus.read_byte(None, self.pc + 2); // Read the high byte

                    // Combine the low and high bytes into a 16-bit value
                    let word_value = ((high_byte as u16) << 8) | (low_byte as u16);

                    match target {
                        LoadWordTarget::BC => match source {
                            LoadWordSource::N16 => {
                                self.registers.set_bc(word_value as u16);
                                self.pc.wrapping_add(3)
                            }
                            _ => panic!("LD WORD BAD MATCH"),
                        },
                        LoadWordTarget::HL => match source {
                            LoadWordSource::N16 => {
                                self.registers.set_hl(word_value as u16);

                                self.pc.wrapping_add(3)
                            }
                            LoadWordSource::SPE8 => {
                                self.registers.set_hl(
                                    ((self.sp as i16).wrapping_add(
                                        (self.bus.read_byte(None, self.pc + 1) as i8) as i16,
                                    )) as u16,
                                );

                                // Set Flags
                                self.registers.f.subtract = false;

                                self.registers.f.half_carry = ((self.sp & 0x0F)
                                    + (self.bus.read_byte(None, self.pc + 1) as u16 & 0x0F))
                                    > 0x0F;

                                self.registers.f.carry = ((self.sp & 0xFF)
                                    + (self.bus.read_byte(None, self.pc + 1) as u16 & 0xFF))
                                    > 0xFF;

                                self.pc.wrapping_add(2)
                            }
                            _ => panic!("LD WORD BAD MATCH"),
                        },
                        LoadWordTarget::DE => match source {
                            LoadWordSource::N16 => {
                                self.registers
                                    .set_de(self.bus.read_byte(None, word_value) as u16);
                                self.pc.wrapping_add(3)
                            }
                            _ => panic!("LD WORD BAD MATCH"),
                        },
                        LoadWordTarget::N16 => match source {
                            LoadWordSource::SP => {
                                self.bus
                                    .write_byte(None, word_value, (self.sp & 0x00FF) as u8);
                                self.bus
                                    .write_byte(None, word_value + 1, (self.sp >> 8) as u8);
                                self.pc.wrapping_add(3)
                            }
                            _ => panic!("LD WORD BAD MATCH"),
                        },
                        LoadWordTarget::SP => match source {
                            LoadWordSource::HL => {
                                self.registers.set_hl(self.sp);
                                self.pc.wrapping_add(1)
                            }
                            LoadWordSource::N16 => {
                                self.sp = word_value;
                                self.pc.wrapping_add(3)
                            }
                            _ => panic!("LD WORD BAD MATCH"),
                        },
                    }
                }
                LoadType::AStoreInN16(target) => match target {
                    LoadN16::BC => {
                        self.bus
                            .write_byte(None, self.registers.get_bc(), self.registers.a);
                        self.pc.wrapping_add(1)
                    }
                    LoadN16::DE => {
                        self.bus
                            .write_byte(None, self.registers.get_de(), self.registers.a);
                        self.pc.wrapping_add(1)
                    }
                    LoadN16::HLDEC => {
                        self.bus
                            .write_byte(None, self.registers.get_hl(), self.registers.a);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_sub(1));
                        self.pc.wrapping_add(1)
                    }
                    LoadN16::HLINC => {
                        self.bus
                            .write_byte(None, self.registers.get_hl(), self.registers.a);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_add(1));
                        self.pc.wrapping_add(1)
                    }
                },
                LoadType::N16StoreInA(source) => match source {
                    LoadN16::BC => {
                        self.registers.a = self.bus.read_byte(None, self.registers.get_bc());
                        self.pc.wrapping_add(1)
                    }
                    LoadN16::DE => {
                        self.registers.a = self.bus.read_byte(None, self.registers.get_de());
                        self.pc.wrapping_add(1)
                    }
                    LoadN16::HLDEC => {
                        self.registers.a = self.bus.read_byte(None, self.registers.get_hl());
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_sub(1));
                        self.pc.wrapping_add(1)
                    }
                    LoadN16::HLINC => {
                        self.registers.a = self.bus.read_byte(None, self.registers.get_hl());
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_add(1));
                        self.pc.wrapping_add(1)
                    }
                },
                LoadType::D8StoreInReg(target) => match target {
                    HLTarget::B => {
                        self.registers.b = self.bus.read_byte(None, self.pc + 1);
                        self.pc.wrapping_add(2)
                    }
                    HLTarget::C => {
                        self.registers.c = self.bus.read_byte(None, self.pc + 1);
                        self.pc.wrapping_add(2)
                    }
                    HLTarget::D => {
                        self.registers.d = self.bus.read_byte(None, self.pc + 1);
                        self.pc.wrapping_add(2)
                    }
                    HLTarget::E => {
                        self.registers.e = self.bus.read_byte(None, self.pc + 1);
                        self.pc.wrapping_add(2)
                    }
                    HLTarget::H => {
                        self.registers.h = self.bus.read_byte(None, self.pc + 1);
                        self.pc.wrapping_add(2)
                    }
                    HLTarget::L => {
                        self.registers.l = self.bus.read_byte(None, self.pc + 1);
                        self.pc.wrapping_add(2)
                    }
                    HLTarget::HL => {
                        self.bus.write_byte(
                            None,
                            self.registers.get_hl(),
                            self.bus.read_byte(None, self.pc + 1),
                        );
                        self.pc.wrapping_add(2)
                    }
                    HLTarget::A => {
                        self.registers.a = self.bus.read_byte(None, self.pc + 1);
                        self.pc.wrapping_add(2)
                    }
                },
                LoadType::AWithA8(target) => match target {
                    LoadA8Target::A => {
                        // First read all values we need
                        let address = 0xFF00 + self.bus.read_byte(None, self.pc + 1) as u16;

                        // Then read the value at the calculated address
                        // We create a temporary mutable reference to self for the read_byte call
                        let value = {
                            let cpu_ref = self as *mut CPU;
                            // SAFETY: We're only creating a temporary reference and not modifying any state
                            // The CPU reference is valid for the duration of this scope
                            // We ensure no other mutable references exist during this time
                            self.bus.read_byte(Some(unsafe { &mut *cpu_ref }), address)
                        };

                        // Finally update register and return
                        self.registers.a = value;
                        self.pc.wrapping_add(2)
                    }
                    LoadA8Target::A8 => {
                        // First read all values we need
                        let address = 0xFF00 + self.bus.read_byte(None, self.pc + 1) as u16;
                        let value = self.registers.a;

                        // Create a temporary mutable reference for the write operation
                        {
                            let cpu_ref = self as *mut CPU;
                            // SAFETY: We're only creating a temporary reference and not modifying any state
                            // The CPU reference is valid for the duration of this scope
                            // We ensure no other mutable references exist during this time
                            self.bus
                                .write_byte(Some(unsafe { &mut *cpu_ref }), address, value);
                        }

                        // Return the new PC
                        self.pc.wrapping_add(2)
                    }
                },
                LoadType::AWithA16(target) => {
                    let low_byte = self.bus.read_byte(None, self.pc + 1); // Read the low byte
                    let high_byte = self.bus.read_byte(None, self.pc + 2); // Read the high byte

                    // Combine the low and high bytes into a 16-bit value
                    let address = ((high_byte as u16) << 8) | (low_byte as u16);

                    match target {
                        LoadA16Target::A => {
                            self.registers.a = self.bus.read_byte(None, address);
                            self.pc.wrapping_add(3)
                        }
                        LoadA16Target::A16 => {
                            self.bus.write_byte(None, address, self.registers.a);
                            self.pc.wrapping_add(3)
                        }
                    }
                }
                LoadType::AWithAC(target) => match target {
                    LoadACTarget::A => {
                        self.bus.write_byte(
                            None,
                            0xFF00 + self.registers.c as u16,
                            self.registers.a,
                        );
                        self.pc.wrapping_add(2)
                    }
                    LoadACTarget::C => {
                        self.registers.a =
                            self.bus.read_byte(None, 0xFF00 + self.registers.c as u16);
                        self.pc.wrapping_add(2)
                    }
                },
            },
            Instruction::HALT => {
                // Instruction For Halting CPU Cycle
                self.is_halted = true;
                self.pc.wrapping_add(1)
            }
            Instruction::ADD(target) => match target {
                OPType::LoadA(target) => {
                    let reg_target = match_hl(self, target);
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

                    self.pc.wrapping_add(1)
                }
                OPType::LoadHL(target) => {
                    let reg_target = match_n16(self, target);
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

                    self.pc.wrapping_add(1)
                }
                OPType::LoadSP => {
                    let immediate_operand: i8 = self.bus.read_byte(None, self.pc + 1) as i8;

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

                    self.pc.wrapping_add(2)
                }
                OPType::LoadD8 => {
                    let immediate_operand: u8 = self.bus.read_byte(None, self.pc + 1);
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

                    self.pc.wrapping_add(2)
                }
            },
            Instruction::ADC(target) => match target {
                OPTarget::B => {
                    // Store Original Value
                    let original_value = self.registers.a;
                    // ADC
                    self.registers.a = self.registers.b.wrapping_add(self.registers.f.carry as u8);

                    // Set Flags
                    set_flags_after_adc(self,self.registers.a, original_value, self.registers.b);
                    self.pc.wrapping_add(1)
                }
                OPTarget::C => {
                    // Store Original Value
                    let original_value = self.registers.a;
                    // ADC
                    self.registers.a = self.registers.c.wrapping_add(self.registers.f.carry as u8);
                    // Set Flags
                    set_flags_after_adc(self,self.registers.a, original_value, self.registers.c);
                    self.pc.wrapping_add(1)
                }
                OPTarget::E => {
                    // Store Original Value
                    let original_value = self.registers.a;
                    // ADC
                    self.registers.a = self.registers.e.wrapping_add(self.registers.f.carry as u8);

                    // Set Flags
                    set_flags_after_adc(self,self.registers.a, original_value, self.registers.e);
                    self.pc.wrapping_add(1)
                }
                OPTarget::D => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self.registers.d.wrapping_add(self.registers.f.carry as u8);

                    // Set Flags
                    set_flags_after_adc(self,self.registers.a, original_value, self.registers.d);
                    self.pc.wrapping_add(1)
                }
                OPTarget::H => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self.registers.h.wrapping_add(self.registers.f.carry as u8);

                    // Set Flags
                    set_flags_after_adc(self,self.registers.a, original_value, self.registers.h);
                    self.pc.wrapping_add(1)
                }
                OPTarget::L => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self.registers.l.wrapping_add(self.registers.f.carry as u8);

                    // Set Flags
                    set_flags_after_adc(self,self.registers.a, original_value, self.registers.l);
                    self.pc.wrapping_add(1)
                }
                OPTarget::HL => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self
                        .bus
                        .read_byte(None, self.registers.get_hl())
                        .wrapping_add(self.registers.f.carry as u8);

                    // Set Flags
                    set_flags_after_adc(self,
                        self.registers.a,
                        original_value,
                        self.bus.read_byte(None, self.registers.get_hl()),
                    );
                    self.pc.wrapping_add(1)
                }
                OPTarget::A => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self.registers.a.wrapping_add(self.registers.f.carry as u8);

                    // Set Flags
                    set_flags_after_adc(self,self.registers.a, original_value, original_value);
                    self.pc.wrapping_add(1)
                }
                OPTarget::D8 => {
                    // Store Original Values
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self
                        .bus
                        .read_byte(None, self.pc + 1)
                        .wrapping_add(self.registers.f.carry as u8);

                    // Set Flags
                    set_flags_after_adc(self,
                        self.registers.a,
                        original_value,
                        self.bus.read_byte(None, self.pc + 1),
                    );
                    self.pc.wrapping_add(2)
                }
            },
            Instruction::SUB(target) => {
                // Get Original Value
                let original_value = self.registers.a;
                match target {
                    OPTarget::B => {
                        // SUB
                        self.registers.a = self.registers.a.wrapping_sub(self.registers.b);

                        // Set Flags
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.b,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::C => {
                        // SUB
                        self.registers.a = self.registers.a.wrapping_sub(self.registers.c);

                        // Set Flags
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.c,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::D => {
                        // SUB
                        self.registers.a = self.registers.a.wrapping_sub(self.registers.d);

                        // Set Flags
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.d,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::E => {
                        // SUB
                        self.registers.a = self.registers.a.wrapping_sub(self.registers.e);

                        // Set Flags
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.e,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::H => {
                        // SUB
                        self.registers.a = self.registers.a.wrapping_sub(self.registers.h);

                        // Set Flags
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.h,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::L => {
                        // SUB
                        self.registers.a = self.registers.a.wrapping_sub(self.registers.l);

                        // Set Flags
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.l,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::HL => {
                        // SUB
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.bus.read_byte(None, self.registers.get_hl()));

                        // Set Flags
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.bus.read_byte(None, self.registers.get_hl()),
                        );

                        self.pc.wrapping_add(3)
                    }
                    OPTarget::A => {
                        // SUB
                        self.registers.a = self.registers.a.wrapping_sub(self.registers.a);

                        // Set Flags
                        set_flags_after_sub(self,self.registers.a, original_value, original_value);

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::D8 => {
                        // SUB
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.bus.read_byte(None, self.pc + 1));

                        // Set Flags
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.bus.read_byte(None, self.pc + 1),
                        );

                        self.pc.wrapping_add(2)
                    }
                }
            }
            Instruction::SBC(target) => {
                let original_value = self.registers.a;
                match target {
                    OPTarget::B => {
                        // SBC
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.registers.b)
                            .wrapping_sub(self.registers.f.carry as u8);

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.b,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::C => {
                        // SBC
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.registers.c)
                            .wrapping_sub(self.registers.f.carry as u8);

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.c,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::D => {
                        let immediate_operand = self.registers.d;
                        let carry_value = self.registers.f.carry as u8;

                        // Perform SBC: A = A - D - carry
                        let result = original_value
                            .wrapping_sub(immediate_operand)
                            .wrapping_sub(carry_value);

                        self.registers.a = result;

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.d,
                        );

                        // Increment the program counter
                        self.pc.wrapping_add(1)
                    }
                    OPTarget::E => {
                        // SBC
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.registers.e)
                            .wrapping_sub(self.registers.f.carry as u8);

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.e,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::H => {
                        // SBC
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.registers.h)
                            .wrapping_sub(self.registers.f.carry as u8);

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.h,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::L => {
                        // SBC
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.registers.l)
                            .wrapping_sub(self.registers.f.carry as u8);

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.l,
                        );

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::HL => {
                        // SBC
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.bus.read_byte(None, self.registers.get_hl()))
                            .wrapping_sub(self.registers.f.carry as u8);

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.registers.get_hl() as u8,
                        );

                        self.pc.wrapping_add(3)
                    }
                    OPTarget::A => {
                        // SBC
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.registers.a)
                            .wrapping_sub(self.registers.f.carry as u8);

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,self.registers.a, original_value, original_value);

                        self.pc.wrapping_add(1)
                    }
                    OPTarget::D8 => {
                        // SBC
                        self.registers.a = self
                            .registers
                            .a
                            .wrapping_sub(self.bus.read_byte(None, self.pc + 1))
                            .wrapping_sub(self.registers.f.carry as u8);

                        // Set Flags -> use sub logic?
                        set_flags_after_sub(self,
                            self.registers.a,
                            original_value,
                            self.bus.read_byte(None, self.pc + 1),
                        );

                        self.pc.wrapping_add(2)
                    }
                }
            }
            Instruction::AND(target) => {
                let result_pc: u16;
                match target {
                    OPTarget::B => {
                        // AND
                        self.registers.a &= self.registers.b;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::C => {
                        // AND
                        self.registers.a &= self.registers.c;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::D => {
                        // AND
                        self.registers.a &= self.registers.d;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::E => {
                        // AND
                        self.registers.a &= self.registers.e;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::H => {
                        // AND
                        self.registers.a &= self.registers.h;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::L => {
                        // AND
                        self.registers.a &= self.registers.l;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::HL => {
                        // AND
                        self.registers.a &= self.bus.read_byte(None, self.registers.get_hl());

                        result_pc = self.pc.wrapping_add(3);
                    }
                    OPTarget::A => {
                        // AND
                        self.registers.a &= self.registers.a;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::D8 => {
                        // AND
                        self.registers.a &= self.bus.read_byte(None, self.pc + 1);

                        result_pc = self.pc.wrapping_add(2);
                    }
                }
                // Set Flags
                set_flags_after_and(self,self.registers.a);

                // Implicit Return
                result_pc
            }
            Instruction::XOR(target) => {
                let result_pc: u16;
                match target {
                    OPTarget::B => {
                        // XOR
                        self.registers.a ^= self.registers.b;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::C => {
                        // XOR
                        self.registers.a ^= self.registers.c;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::D => {
                        // XOR
                        self.registers.a ^= self.registers.d;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::E => {
                        // XOR
                        self.registers.a ^= self.registers.e;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::H => {
                        // XOR
                        self.registers.a ^= self.registers.h;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::L => {
                        // XOR
                        self.registers.a ^= self.registers.l;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::HL => {
                        // XOR
                        self.registers.a ^= self.bus.read_byte(None, self.registers.get_hl());

                        result_pc = self.pc.wrapping_add(3);
                    }
                    OPTarget::A => {
                        // XOR
                        self.registers.a ^= self.registers.a;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::D8 => {
                        // XOR
                        self.registers.a ^= self.bus.read_byte(None, self.pc + 1);

                        result_pc = self.pc.wrapping_add(2);
                    }
                }
                // Set Flags
                set_flags_after_xor_or(self,self.registers.a);

                // Implicit Return
                result_pc
            }
            Instruction::OR(target) => {
                let result_pc: u16;
                match target {
                    OPTarget::B => {
                        // OR
                        self.registers.a |= self.registers.b;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::C => {
                        // OR
                        self.registers.a |= self.registers.c;

                        result_pc = self.pc.wrapping_add(1);
                    }
                    OPTarget::D => {
                        // OR
                        self.registers.a |= self.registers.d;

                        result_pc = self.pc.wrapping_add(1)
                    }
                    OPTarget::E => {
                        // OR
                        self.registers.a |= self.registers.e;

                        result_pc = self.pc.wrapping_add(1)
                    }
                    OPTarget::H => {
                        // OR
                        self.registers.a |= self.registers.h;

                        result_pc = self.pc.wrapping_add(1)
                    }
                    OPTarget::L => {
                        // OR
                        self.registers.a |= self.registers.l;

                        result_pc = self.pc.wrapping_add(1)
                    }
                    OPTarget::HL => {
                        // OR
                        self.registers.a |= self.bus.read_byte(None, self.registers.get_hl());

                        result_pc = self.pc.wrapping_add(3)
                    }
                    OPTarget::A => {
                        // OR
                        self.registers.a |= self.registers.a;

                        result_pc = self.pc.wrapping_add(1)
                    }
                    OPTarget::D8 => {
                        // OR
                        self.registers.a = self.bus.read_byte(None, self.pc + 1);

                        result_pc = self.pc.wrapping_add(2)
                    }
                }
                // Set Flags
                set_flags_after_xor_or(self,self.registers.a);

                // Implicit Return
                result_pc
            }
            Instruction::CP(target) => match target {
                OPTarget::B => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,self.registers.a, self.registers.b);

                    self.pc.wrapping_add(1)
                }
                OPTarget::C => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,self.registers.a, self.registers.c);

                    self.pc.wrapping_add(1)
                }
                OPTarget::D => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,self.registers.a, self.registers.d);

                    self.pc.wrapping_add(1)
                }
                OPTarget::E => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,self.registers.a, self.registers.e);

                    self.pc.wrapping_add(1)
                }
                OPTarget::H => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,self.registers.a, self.registers.h);

                    self.pc.wrapping_add(1)
                }
                OPTarget::L => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,self.registers.a, self.registers.l);

                    self.pc.wrapping_add(1)
                }
                OPTarget::HL => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,self.registers.a, self.registers.get_hl() as u8);

                    self.pc.wrapping_add(3)
                }
                OPTarget::A => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,self.registers.a, self.registers.a);
                    self.pc.wrapping_add(1)
                }
                OPTarget::D8 => {
                    // CP -> Set Flags
                    set_flags_after_cp(self,
                        self.registers.a,
                        self.bus.read_byte(None, self.pc + 1),
                    );
                    self.pc.wrapping_add(2)
                }
            },
            Instruction::RET(test) => {
                let jump_condition = match_jump(self, test);
                panic!("RET NOT IMPLEMENTED")
            }
            Instruction::RETI => {
                panic!("RETI NOT IMPLEMENTED")
            }
            Instruction::POP(target) => {
                panic!("POP NOT IMPLEMENTED")
            }
            Instruction::JP(test) => {
                let jump_condition = match_jump(self, test);
                jump(self, jump_condition)
            }
            Instruction::CALL(test) => {
                let jump_condition = match_jump(self, test);
                panic!("CALL NOT IMPLEMENTED")
            }
            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::AF => self.registers.get_af(),
                    StackTarget::BC => self.registers.get_bc(),
                    StackTarget::DE => self.registers.get_de(),
                    StackTarget::HL => self.registers.get_hl(),
                };
                // push value to stack
                //push(self, value);

                // increment pc
                self.pc.wrapping_add(1)
            }
            Instruction::RST(target) => {
                // Push PC to memory stack
                //self.push(self.pc);

                // Wait to see how this is done

                // After Push decrement SP and

                panic!("RST NOT IMPLEMENTED")
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
                // Find Target Register
                let mut reg_target = match_hl(self, target);

                // Perform Operation
                op_rlc(self, &mut reg_target);

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::RRC(target) => {
                // Find target Register
                let mut reg_target = match_hl(self, target);

                // Perform Operation
                op_rrc(self, &mut reg_target);

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::RL(target) => {
                // Find Target Register
                let mut reg_target = match_hl(self, target);

                // Perform Operation
                op_rl(self, &mut reg_target);

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::RR(target) => {
                // Find Target Register
                let mut reg_target = match_hl(self, target);

                // Perform Operation
                op_rr(self, &mut reg_target);

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::SLA(target) => {
                let mut reg_target = match_hl(self, target);

                op_sla(self, &mut reg_target);

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::SRA(target) => {
                // Find Target Register
                let mut reg_target = match_hl(self,target);

                // Perform Operation
                op_sra(self, &mut reg_target);

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::SWAP(target) => {
                // Find Target Register
                let mut reg_target = match_hl(self,target);

                // Perform Operation
                op_swap(self, &mut reg_target);

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::SRL(target) => {
                // Find Target Register
                let mut reg_target = match_hl(self,target);

                // Perform Operation
                op_srl(self, &mut reg_target);

                // Implicit Return
                self.pc.wrapping_add(1)
            }
            Instruction::BIT(target) => {
                // Perform Operation
                op_bit(self, target);

                // Prefixed Return
                self.pc.wrapping_add(2)
            }
            Instruction::RES(target) => {
                // Perform Operation
                op_res(self, target);

                // Prefixed Return
                self.pc.wrapping_add(2)
            }
            Instruction::SET(target) => {
                // Perform Operation
                op_set(self, target);
                
                // Prefixed Return
                self.pc.wrapping_add(2)
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
