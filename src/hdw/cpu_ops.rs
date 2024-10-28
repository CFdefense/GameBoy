/*

    Helper File to Contain Helper Utilization Functions For CPU Execute Operations

*/
use crate::hdw::cpu::*;
use crate::hdw::cpu_util::*;
use crate::hdw::instructions::*;

pub fn op_srl(cpu: &mut CPU, reg_target: &mut u8) {
    // Get LSB For Carry Flag
    let lsb = *reg_target & 0x1;

    // Shift Right
    *reg_target = *reg_target >> 1;

    // Update Flags
    cpu.registers.f.carry = lsb != 0;
    cpu.registers.f.zero = *reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

pub fn op_swap(cpu: &mut CPU, reg_target: &mut u8) {
    // Swap the the nibbles
    *reg_target = (*reg_target << 4) | (*reg_target >> 4);

    // Upd Flags
    cpu.registers.f.zero = *reg_target == 0;
    cpu.registers.f.carry = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

pub fn op_sra(cpu: &mut CPU, reg_target: &mut u8) {
    // Get LSB For Carry
    let lsb = *reg_target & 0x1;

    // Preserve Sign Bit
    let sign_bit = (*reg_target & 0x80) != 0;

    // Shift Right
    *reg_target >>= 1;

    // Put Sign Bit Back
    if sign_bit {
        *reg_target |= 0x80;
    }

    // Update Flags
    cpu.registers.f.carry = lsb != 0;
    cpu.registers.f.zero = *reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

pub fn op_sla(cpu: &mut CPU, reg_target: &mut u8) {
    // Get Bit 7 For Carry
    let bit_7 = (*reg_target & 0x80) != 0;

    // Shift Left
    *reg_target <<= 1;

    // Update Flag
    cpu.registers.f.carry = bit_7;
    cpu.registers.f.zero = *reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

pub fn op_rlc(cpu: &mut CPU, reg_target: &mut u8) {
    // Get Bit 7 For Carry
    let bit_7 = (*reg_target >> 7) & 0x1;

    // Rotate Left With Carry
    *reg_target = (*reg_target << 1) | bit_7;

    // Update Flags
    cpu.registers.f.zero = *reg_target == 0;
    cpu.registers.f.carry = bit_7 != 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

pub fn op_rrc(cpu: &mut CPU, reg_target: &mut u8) {
    // Get Bit 0 For Carry
    let bit_0 = *reg_target & 0x1;

    // Rotate Right and Append bit 0
    *reg_target = (*reg_target >> 1) | (bit_0 >> 7);

    // Update Flags
    cpu.registers.f.carry = bit_0 != 0;
    cpu.registers.f.zero = *reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

pub fn op_rl(cpu: &mut CPU, reg_target: &mut u8) {
    // Store Previous Carry
    let prev_carry = cpu.registers.f.carry;

    // Store Bit 7 For Carry
    let bit_7 = (*reg_target >> 7) & 0x1;

    // Rotate Left and Append
    *reg_target = (*reg_target << 1) | (prev_carry as u8);

    // Update Flags
    cpu.registers.f.carry = bit_7 != 0;
    cpu.registers.f.zero = *reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

pub fn op_rr(cpu: &mut CPU, reg_target: &mut u8) {
    // Store Previous Carry
    let prev_carry = cpu.registers.f.carry;

    // Store Bit 0
    let bit_0 = *reg_target & 0x1;

    // Rotate Right and append bit 0
    *reg_target = (*reg_target >> 1) | (prev_carry as u8) << 7;

    // Update Flags
    cpu.registers.f.carry = bit_0 != 0;
    cpu.registers.f.zero = *reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

pub fn op_bit(cpu: &mut CPU, target: ByteTarget) {
    let bit: u8;
    let target_register: u8;
    match target {
        ByteTarget::Zero(hl_target) => {
            bit = 0b00000010; // Byte to match
            target_register = match_hl(cpu, hl_target); // find target
        }
        ByteTarget::One(hl_target) => {
            bit = 0b00000100; // Byte to match
            target_register = match_hl(cpu, hl_target); // find target
        }
        ByteTarget::Two(hl_target) => {
            bit = 0b00001000; // Byte to match
            target_register = match_hl(cpu, hl_target); // find target
        }
        ByteTarget::Three(hl_target) => {
            bit = 0b00010000; // Byte to match
            target_register = match_hl(cpu, hl_target); // find target
        }
        ByteTarget::Four(hl_target) => {
            bit = 0b00100000; // Byte to match
            target_register = match_hl(cpu, hl_target); // find target
        }
        ByteTarget::Five(hl_target) => {
            bit = 0b01000000; // Byte to match
            target_register = match_hl(cpu, hl_target); // find target
        }
        ByteTarget::Six(hl_target) => {
            bit = 0b10000000; // Byte to match
            target_register = match_hl(cpu, hl_target); // find target
        }
        ByteTarget::Seven(hl_target) => {
            bit = 0b00000000; // Byte to match
            target_register = match_hl(cpu, hl_target); // find target
        }
    }

    // Set Flags
    set_flags_after_bit(cpu, bit, target_register);
}

pub fn op_res(cpu: &mut CPU, target: ByteTarget) {
    let mask: u8;
    let mut target_register: u8;
    let mut is_mem: bool = false;
    match target {
        ByteTarget::Zero(hl_target) => {
            mask = 0b11111110; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::One(hl_target) => {
            mask = 0b11111101; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Two(hl_target) => {
            mask = 0b11111011; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Three(hl_target) => {
            mask = 0b11110111; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Four(hl_target) => {
            mask = 0b11101111; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Five(hl_target) => {
            mask = 0b11011111; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Six(hl_target) => {
            mask = 0b10111111; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Seven(hl_target) => {
            mask = 0b01111111; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
    }

    // Perform Operation
    if is_mem {
        // if were updating memory write back to grabbed location the new value
        cpu.bus
            .write_byte(None, cpu.registers.get_hl(), target_register & mask);
    } else {
        target_register &= mask;
    }
}

pub fn op_set(cpu: &mut CPU, target: ByteTarget) {
    let mask: u8;
    let mut target_register: u8;
    let mut is_mem: bool = false;
    match target {
        ByteTarget::Zero(hl_target) => {
            mask = 0b00000001; // Byte Mask
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::One(hl_target) => {
            mask = 0b00000010;
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Two(hl_target) => {
            mask = 0b00000100;
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Three(hl_target) => {
            mask = 0b00001000;
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Four(hl_target) => {
            mask = 0b00010000;
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Five(hl_target) => {
            mask = 0b00100000;
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Six(hl_target) => {
            mask = 0b01000000;
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
        ByteTarget::Seven(hl_target) => {
            mask = 0b10000000;
            match hl_target {
                HLTarget::HL => {
                    is_mem = true; // flag that were grabbing memory
                }
                _ => {}
            }
            target_register = match_hl(cpu, hl_target);
        }
    }
    // Perform Operation
    if is_mem {
        // if were updating memory write back to grabbed location the new value
        cpu.bus
            .write_byte(None, cpu.registers.get_hl(), target_register & mask);
    } else {
        target_register &= mask;
    }
}
