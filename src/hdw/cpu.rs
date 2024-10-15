use crate::hdw::bus::Bus;
use crate::hdw::instructions::*;
use crate::hdw::registers::*;
use core::panic;

// Our CPU to Call and Control
pub struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    bus: Bus,
    is_halted: bool,
    curr_opcode: u8,
    curr_instruction: Option<Instruction>,
}
impl CPU {
    // Contructor
    pub fn new(new_bus: Bus) -> Self {
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
            bus: new_bus,
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
        self.curr_opcode = self.bus.read_byte(self.pc);
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
                let jump_distance = self.bus.read_byte(self.pc + 1) as i8;
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
                    // Increment value at bus location HL
                    AllRegisters::HLMEM => {
                        let hl_addr = self.registers.get_hl();
                        let value = self.bus.read_byte(hl_addr).wrapping_add(1);
                        self.bus.write_byte(hl_addr, value);
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
                    // Increment value at bus location HL
                    AllRegisters::HLMEM => {
                        let hl_addr = self.registers.get_hl();
                        let original_value = self.bus.read_byte(hl_addr);
                        let value = self.bus.read_byte(hl_addr).wrapping_sub(1);
                        self.bus.write_byte(hl_addr, value);
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
                    HLTarget::B => match source {
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
                            self.registers.b = self.bus.read_byte(self.registers.get_hl());
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
                            self.registers.c = self.bus.read_byte(self.registers.get_hl());
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
                            self.registers.d = self.bus.read_byte(self.registers.get_hl());
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
                            self.registers.e = self.bus.read_byte(self.registers.get_hl());
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
                            self.registers.h = self.bus.read_byte(self.registers.get_hl());
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
                            self.registers.l = self.bus.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.l = self.registers.a;
                            self.pc + 1
                        }
                    },
                    HLTarget::HL => match target {
                        HLTarget::B => {
                            self.bus
                                .write_byte(self.registers.get_hl(), self.registers.b);
                            self.pc + 1
                        }
                        HLTarget::C => {
                            self.bus
                                .write_byte(self.registers.get_hl(), self.registers.c);
                            self.pc + 1
                        }
                        HLTarget::D => {
                            self.bus
                                .write_byte(self.registers.get_hl(), self.registers.d);
                            self.pc + 1
                        }
                        HLTarget::E => {
                            self.bus
                                .write_byte(self.registers.get_hl(), self.registers.e);
                            self.pc + 1
                        }
                        HLTarget::H => {
                            self.bus
                                .write_byte(self.registers.get_hl(), self.registers.h);
                            self.pc + 1
                        }
                        HLTarget::L => {
                            self.bus
                                .write_byte(self.registers.get_hl(), self.registers.l);
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.bus
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
                            self.registers.a = self.bus.read_byte(self.registers.get_hl());
                            self.pc + 1
                        }
                        HLTarget::A => {
                            self.registers.a = self.registers.a;
                            self.pc + 1
                        }
                    },
                },
                LoadType::Word(target, source) => {
                    // Read the next two bytes from bus at the current PC
                    let low_byte = self.bus.read_byte(self.pc + 1); // Read the low byte
                    let high_byte = self.bus.read_byte(self.pc + 2); // Read the high byte

                    // Combine the low and high bytes into a 16-bit value
                    let word_value = ((high_byte as u16) << 8) | (low_byte as u16);

                    match target {
                        LoadWordTarget::BC => match source {
                            LoadWordSource::N16 => {
                                self.registers.set_bc(self.bus.read_byte(word_value) as u16);
                                self.pc + 3
                            }
                            _ => panic!("BAD MATCH"),
                        },
                        LoadWordTarget::HL => match source {
                            LoadWordSource::N16 => {
                                self.registers.set_hl(self.bus.read_byte(word_value) as u16);
                                self.pc + 3
                            }
                            LoadWordSource::SPE8 => {
                                self.registers.set_hl(
                                    ((self.sp as i16).wrapping_add(
                                        (self.bus.read_byte(self.pc + 1) as i8) as i16,
                                    )) as u16,
                                );

                                // UPD FLAGS
                                self.registers.f.subtract = false;

                                self.registers.f.half_carry = ((self.sp & 0x0F)
                                    + (self.bus.read_byte(self.pc + 1) as u16 & 0x0F))
                                    > 0x0F;

                                self.registers.f.carry = ((self.sp & 0xFF)
                                    + (self.bus.read_byte(self.pc + 1) as u16 & 0xFF))
                                    > 0xFF;

                                self.pc + 2
                            }
                            _ => panic!("BAD MATCH"),
                        },
                        LoadWordTarget::DE => match source {
                            LoadWordSource::N16 => {
                                self.registers.set_de(self.bus.read_byte(word_value) as u16);
                                self.pc + 3
                            }
                            _ => panic!("BAD MATCH"),
                        },
                        LoadWordTarget::N16 => match source {
                            LoadWordSource::SP => {
                                self.bus.write_byte(word_value, (self.sp & 0x00FF) as u8);
                                self.bus.write_byte(word_value + 1, (self.sp >> 8) as u8);
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
                        self.bus
                            .write_byte(self.registers.get_bc(), self.registers.a);
                        self.pc + 1
                    }
                    LoadN16::DE => {
                        self.bus
                            .write_byte(self.registers.get_de(), self.registers.a);
                        self.pc + 1
                    }
                    LoadN16::HLDEC => {
                        self.bus
                            .write_byte(self.registers.get_hl(), self.registers.a);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_sub(1));
                        self.pc + 1
                    }
                    LoadN16::HLINC => {
                        self.bus
                            .write_byte(self.registers.get_hl(), self.registers.a);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_add(1));
                        self.pc + 1
                    }
                },
                LoadType::N16StoreInA(source) => match source {
                    LoadN16::BC => {
                        self.registers.a = self.bus.read_byte(self.registers.get_bc());
                        self.pc + 1
                    }
                    LoadN16::DE => {
                        self.registers.a = self.bus.read_byte(self.registers.get_de());
                        self.pc + 1
                    }
                    LoadN16::HLDEC => {
                        self.registers.a = self.bus.read_byte(self.registers.get_hl());
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_sub(1));
                        self.pc + 1
                    }
                    LoadN16::HLINC => {
                        self.registers.a = self.bus.read_byte(self.registers.get_hl());
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_add(1));
                        self.pc + 1
                    }
                },
                LoadType::D8StoreInReg(target) => match target {
                    HLTarget::B => {
                        self.registers.b = self.bus.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::C => {
                        self.registers.c = self.bus.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::D => {
                        self.registers.d = self.bus.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::E => {
                        self.registers.e = self.bus.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::H => {
                        self.registers.h = self.bus.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::L => {
                        self.registers.l = self.bus.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                    HLTarget::HL => {
                        self.bus
                            .write_byte(self.registers.get_hl(), self.bus.read_byte(self.pc + 1));
                        self.pc + 2
                    }
                    HLTarget::A => {
                        self.registers.a = self.bus.read_byte(self.pc + 1);
                        self.pc + 2
                    }
                },
                LoadType::AWithA8(target) => match target {
                    LoadA8Target::A => {
                        self.registers.a = self
                            .bus
                            .read_byte(0xFF00 + self.bus.read_byte(self.pc + 1) as u16);
                        self.pc + 2
                    }
                    LoadA8Target::A8 => {
                        self.bus.write_byte(
                            0xFF00 + self.bus.read_byte(self.pc + 1) as u16,
                            self.registers.a,
                        );
                        self.pc + 2
                    }
                },
                LoadType::AWithA16(target) => {
                    let low_byte = self.bus.read_byte(self.pc + 1); // Read the low byte
                    let high_byte = self.bus.read_byte(self.pc + 2); // Read the high byte

                    // Combine the low and high bytes into a 16-bit value
                    let address = ((high_byte as u16) << 8) | (low_byte as u16);

                    match target {
                        LoadA16Target::A => {
                            self.registers.a = self.bus.read_byte(address);
                            self.pc + 3
                        }
                        LoadA16Target::A16 => {
                            self.bus.write_byte(address, self.registers.a);
                            self.pc + 3
                        }
                    }
                }
                LoadType::AWithAC(target) => match target {
                    LoadACTarget::A => {
                        self.bus
                            .write_byte(0xFF00 + self.registers.c as u16, self.registers.a);
                        self.pc + 2
                    }
                    LoadACTarget::C => {
                        self.registers.a = self.bus.read_byte(0xFF00 + self.registers.c as u16);
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
                    let immediate_operand: i8 = self.bus.read_byte(self.pc + 1) as i8;

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
                    let immediate_operand: u8 = self.bus.read_byte(self.pc + 1);
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
                OPTarget::B => {
                    // Store Original Value
                    let original_value = self.registers.a;
                    // ADC
                    self.registers.a = self.registers.b.wrapping_add(self.registers.f.carry as u8);

                    // Upd Flags
                    self.update_flags_after_adc(self.registers.a, original_value, self.registers.b);
                    self.pc + 1
                }
                OPTarget::C => {
                    // Store Original Value
                    let original_value = self.registers.a;
                    // ADC
                    self.registers.a = self.registers.c.wrapping_add(self.registers.f.carry as u8);
                    // Upd Flags
                    self.update_flags_after_adc(self.registers.a, original_value, self.registers.c);
                    self.pc + 1
                }
                OPTarget::E => {
                    // Store Original Value
                    let original_value = self.registers.a;
                    // ADC
                    self.registers.a = self.registers.e.wrapping_add(self.registers.f.carry as u8);

                    // Upd Flags
                    self.update_flags_after_adc(self.registers.a, original_value, self.registers.e);
                    self.pc + 1
                }
                OPTarget::D => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self.registers.d.wrapping_add(self.registers.f.carry as u8);

                    // Upd Flags
                    self.update_flags_after_adc(self.registers.a, original_value, self.registers.d);
                    self.pc + 1
                }
                OPTarget::H => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self.registers.h.wrapping_add(self.registers.f.carry as u8);

                    // Upd Flags
                    self.update_flags_after_adc(self.registers.a, original_value, self.registers.h);
                    self.pc + 1
                }
                OPTarget::L => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self.registers.l.wrapping_add(self.registers.f.carry as u8);

                    // Upd Flags
                    self.update_flags_after_adc(self.registers.a, original_value, self.registers.l);
                    self.pc + 1
                }
                OPTarget::HL => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self
                        .bus
                        .read_byte(self.registers.get_hl())
                        .wrapping_add(self.registers.f.carry as u8);

                    // Upd Flags
                    self.update_flags_after_adc(
                        self.registers.a,
                        original_value,
                        self.bus.read_byte(self.registers.get_hl()),
                    );
                    self.pc + 1
                }
                OPTarget::A => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self.registers.a.wrapping_add(self.registers.f.carry as u8);

                    // Upd Flags
                    self.update_flags_after_adc(self.registers.a, original_value, self.registers.a);
                    self.pc + 1
                }
                OPTarget::D8 => {
                    // Store Original Value
                    let original_value = self.registers.a;

                    // ADC
                    self.registers.a = self
                        .bus
                        .read_byte(self.pc + 1)
                        .wrapping_add(self.registers.f.carry as u8);

                    // Upd Flags
                    self.update_flags_after_adc(
                        self.registers.a,
                        original_value,
                        self.bus.read_byte(self.pc + 1),
                    );
                    self.pc + 2
                }
            },
            Instruction::SUB(target) => match target {
                OPTarget::B => {
                    // SUB
                    self.registers.a = self.registers.a.wrapping_sub(self.registers.b);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::C => {
                    // SUB
                    self.registers.a = self.registers.a.wrapping_sub(self.registers.c);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D => {
                    // SUB
                    self.registers.a = self.registers.a.wrapping_sub(self.registers.d);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::E => {
                    // SUB
                    self.registers.a = self.registers.a.wrapping_sub(self.registers.e);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::H => {
                    // SUB
                    self.registers.a = self.registers.a.wrapping_sub(self.registers.h);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::L => {
                    // SUB
                    self.registers.a = self.registers.a.wrapping_sub(self.registers.l);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::HL => {
                    // SUB
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.bus.read_byte(self.registers.get_hl()));

                    // Upd Flags

                    self.pc + 3
                }
                OPTarget::A => {
                    // SUB
                    self.registers.a = self.registers.a.wrapping_sub(self.registers.a);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D8 => {
                    // SUB
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.bus.read_byte(self.pc + 1));

                    // Upd Flags

                    self.pc + 2
                }
            },
            Instruction::SBC(target) => match target {
                OPTarget::B => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.registers.b)
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::C => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.registers.c)
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.registers.d)
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::E => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.registers.e)
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::H => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.registers.h)
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::L => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.registers.l)
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::HL => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.bus.read_byte(self.registers.get_hl()))
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 3
                }
                OPTarget::A => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.registers.a)
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D8 => {
                    // SBC
                    self.registers.a = self
                        .registers
                        .a
                        .wrapping_sub(self.bus.read_byte(self.pc + 1))
                        .wrapping_sub(self.registers.f.carry as u8);

                    // Upd Flags

                    self.pc + 2
                }
            },
            Instruction::AND(target) => match target {
                OPTarget::B => {
                    // AND
                    self.registers.a &= self.registers.b;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::C => {
                    // AND
                    self.registers.a &= self.registers.c;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D => {
                    // AND
                    self.registers.a &= self.registers.d;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::E => {
                    // AND
                    self.registers.a &= self.registers.e;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::H => {
                    // AND
                    self.registers.a &= self.registers.h;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::L => {
                    // AND
                    self.registers.a &= self.registers.l;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::HL => {
                    // AND
                    self.registers.a &= self.bus.read_byte(self.registers.get_hl());

                    // Upd Flags

                    self.pc + 3
                }
                OPTarget::A => {
                    // AND
                    self.registers.a &= self.registers.a;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D8 => {
                    // AND
                    self.registers.a &= self.bus.read_byte(self.pc + 1);

                    // Upd Flags

                    self.pc + 2
                }
            },
            Instruction::XOR(target) => match target {
                OPTarget::B => {
                    // XOR
                    self.registers.a ^= self.registers.b;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::C => {
                    // XOR
                    self.registers.a ^= self.registers.c;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D => {
                    // XOR
                    self.registers.a ^= self.registers.d;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::E => {
                    // XOR
                    self.registers.a ^= self.registers.e;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::H => {
                    // XOR
                    self.registers.a ^= self.registers.h;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::L => {
                    // XOR
                    self.registers.a ^= self.registers.l;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::HL => {
                    // XOR
                    self.registers.a ^= self.bus.read_byte(self.registers.get_hl());

                    // Upd Flags

                    self.pc + 3
                }
                OPTarget::A => {
                    // XOR
                    self.registers.a ^= self.registers.a;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D8 => {
                    // XOR
                    self.registers.a ^= self.bus.read_byte(self.pc + 1);

                    // Upd Flags

                    self.pc + 2
                }
            },
            Instruction::OR(target) => match target {
                OPTarget::B => {
                    // OR
                    self.registers.a |= self.registers.b;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::C => {
                    // OR
                    self.registers.a |= self.registers.c;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D => {
                    // OR
                    self.registers.a |= self.registers.d;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::E => {
                    // OR
                    self.registers.a |= self.registers.e;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::H => {
                    // OR
                    self.registers.a |= self.registers.h;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::L => {
                    // OR
                    self.registers.a |= self.registers.l;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::HL => {
                    // OR
                    self.registers.a |= self.bus.read_byte(self.registers.get_hl());

                    // Upd Flags

                    self.pc + 3
                }
                OPTarget::A => {
                    // OR
                    self.registers.a |= self.registers.a;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::D8 => {
                    // OR
                    self.registers.a = self.bus.read_byte(self.pc + 1);

                    // Upd Flags

                    self.pc + 2
                }
            },
            Instruction::CP(target) => match target {
                OPTarget::B => {
                    // CP
                    let cp_value = self.registers.a.wrapping_sub(self.registers.b);
                    self.registers.f.zero = cp_value == 0;

                    // Upd Flags

                    self.pc + 1
                }
                OPTarget::C => {
                    // CP
                    let cp_value = self.registers.a.wrapping_sub(self.registers.c);
                    self.registers.f.zero = cp_value == 0;

                    // Upd Flag

                    self.pc + 1
                }
                OPTarget::D => {
                    // CP
                    let cp_value = self.registers.a.wrapping_sub(self.registers.d);
                    self.registers.f.zero = cp_value == 0;

                    // Upd Flag

                    self.pc + 1
                }
                OPTarget::E => {
                    // CP
                    let cp_value = self.registers.a.wrapping_sub(self.registers.e);
                    self.registers.f.zero = cp_value == 0;

                    // Upd Flag

                    self.pc + 1
                }
                OPTarget::H => {
                    // CP
                    let cp_value = self.registers.a.wrapping_sub(self.registers.h);
                    self.registers.f.zero = cp_value == 0;

                    // Upd Flag

                    self.pc + 1
                }
                OPTarget::L => {
                    // CP
                    let cp_value = self.registers.a.wrapping_sub(self.registers.l);
                    self.registers.f.zero = cp_value == 0;

                    // Upd Flag

                    self.pc + 1
                }
                OPTarget::HL => {
                    // CP
                    let cp_value = self
                        .registers
                        .a
                        .wrapping_sub(self.bus.read_byte(self.registers.get_hl()));
                    self.registers.f.zero = cp_value == 0;

                    // Upd Flag

                    self.pc + 3
                }
                OPTarget::A => {
                    // CP
                    let cp_value = self.registers.a.wrapping_sub(self.registers.a);
                    self.registers.f.zero = cp_value == 0;

                    // Upd Flag

                    self.pc + 1
                }
                OPTarget::D8 => {
                    // CP
                    let cp_value = self
                        .registers
                        .a
                        .wrapping_sub(self.bus.read_byte(self.pc + 1));
                    self.registers.f.zero = cp_value == 0;
                    // Upd Flag

                    self.pc + 2
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

    // Jump to addr in bus or increment pc
    fn jump(&self, jump: bool) -> u16 {
        if jump {
            let least_significant = self.bus.read_byte(self.pc + 1) as u16;
            let most_significant = self.bus.read_byte(self.pc + 2) as u16;

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

        // mask shift and write first byte to bus at SP
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        // increment stack pointer
        self.sp = self.sp.wrapping_add(1);

        // mask and write second byte to bus at SP
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
    }

    // Pop from stack and increment pointers
    fn pop(&mut self) -> u16 {
        // read least significant byte from bus at SP
        let least_significant_byte = self.bus.read_byte(self.sp) as u16;

        // increment stack pointer
        self.sp = self.sp.wrapping_add(1);

        // read most significan byte from bus at SP
        let most_significant_byte = self.bus.read_byte(self.sp) as u16;

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
            self.bus.read_next_byte();
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
            AllRegisters::HLMEM => self.bus.read_byte(self.registers.get_hl()) as u16,
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
            HLTarget::HL => self.bus.read_byte(self.registers.get_hl()),
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

    // Method to update relevant flags after ADC instructions
    fn update_flags_after_adc(&mut self, result: u8, original_value: u8, immediate_operand: u8) {
        // Zero Flag: Set if the result is zero
        self.registers.f.zero = result == 0;

        // Subtract Flag: SET (ADC is not a subtraction)
        self.registers.f.subtract = false;

        // Half-Carry Flag: Set if there was a carry from bit 4 to bit 3
        let half_carry = ((original_value & 0x0F) + (immediate_operand & 0x0F)) > 0x0F; // Check for carry from the lower nibble
        self.registers.f.half_carry = half_carry;

        // Carry Flag: Set if there was a carry from the 8th bit
        self.registers.f.carry = (result < original_value) || (result < immediate_operand);
    }
    // CPU ENDS HERE
}
