/*

    Helper File to Contain Helper Utilization Functions For CPU Execute Operations

*/
use crate::hdw::cpu::*;
use crate::hdw::cpu_util::*;
use crate::hdw::instructions::*;
use crate::hdw::stack::*;

// [0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F]
pub fn op_srl(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, &target);

    // Get LSB For Carry Flag
    let lsb = reg_target & 0x1;

    // Shift Right
    reg_target = reg_target >> 1;

    // Update Flags
    set_flags_after_pref_op(cpu, lsb, reg_target);

    
    cpu.pc.wrapping_add(1)
}

// [0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37]
pub fn op_swap(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, &target);

    // Swap the the nibbles
    reg_target = (reg_target << 4) | (reg_target >> 4);

    // Upd Flags
    set_flags_after_swap(cpu, reg_target);

    
    cpu.pc.wrapping_add(1)
}

// [0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F]
pub fn op_sra(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, &target);

    // Get LSB For Carry
    let lsb = reg_target & 0x1;

    // Preserve Sign Bit
    let sign_bit = (reg_target & 0x80) != 0;

    // Shift Right
    reg_target >>= 1;

    // Put Sign Bit Back
    if sign_bit {
        reg_target |= 0x80;
    }

    // Update Flags
    set_flags_after_pref_op(cpu, lsb, reg_target);

    
    cpu.pc.wrapping_add(1)
}

// [0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27]
pub fn op_sla(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, &target);

    // Get Bit 7 For Carry
    let bit_7 = (reg_target >> 7) & 0x1;

    // Shift Left
    reg_target <<= 1;

    // Update Flag
    set_flags_after_pref_op(cpu, bit_7, reg_target);

    
    cpu.pc.wrapping_add(1)
}

// [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
pub fn op_rlc(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, &target);

    // Get Bit 7 For Carry
    let bit_7 = (reg_target >> 7) & 0x1;

    // Rotate Left With Carry
    reg_target = (reg_target << 1) | bit_7;

    // Update Flags
    set_flags_after_pref_op(cpu, bit_7, reg_target);

    
    cpu.pc.wrapping_add(1)
}

// [0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
pub fn op_rrc(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find target Register
    let mut reg_target = match_hl(cpu, &target);

    // Get Bit 0 For Carry
    let bit_0 = reg_target & 0x1;

    // Rotate Right and Append bit 0
    reg_target = (reg_target >> 1) | (bit_0 >> 7);

    // Update Flags
    set_flags_after_pref_op(cpu, bit_0, reg_target);

    
    cpu.pc.wrapping_add(1)
}

// [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17]
pub fn op_rl(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, &target);

    // Store Previous Carry
    let prev_carry = cpu.registers.f.carry;

    // Store Bit 7 For Carry
    let bit_7 = (reg_target >> 7) & 0x1;

    // Rotate Left and Append
    reg_target = (reg_target << 1) | (prev_carry as u8);

    // Update Flags
    set_flags_after_pref_op(cpu, bit_7, reg_target);

    
    cpu.pc.wrapping_add(1)
}

// [0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F]
pub fn op_rr(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, &target);

    // Store Previous Carry
    let prev_carry = cpu.registers.f.carry;

    // Store Bit 0
    let bit_0 = reg_target & 0x1;

    // Rotate Right and append bit 0
    reg_target = (reg_target >> 1) | (prev_carry as u8) << 7;

    // Update Flags
    set_flags_after_pref_op(cpu, bit_0, reg_target);

    
    cpu.pc.wrapping_add(1)
}

// [0x2F]
pub fn op_cpl(cpu: &mut CPU) -> u16 {
    // Flip all bits of register A
    cpu.registers.a = !cpu.registers.a;

    // Set flags
    set_flags_after_cpl(cpu);

    
    cpu.pc.wrapping_add(1)
}

// [0x27]
pub fn op_daa(cpu: &mut CPU) -> u16 {
    let mut adjustment = 0;
    let mut carry = false;

    // If the subtract flag is clear, this is an addition
    if !cpu.registers.f.subtract {
        if cpu.registers.f.half_carry || cpu.registers.a & 0x0F > 9 {
            adjustment += 0x06;
        }
        if cpu.registers.f.carry || cpu.registers.a > 0x99 {
            adjustment += 0x60;
            carry = true;
        }
    } else {
        // If subtract is set, it's a subtraction
        if cpu.registers.f.half_carry {
            adjustment -= 0x06;
        }
        if cpu.registers.f.carry {
            adjustment -= 0x60;
        }
    }

    // Apply the adjustment to the accumulator
    cpu.registers.a = cpu.registers.a.wrapping_add(adjustment);

    // Update Flags
    set_flags_after_daa(cpu, carry);

    
    cpu.pc.wrapping_add(1)
}

// [0x1F]
pub fn op_rra(cpu: &mut CPU) -> u16 {
    // Store the original bit 0 to set the carry flag
    let bit_0 = cpu.registers.a & 1;

    // Rotate right: shift right by 1 and add carry to bit 7
    cpu.registers.a = (cpu.registers.a >> 1) | (cpu.registers.f.carry as u8) << 7;

    // Update Flags
    set_flags_after_no_pre_rl_rr(cpu, bit_0);

    
    cpu.pc.wrapping_add(1)
}

// [0x17]
pub fn op_rla(cpu: &mut CPU) -> u16 {
    // Store the original bit 7 to set the carry flag
    let bit_7 = (cpu.registers.a & 0x80) >> 7;

    // Rotate left: shift left by 1 and add carry to bit 0
    cpu.registers.a = (cpu.registers.a << 1) | (cpu.registers.f.carry as u8);

    // Update Flags
    set_flags_after_no_pre_rl_rr(cpu, bit_7);

    
    cpu.pc.wrapping_add(1)
}

// [0x0F]
pub fn op_rrca(cpu: &mut CPU) -> u16 {
    // Store the original bit 0 to set the carry flag and bit 7
    let bit_0 = cpu.registers.a & 1;

    // Rotate right: shift right by 1 and add bit 0 to bit 7
    cpu.registers.a = (cpu.registers.a >> 1) | (bit_0 << 7);

    // Update Flags
    set_flags_after_no_pre_rl_rr(cpu, bit_0);

    
    cpu.pc.wrapping_add(1)
}
// [0x07]
pub fn op_rlca(cpu: &mut CPU) -> u16 {
    // Store the original bit 7 to set the Carry flag and bit 0
    let bit_7 = (cpu.registers.a >> 7) & 1;

    // Rotate left: shift left by 1 and add bit 7 to bit 0
    cpu.registers.a = (cpu.registers.a << 1) | bit_7;

    // Update Flags
    set_flags_after_no_pre_rl_rr(cpu, bit_7);

    
    cpu.pc.wrapping_add(1)
}

// [0xC2, 0xC3, 0xCA, 0xD2, 0xDA, 0xE9]
pub fn op_jp(cpu: &mut CPU, target: JumpTest) -> u16 {
    // Match Jump
    let jump = match_jump(cpu, target);

    // Get Bytes
    let least_significant = cpu.bus.read_byte(None, cpu.pc + 1) as u16;
    let most_significant = cpu.bus.read_byte(None, cpu.pc + 2) as u16;

    // Perform Operation & Implicit Return
    goto_addr(
        cpu,
        (most_significant << 8) | least_significant,
        jump,
        false,
    )
}

// [0xC4, 0xCC, 0xCD, 0xD4, 0xDC]
pub fn op_call(cpu: &mut CPU, target: JumpTest) -> u16 {
    // Jump to addr in bus or increment pc
    // Match Jump
    let jump = match_jump(cpu, target);

    // Get Bytes
    let least_significant = cpu.bus.read_byte(None, cpu.pc + 1) as u16;
    let most_significant = cpu.bus.read_byte(None, cpu.pc + 2) as u16;

    // Perform Operation & Implicit Return
    goto_addr(cpu, (most_significant << 8) | least_significant, jump, true)
}

/*
[0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F]

*/
pub fn op_bit(cpu: &mut CPU, target: ByteTarget) -> u16 {
    let bit: u8;
    let target_register: u8;
    match target {
        // [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47]
        ByteTarget::Zero(hl_target) => {
            bit = 0b00000010; // Byte to match
            target_register = match_hl(cpu, &hl_target); // find target
        }
        // [0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F]
        ByteTarget::One(hl_target) => {
            bit = 0b00000100; // Byte to match
            target_register = match_hl(cpu, &hl_target); // find target
        }
        // [0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57]
        ByteTarget::Two(hl_target) => {
            bit = 0b00001000; // Byte to match
            target_register = match_hl(cpu, &hl_target); // find target
        }
        // [0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F]
        ByteTarget::Three(hl_target) => {
            bit = 0b00010000; // Byte to match
            target_register = match_hl(cpu, &hl_target); // find target
        }
        // [0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67]
        ByteTarget::Four(hl_target) => {
            bit = 0b00100000; // Byte to match
            target_register = match_hl(cpu, &hl_target); // find target
        }
        // [0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F]
        ByteTarget::Five(hl_target) => {
            bit = 0b01000000; // Byte to match
            target_register = match_hl(cpu, &hl_target); // find target
        }
        // [0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77]
        ByteTarget::Six(hl_target) => {
            bit = 0b10000000; // Byte to match
            target_register = match_hl(cpu, &hl_target); // find target
        }
        // [0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F]
        ByteTarget::Seven(hl_target) => {
            bit = 0b00000000; // Byte to match
            target_register = match_hl(cpu, &hl_target); // find target
        }
    }

    // Set Flags
    set_flags_after_bit(cpu, bit, target_register);

    // Prefixed Return
    cpu.pc.wrapping_add(2)
}

/*
[0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F,
 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF]
*/
pub fn op_res(cpu: &mut CPU, target: ByteTarget) -> u16 {
    let mask: u8;
    let target_register: u8;
    let is_mem: bool;
    let found_target: HLTarget;

    match target {
        // [0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87]
        ByteTarget::Zero(hl_target) => {
            mask = 0b11111110; // Byte Mask
            found_target = hl_target;
        }
        // [0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F]
        ByteTarget::One(hl_target) => {
            mask = 0b11111101; // Byte Mask
            found_target = hl_target;
        }
        // [0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97]
        ByteTarget::Two(hl_target) => {
            mask = 0b11111011; // Byte Mask
            found_target = hl_target;
        }
        // [0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F]
        ByteTarget::Three(hl_target) => {
            mask = 0b11110111; // Byte Mask
            found_target = hl_target;
        }
        // [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7]
        ByteTarget::Four(hl_target) => {
            mask = 0b11101111; // Byte Mask
            found_target = hl_target;
        }
        // [0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF]
        ByteTarget::Five(hl_target) => {
            mask = 0b11011111; // Byte Mask
            found_target = hl_target;
        }
        // [0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7]
        ByteTarget::Six(hl_target) => {
            mask = 0b10111111; // Byte Mask
            found_target = hl_target;
        }
        // [0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF]
        ByteTarget::Seven(hl_target) => {
            mask = 0b01111111; // Byte Mask
            found_target = hl_target;
        }
    }

    is_mem = matches!(found_target, HLTarget::HL);

    // Get Target Register
    target_register = match_hl(cpu, &found_target);

    // Perform Operation
    if is_mem {
        // if we're updating memory write back to grabbed location the new value
        cpu.bus
            .write_byte(None, cpu.registers.get_hl(), target_register & mask);
    } else {
        // Update the appropriate register based on found_target
        match found_target {
            HLTarget::A => cpu.registers.a &= mask,
            HLTarget::B => cpu.registers.b &= mask,
            HLTarget::C => cpu.registers.c &= mask,
            HLTarget::D => cpu.registers.d &= mask,
            HLTarget::E => cpu.registers.e &= mask,
            HLTarget::H => cpu.registers.h &= mask,
            HLTarget::L => cpu.registers.l &= mask,
            HLTarget::HL => {} // Already handled in is_mem case
        }
    }

    // Prefixed Return
    cpu.pc.wrapping_add(2)
}

/*
[0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF
 0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF
 0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF
 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF]
*/
pub fn op_set(cpu: &mut CPU, target: ByteTarget) -> u16 {
    let mask: u8;
    let is_mem: bool;
    let found_target: HLTarget;

    match target {
        // [0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7]
        ByteTarget::Zero(hl_target) => {
            mask = 0b00000001; // Byte Mask
            found_target = hl_target;
        }
        // [0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF]
        ByteTarget::One(hl_target) => {
            mask = 0b00000010;
            found_target = hl_target;
        }
        // [0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7]
        ByteTarget::Two(hl_target) => {
            mask = 0b00000100;
            found_target = hl_target;
        }
        // [0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF]
        ByteTarget::Three(hl_target) => {
            mask = 0b00001000;
            found_target = hl_target;
        }
        // [0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7]
        ByteTarget::Four(hl_target) => {
            mask = 0b00010000;
            found_target = hl_target;
        }
        // [0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF]
        ByteTarget::Five(hl_target) => {
            mask = 0b00100000;
            found_target = hl_target;
        }
        // [0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7]
        ByteTarget::Six(hl_target) => {
            mask = 0b01000000;
            found_target = hl_target;
        }
        // [0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF]
        ByteTarget::Seven(hl_target) => {
            mask = 0b10000000;
            found_target = hl_target;
        }
    }

    // Determine if we're using memory
    is_mem = matches!(found_target, HLTarget::HL);

    if is_mem {
        // If we're updating memory, read current value and set the bit
        let value = cpu.bus.read_byte(None, cpu.registers.get_hl());
        cpu.bus.write_byte(None, cpu.registers.get_hl(), value | mask);
    } else {
        // Update the appropriate register based on found_target
        match found_target {
            HLTarget::A => cpu.registers.a |= mask,
            HLTarget::B => cpu.registers.b |= mask,
            HLTarget::C => cpu.registers.c |= mask,
            HLTarget::D => cpu.registers.d |= mask,
            HLTarget::E => cpu.registers.e |= mask,
            HLTarget::H => cpu.registers.h |= mask,
            HLTarget::L => cpu.registers.l |= mask,
            HLTarget::HL => {} // Already handled in is_mem case
        }
    }

    // Prefixed Return
    cpu.pc.wrapping_add(2)
}

// [0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xFE]
pub fn op_cp(cpu: &mut CPU, target: OPTarget) -> u16 {
    match target {
        // [0xB8]
        OPTarget::B => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.b);

            cpu.pc.wrapping_add(1)
        }
        // [0xB9]
        OPTarget::C => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.c);

            cpu.pc.wrapping_add(1)
        }
        // [0xBA]
        OPTarget::D => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.d);

            cpu.pc.wrapping_add(1)
        }
        // [0xBB]
        OPTarget::E => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.e);

            cpu.pc.wrapping_add(1)
        }
        // [0xBC]
        OPTarget::H => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.h);

            cpu.pc.wrapping_add(1)
        }
        // [0xBD]
        OPTarget::L => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.l);

            cpu.pc.wrapping_add(1)
        }
        // [0xBE]
        OPTarget::HL => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.get_hl() as u8);

            cpu.pc.wrapping_add(3)
        }
        // [0xBF]
        OPTarget::A => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.a);
            cpu.pc.wrapping_add(1)
        }
        // [0xFE]
        OPTarget::D8 => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.bus.read_byte(None, cpu.pc + 1));
            cpu.pc.wrapping_add(2)
        }
    }
}

// [0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xF6]
pub fn op_or(cpu: &mut CPU, target: OPTarget) -> u16 {
    let result_pc: u16;
    match target {
        // [0xB0]
        OPTarget::B => {
            // OR
            cpu.registers.a |= cpu.registers.b;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xB1]
        OPTarget::C => {
            // OR
            cpu.registers.a |= cpu.registers.c;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xB2]
        OPTarget::D => {
            // OR
            cpu.registers.a |= cpu.registers.d;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xB3]
        OPTarget::E => {
            // OR
            cpu.registers.a |= cpu.registers.e;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xB4]
        OPTarget::H => {
            // OR
            cpu.registers.a |= cpu.registers.h;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xB5]
        OPTarget::L => {
            // OR
            cpu.registers.a |= cpu.registers.l;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xB6]
        OPTarget::HL => {
            // OR
            cpu.registers.a |= cpu.bus.read_byte(None, cpu.registers.get_hl());

            result_pc = cpu.pc.wrapping_add(3);
        }
        // [0xB7]
        OPTarget::A => {
            // OR
            cpu.registers.a |= cpu.registers.a;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xF6]
        OPTarget::D8 => {
            // OR
            cpu.registers.a = cpu.bus.read_byte(None, cpu.pc + 1);

            result_pc = cpu.pc.wrapping_add(2);
        }
    }
    // Set Flags
    set_flags_after_xor_or(cpu, cpu.registers.a);

    
    result_pc
}

// [0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF, 0xEE]
pub fn op_xor(cpu: &mut CPU, target: OPTarget) -> u16 {
    let result_pc: u16;
    match target {
        // [0xA8]
        OPTarget::B => {
            // XOR
            cpu.registers.a ^= cpu.registers.b;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xA9]
        OPTarget::C => {
            // XOR
            cpu.registers.a ^= cpu.registers.c;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xAA]
        OPTarget::D => {
            // XOR
            cpu.registers.a ^= cpu.registers.d;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xAB]
        OPTarget::E => {
            // XOR
            cpu.registers.a ^= cpu.registers.e;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xAC]
        OPTarget::H => {
            // XOR
            cpu.registers.a ^= cpu.registers.h;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xAD]
        OPTarget::L => {
            // XOR
            cpu.registers.a ^= cpu.registers.l;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xAE]
        OPTarget::HL => {
            // XOR
            cpu.registers.a ^= cpu.bus.read_byte(None, cpu.registers.get_hl());

            result_pc = cpu.pc.wrapping_add(3);
        }
        // [0xAF]
        OPTarget::A => {
            // XOR
            cpu.registers.a ^= cpu.registers.a;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xEE]
        OPTarget::D8 => {
            // XOR
            cpu.registers.a ^= cpu.bus.read_byte(None, cpu.pc + 1);

            result_pc = cpu.pc.wrapping_add(2);
        }
    }
    // Set Flags
    set_flags_after_xor_or(cpu, cpu.registers.a);

    
    result_pc
}

// [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xE6]
pub fn op_and(cpu: &mut CPU, target: OPTarget) -> u16 {
    let result_pc: u16;
    match target {
        // [0xA0]
        OPTarget::B => {
            // AND
            cpu.registers.a &= cpu.registers.b;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xA1]
        OPTarget::C => {
            // AND
            cpu.registers.a &= cpu.registers.c;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xA2]
        OPTarget::D => {
            // AND
            cpu.registers.a &= cpu.registers.d;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xA3]
        OPTarget::E => {
            // AND
            cpu.registers.a &= cpu.registers.e;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xA4]
        OPTarget::H => {
            // AND
            cpu.registers.a &= cpu.registers.h;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xA5]
        OPTarget::L => {
            // AND
            cpu.registers.a &= cpu.registers.l;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xA6]
        OPTarget::HL => {
            // AND
            cpu.registers.a &= cpu.bus.read_byte(None, cpu.registers.get_hl());

            result_pc = cpu.pc.wrapping_add(3);
        }
        // [0xA7]
        OPTarget::A => {
            // AND
            cpu.registers.a &= cpu.registers.a;

            result_pc = cpu.pc.wrapping_add(1);
        }
        // [0xE6]
        OPTarget::D8 => {
            // AND
            cpu.registers.a &= cpu.bus.read_byte(None, cpu.pc + 1);

            result_pc = cpu.pc.wrapping_add(2);
        }
    }
    // Set Flags
    set_flags_after_and(cpu, cpu.registers.a);

    
    result_pc
}

// [0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0xDE]
pub fn op_sbc(cpu: &mut CPU, target: OPTarget) -> u16 {
    let original_value = cpu.registers.a;
    match target {
        // [0x98]
        OPTarget::B => {
            // SBC
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.registers.b)
                .wrapping_sub(cpu.registers.f.carry as u8);

            // Set Flags -> use sub logic?
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.b);

            cpu.pc.wrapping_add(1)
        }
        // [0x99]
        OPTarget::C => {
            // SBC
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.registers.c)
                .wrapping_sub(cpu.registers.f.carry as u8);

            // Set Flags -> use sub logic?
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.c);

            cpu.pc.wrapping_add(1)
        }
        // [0x9A]
        OPTarget::D => {
            let immediate_operand = cpu.registers.d;
            let carry_value = cpu.registers.f.carry as u8;

            // Perform SBC: A = A - D - carry
            let result = original_value
                .wrapping_sub(immediate_operand)
                .wrapping_sub(carry_value);

            cpu.registers.a = result;

            // Set Flags -> use sub logic?
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.d);

            // Increment the program counter
            cpu.pc.wrapping_add(1)
        }
        // [0x9B]
        OPTarget::E => {
            // SBC
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.registers.e)
                .wrapping_sub(cpu.registers.f.carry as u8);

            // Set Flags -> use sub logic?
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.e);

            cpu.pc.wrapping_add(1)
        }
        // [0x9C]
        OPTarget::H => {
            // SBC
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.registers.h)
                .wrapping_sub(cpu.registers.f.carry as u8);

            // Set Flags -> use sub logic?
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.h);

            cpu.pc.wrapping_add(1)
        }
        // [0x9D]
        OPTarget::L => {
            // SBC
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.registers.l)
                .wrapping_sub(cpu.registers.f.carry as u8);

            // Set Flags -> use sub logic?
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.l);

            cpu.pc.wrapping_add(1)
        }
        // [0x9E]
        OPTarget::HL => {
            // SBC
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.bus.read_byte(None, cpu.registers.get_hl()))
                .wrapping_sub(cpu.registers.f.carry as u8);

            // Set Flags -> use sub logic?
            set_flags_after_sub(
                cpu,
                cpu.registers.a,
                original_value,
                cpu.registers.get_hl() as u8,
            );

            cpu.pc.wrapping_add(3)
        }
        // [0x9F]
        OPTarget::A => {
            // SBC
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.registers.a)
                .wrapping_sub(cpu.registers.f.carry as u8);

            // Set Flags -> use sub logic?
            set_flags_after_sub(cpu, cpu.registers.a, original_value, original_value);

            cpu.pc.wrapping_add(1)
        }
        // [0xDE]
        OPTarget::D8 => {
            // SBC
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.bus.read_byte(None, cpu.pc + 1))
                .wrapping_sub(cpu.registers.f.carry as u8);

            // Set Flags -> use sub logic?
            set_flags_after_sub(
                cpu,
                cpu.registers.a,
                original_value,
                cpu.bus.read_byte(None, cpu.pc + 1),
            );

            cpu.pc.wrapping_add(2)
        }
    }
}

// [0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0xD6]
pub fn op_sub(cpu: &mut CPU, target: OPTarget) -> u16 {
    // Get Original Value
    let original_value = cpu.registers.a;
    match target {
        // [0x90]
        OPTarget::B => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.b);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.b);

            cpu.pc.wrapping_add(1)
        }
        // [0x91]
        OPTarget::C => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.c);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.c);

            cpu.pc.wrapping_add(1)
        }
        // [0x92]
        OPTarget::D => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.d);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.d);

            cpu.pc.wrapping_add(1)
        }
        // [0x93]
        OPTarget::E => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.e);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.e);

            cpu.pc.wrapping_add(1)
        }
        // [0x94]
        OPTarget::H => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.h);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.h);

            cpu.pc.wrapping_add(1)
        }
        // [0x95]
        OPTarget::L => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.l);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.l);

            cpu.pc.wrapping_add(1)
        }
        // [0x96]
        OPTarget::HL => {
            // SUB
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.bus.read_byte(None, cpu.registers.get_hl()));

            // Set Flags
            set_flags_after_sub(
                cpu,
                cpu.registers.a,
                original_value,
                cpu.bus.read_byte(None, cpu.registers.get_hl()),
            );
            cpu.pc.wrapping_add(3)
        }
        // [0x97]
        OPTarget::A => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.a);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, original_value);

            cpu.pc.wrapping_add(1)
        }
        // [0xD6]
        OPTarget::D8 => {
            // SUB
            cpu.registers.a = cpu
                .registers
                .a
                .wrapping_sub(cpu.bus.read_byte(None, cpu.pc + 1));

            // Set Flags
            set_flags_after_sub(
                cpu,
                cpu.registers.a,
                original_value,
                cpu.bus.read_byte(None, cpu.pc + 1),
            );
            cpu.pc.wrapping_add(2)
        }
    }
}

// [0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F, 0xCE]
pub fn op_adc(cpu: &mut CPU, target: OPTarget) -> u16 {
    match target {
        // [0x88]
        OPTarget::B => {
            let original_value = cpu.registers.a; // Store Original Value
            cpu.registers.a = cpu.registers.b.wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.b); // Set Flags
            cpu.pc.wrapping_add(1)
        }
        // [0x89]
        OPTarget::C => {
            let original_value = cpu.registers.a; // Store Original Value
            cpu.registers.a = cpu.registers.c.wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.c); // Set Flags
            cpu.pc.wrapping_add(1)
        }
        // [0x8A]
        OPTarget::E => {
            let original_value = cpu.registers.a; // Store Original Value
            cpu.registers.a = cpu.registers.e.wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.e); // Set Flags
            cpu.pc.wrapping_add(1)
        }
        // [0x8B]
        OPTarget::D => {
            let original_value = cpu.registers.a; // Store Original Value
            cpu.registers.a = cpu.registers.d.wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.d); // Set Flags
            cpu.pc.wrapping_add(1)
        }
        // [0x8C]
        OPTarget::H => {
            let original_value = cpu.registers.a; // Store Original Value
            cpu.registers.a = cpu.registers.h.wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.h); // Set Flags
            cpu.pc.wrapping_add(1)
        }
        // [0x8D]
        OPTarget::L => {
            let original_value = cpu.registers.a; // Store Original Value
            cpu.registers.a = cpu.registers.l.wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.l); // Set Flags
            cpu.pc.wrapping_add(1)
        }
        // [0x8E]
        OPTarget::HL => {
            let original_value = cpu.registers.a; // Store Original Value
            cpu.registers.a = cpu
                .bus
                .read_byte(None, cpu.registers.get_hl())
                .wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(
                cpu,
                cpu.registers.a,
                original_value,
                cpu.bus.read_byte(None, cpu.registers.get_hl()),
            ); // Set Flags
            cpu.pc.wrapping_add(1)
        }
        // [0x8E]
        OPTarget::A => {
            let original_value = cpu.registers.a; // Store Original Value
            cpu.registers.a = cpu.registers.a.wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(cpu, cpu.registers.a, original_value, original_value); // Set Flags
            cpu.pc.wrapping_add(1)
        }
        // [0xCE]
        OPTarget::D8 => {
            let original_value = cpu.registers.a; // Store Original Values
            cpu.registers.a = cpu
                .bus
                .read_byte(None, cpu.pc + 1)
                .wrapping_add(cpu.registers.f.carry as u8); // ADC
            set_flags_after_adc(
                cpu,
                cpu.registers.a,
                original_value,
                cpu.bus.read_byte(None, cpu.pc + 1),
            ); // Set Flags
            cpu.pc.wrapping_add(2)
        }
    }
}

// [0x09, 0x19, 0x29, 0x39,]
pub fn op_add(cpu: &mut CPU, target: OPType) -> u16 {
    match target {
        // [0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87]
        OPType::LoadA(target) => {
            // Find Register Target
            let reg_target = match_hl(cpu, &target);

            // Store the original value of A
            let original = cpu.registers.a;

            // ADD
            cpu.registers.a = original.wrapping_add(reg_target);

            // Set Flags
            set_flags_after_add_a(cpu, reg_target, original, false);

            cpu.pc.wrapping_add(1)
        }
        // [0x09, 0x19, 0x29, 0x39]
        OPType::LoadHL(target) => {
            // Find Register Target
            let reg_target = match_n16(cpu, target);

            // ADD
            cpu.registers
                .set_hl(cpu.registers.get_hl().wrapping_add(reg_target));

            // Set Flags [- 0 H CY]
            set_flags_after_add_n16(cpu, reg_target);

            cpu.pc.wrapping_add(1)
        }
        // [0xE8]
        OPType::LoadSP => {
            // Find and Sign-extend the immediate operand to 16 bits
            let signed_value = (cpu.bus.read_byte(None, cpu.pc + 1) as i8) as i16;

            // ADD
            cpu.sp = cpu.sp.wrapping_add(signed_value as u16);

            // Set Flags
            set_flags_after_add_sp(cpu, signed_value);

            cpu.pc.wrapping_add(2)
        }
        // [0xC6]
        OPType::LoadD8 => {
            // Get Immediate Operand and Store Original A Value
            let immediate_operand: u8 = cpu.bus.read_byte(None, cpu.pc + 1);
            let original = cpu.registers.a;

            // ADD
            cpu.registers.a = cpu.registers.a.wrapping_add(immediate_operand);

            // Set Flags
            set_flags_after_add_a(cpu, immediate_operand, original, true);

            cpu.pc.wrapping_add(2)
        }
    }
}

/*
[0x01, 0x11, 0x21, 0x31,
 0x02, 0x12, 0x22, 0x32,
 0x06, 0x16, 0x26, 0x36,
 0x0A, 0x1A, 0x2A, 0x3A,
 0x0E, 0x1E, 0x2E, 0x3E
 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57,
 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77,
 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F
 0xE0, 0xF0, 0xE2, 0xF2, 0x08, 0xF8, 0xF9, 0xEA, 0xFA]
*/
pub fn op_ld(cpu: &mut CPU, target: LoadType) -> u16 {
    match target {
        LoadType::RegInReg(target, source) => match target {
            // [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47]
            HLTarget::B => match source {
                // [0x40]
                HLTarget::B => {
                    cpu.registers.b = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                // [0x41]
                HLTarget::C => {
                    cpu.registers.b = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                // [0x42]
                HLTarget::D => {
                    cpu.registers.b = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                // [0x43]
                HLTarget::E => {
                    cpu.registers.b = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                // [0x44]
                HLTarget::H => {
                    cpu.registers.b = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                // [0x45]
                HLTarget::L => {
                    cpu.registers.b = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                // [0x46]
                HLTarget::HL => {
                    cpu.registers.b = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                // 0x47
                HLTarget::A => {
                    cpu.registers.b = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            // [0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F]
            HLTarget::C => match target {
                // [0x48]
                HLTarget::B => {
                    cpu.registers.c = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                // [0x49]
                HLTarget::C => {
                    cpu.registers.c = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                // [0x4A]
                HLTarget::D => {
                    cpu.registers.c = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                // [0x4B]
                HLTarget::E => {
                    cpu.registers.c = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                // [0x4C]
                HLTarget::H => {
                    cpu.registers.c = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                // [0x4D]
                HLTarget::L => {
                    cpu.registers.c = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                // [0x4E]
                HLTarget::HL => {
                    cpu.registers.c = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                // [0x4F]
                HLTarget::A => {
                    cpu.registers.c = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            // [0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57]
            HLTarget::D => match target {
                // [0x50]
                HLTarget::B => {
                    cpu.registers.d = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                // [0x51]
                HLTarget::C => {
                    cpu.registers.d = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                // [0x52]
                HLTarget::D => {
                    cpu.registers.d = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                // [0x53]
                HLTarget::E => {
                    cpu.registers.d = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                // [0x54]
                HLTarget::H => {
                    cpu.registers.d = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                // [0x55]
                HLTarget::L => {
                    cpu.registers.d = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                // [0x56]
                HLTarget::HL => {
                    cpu.registers.d = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                // [0x57]
                HLTarget::A => {
                    cpu.registers.d = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            // [0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F]
            HLTarget::E => match target {
                // [0x58]
                HLTarget::B => {
                    cpu.registers.e = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                // [0x59]
                HLTarget::C => {
                    cpu.registers.e = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                // [0x5A]
                HLTarget::D => {
                    cpu.registers.e = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                // [0x5B]
                HLTarget::E => {
                    cpu.registers.e = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                // [0x5C]
                HLTarget::H => {
                    cpu.registers.e = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                // [0x5D]
                HLTarget::L => {
                    cpu.registers.e = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                // [0x5E]
                HLTarget::HL => {
                    cpu.registers.e = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                // [0x5F]
                HLTarget::A => {
                    cpu.registers.e = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            // [0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67]
            HLTarget::H => match target {
                // [0x60]
                HLTarget::B => {
                    cpu.registers.h = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                // [0x61]
                HLTarget::C => {
                    cpu.registers.h = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                // [0x62]
                HLTarget::D => {
                    cpu.registers.h = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                // [0x63]
                HLTarget::E => {
                    cpu.registers.h = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                // [0x64]
                HLTarget::H => {
                    cpu.registers.h = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                // [0x65]
                HLTarget::L => {
                    cpu.registers.h = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                // [0x66]
                HLTarget::HL => {
                    cpu.registers.h = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                // [0x67]
                HLTarget::A => {
                    cpu.registers.h = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            // [0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F]
            HLTarget::L => match target {
                // [0x68]
                HLTarget::B => {
                    cpu.registers.l = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                // [0x69]
                HLTarget::C => {
                    cpu.registers.l = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                // [0x6A]
                HLTarget::D => {
                    cpu.registers.l = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                // [0x6B]
                HLTarget::E => {
                    cpu.registers.l = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                // [0x6C]
                HLTarget::H => {
                    cpu.registers.l = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                // [0x6D]
                HLTarget::L => {
                    cpu.registers.l = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                // [0x6E]
                HLTarget::HL => {
                    cpu.registers.l = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                // [0x6F]
                HLTarget::A => {
                    cpu.registers.l = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            // [0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77]
            HLTarget::HL => match target {
                // [0x70]
                HLTarget::B => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.b);
                    cpu.pc.wrapping_add(1)
                }
                // [0x71]
                HLTarget::C => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.c);
                    cpu.pc.wrapping_add(1)
                }
                // [0x72]
                HLTarget::D => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.d);
                    cpu.pc.wrapping_add(1)
                }
                // [0x73]
                HLTarget::E => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.e);
                    cpu.pc.wrapping_add(1)
                }
                // [0x74]
                HLTarget::H => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.h);
                    cpu.pc.wrapping_add(1)
                }
                // [0x75]
                HLTarget::L => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.l);
                    cpu.pc.wrapping_add(1)
                }
                // [0x77]
                HLTarget::A => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.a);
                    cpu.pc.wrapping_add(1)
                }
                _ => panic!("Getting LD HL HL Should be HALT"),
            },
            // [0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F]
            HLTarget::A => match target {
                // [0x78]
                HLTarget::B => {
                    cpu.registers.a = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                // [0x79]
                HLTarget::C => {
                    cpu.registers.a = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                // [0x7A]
                HLTarget::D => {
                    cpu.registers.a = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                // [0x7B]
                HLTarget::E => {
                    cpu.registers.a = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                // [0x7C]
                HLTarget::H => {
                    cpu.registers.a = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                // [0x7D]
                HLTarget::L => {
                    cpu.registers.a = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                // [0x7E]
                HLTarget::HL => {
                    cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                // [0x7F]
                HLTarget::A => {
                    cpu.registers.a = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
        },
        // [0x01, 0x21, 0xF8, 0x11, 0x08]
        LoadType::Word(target, source) => {
            // Read the next two bytes from bus at the current PC
            let low_byte = cpu.bus.read_byte(None, cpu.pc + 1); // Read the low byte
            let high_byte = cpu.bus.read_byte(None, cpu.pc + 2); // Read the high byte

            // Combine the low and high bytes into a 16-bit value
            let word_value = ((high_byte as u16) << 8) | (low_byte as u16);

            match target {
                // [0x01]
                LoadWordTarget::BC => match source {
                    LoadWordSource::N16 => {
                        cpu.registers.set_bc(word_value as u16);
                        cpu.pc.wrapping_add(3)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
                // [0x21, 0xF8]
                LoadWordTarget::HL => match source {
                    // [0x21]
                    LoadWordSource::N16 => {
                        cpu.registers.set_hl(word_value as u16);

                        cpu.pc.wrapping_add(3)
                    }
                    // [0xF8]
                    LoadWordSource::SPE8 => {
                        cpu.registers.set_hl(
                            ((cpu.sp as i16)
                                .wrapping_add((cpu.bus.read_byte(None, cpu.pc + 1) as i8) as i16))
                                as u16,
                        );
                        // Set Flags
                        set_flags_after_ld_spe8(cpu);

                        cpu.pc.wrapping_add(2)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
                // [0x11]
                LoadWordTarget::DE => match source {
                    LoadWordSource::N16 => {
                        cpu.registers.set_de(word_value as u16);
                        cpu.pc.wrapping_add(3)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
                // [0x08]
                LoadWordTarget::N16 => match source {
                    LoadWordSource::SP => {
                        cpu.bus
                            .write_byte(None, word_value, (cpu.sp & 0x00FF) as u8);
                        cpu.bus
                            .write_byte(None, word_value + 1, (cpu.sp >> 8) as u8);
                        cpu.pc.wrapping_add(3)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
                // [0x31, 0xF9]
                LoadWordTarget::SP => match source {
                    // [0xF9]
                    LoadWordSource::HL => {
                        cpu.registers.set_hl(cpu.sp);
                        cpu.pc.wrapping_add(1)
                    }
                    // [0x31]
                    LoadWordSource::N16 => {
                        cpu.sp = word_value;
                        cpu.pc.wrapping_add(3)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
            }
        }
        // [0x0A, 0x1A, 0x2A, 0x3A]
        LoadType::AStoreInN16(target) => match target {
            // [0x0A]
            LoadN16::BC => {
                cpu.bus
                    .write_byte(None, cpu.registers.get_bc(), cpu.registers.a);
                cpu.pc.wrapping_add(1)
            }
            // [0x1A]
            LoadN16::DE => {
                cpu.bus
                    .write_byte(None, cpu.registers.get_de(), cpu.registers.a);
                cpu.pc.wrapping_add(1)
            }
            // [0x2A]
            LoadN16::HLDEC => {
                cpu.bus
                    .write_byte(None, cpu.registers.get_hl(), cpu.registers.a);
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1));
                cpu.pc.wrapping_add(1)
            }
            // [0x3A]
            LoadN16::HLINC => {
                cpu.bus
                    .write_byte(None, cpu.registers.get_hl(), cpu.registers.a);
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(1));
                cpu.pc.wrapping_add(1)
            }
        },
        // [0x02, 0x12, 0x22, 0x32]
        LoadType::N16StoreInA(source) => match source {
            // [0x02]
            LoadN16::BC => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_bc());
                cpu.pc.wrapping_add(1)
            }
            // [0x12]
            LoadN16::DE => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_de());
                cpu.pc.wrapping_add(1)
            }
            // [0x22]
            LoadN16::HLDEC => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_hl());
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1));
                cpu.pc.wrapping_add(1)
            }
            // [0x32]
            LoadN16::HLINC => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_hl());
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(1));
                cpu.pc.wrapping_add(1)
            }
            
        },
        // [0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E]
        LoadType::D8StoreInReg(target) => match target {
            // [0x06]
            HLTarget::B => {
                cpu.registers.b = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            // [0x0E]
            HLTarget::C => {
                cpu.registers.c = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            // [0x16]
            HLTarget::D => {
                cpu.registers.d = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            // [0x1E]
            HLTarget::E => {
                cpu.registers.e = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            // [0x26]
            HLTarget::H => {
                cpu.registers.h = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            // [0x2E]
            HLTarget::L => {
                cpu.registers.l = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            // [0x36]
            HLTarget::HL => {
                cpu.bus.write_byte(
                    None,
                    cpu.registers.get_hl(),
                    cpu.bus.read_byte(None, cpu.pc + 1),
                );
                cpu.pc.wrapping_add(2)
            }
            // [0x3E]
            HLTarget::A => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
        },
        // [0xE0, 0xF0]
        LoadType::AWithA8(target) => match target {
            // [0xF0]
            LoadA8Target::A => {
                // First read all values we need
                let address = 0xFF00 + cpu.bus.read_byte(None, cpu.pc + 1) as u16;

                // Then read the value at the calculated address
                // We create a temporary mutable reference to cpu for the read_byte call
                let value = {
                    let cpu_ref = cpu as *mut CPU;
                    // SAFETY: We're only creating a temporary reference and not modifying any state
                    // The CPU reference is valid for the duration of this scope
                    // We ensure no other mutable references exist during this time
                    cpu.bus.read_byte(Some(unsafe { &mut *cpu_ref }), address)
                };

                // Finally update register and return
                cpu.registers.a = value;
                cpu.pc.wrapping_add(2)
            }
            // [0xE0]
            LoadA8Target::A8 => {
                // First read all values we need
                let address = 0xFF00 + cpu.bus.read_byte(None, cpu.pc + 1) as u16;
                let value = cpu.registers.a;

                // Create a temporary mutable reference for the write operation
                {
                    let cpu_ref = cpu as *mut CPU;
                    // SAFETY: We're only creating a temporary reference and not modifying any state
                    // The CPU reference is valid for the duration of this scope
                    // We ensure no other mutable references exist during this time
                    cpu.bus
                        .write_byte(Some(unsafe { &mut *cpu_ref }), address, value);
                }

                // Return the new PC
                cpu.pc.wrapping_add(2)
            }
        },
        // [0xEA, 0xFA]
        LoadType::AWithA16(target) => {
            let low_byte = cpu.bus.read_byte(None, cpu.pc + 1); // Read the low byte
            let high_byte = cpu.bus.read_byte(None, cpu.pc + 2); // Read the high byte

            // Combine the low and high bytes into a 16-bit value
            let address = ((high_byte as u16) << 8) | (low_byte as u16);

            match target {
                // [0xFA]
                LoadA16Target::A => {
                    cpu.registers.a = cpu.bus.read_byte(None, address);
                    cpu.pc.wrapping_add(3)
                }
                // [0xEA]
                LoadA16Target::A16 => {
                    cpu.bus.write_byte(None, address, cpu.registers.a);
                    cpu.pc.wrapping_add(3)
                }
            }
        }
        // [0xE2, 0xF2]
        LoadType::AWithAC(target) => match target {
            // [0xF2]
            LoadACTarget::A => {
                cpu.bus
                    .write_byte(None, 0xFF00 + cpu.registers.c as u16, cpu.registers.a);
                cpu.pc.wrapping_add(2)
            }
            // [0xE2]
            LoadACTarget::C => {
                cpu.registers.a = cpu.bus.read_byte(None, 0xFF00 + cpu.registers.c as u16);
                cpu.pc.wrapping_add(2)
            }
        },
    }
}

// [0x05, 0x0B, 0x0D, 0x15, 0x1B, 0x1D, 0x25, 0x2B, 0x2D, 0x35, 0x3B, 0x3D]
pub fn op_dec(cpu: &mut CPU, target: AllRegisters) -> u16 {
    match target {
        // Increment 8-bit registers and Set Flags
        // [0x3D]
        AllRegisters::A => {
            let original_value = cpu.registers.a;
            cpu.registers.a = cpu.registers.a.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.a, original_value);
        }
        // [0x05]
        AllRegisters::B => {
            let original_value = cpu.registers.b;
            cpu.registers.b = cpu.registers.b.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.b, original_value);
        }
        // [0x0D]
        AllRegisters::C => {
            let original_value = cpu.registers.c;
            cpu.registers.c = cpu.registers.c.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.c, original_value);
        }
        // [0x15]
        AllRegisters::D => {
            let original_value = cpu.registers.d;
            cpu.registers.d = cpu.registers.d.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.d, original_value);
        }
        // [0x1D]
        AllRegisters::E => {
            let original_value = cpu.registers.e;
            cpu.registers.e = cpu.registers.e.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.e, original_value);
        }
        // [0x25]
        AllRegisters::H => {
            let original_value = cpu.registers.h;
            cpu.registers.h = cpu.registers.h.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.h, original_value);
        }
        // [0x2D]
        AllRegisters::L => {
            let original_value = cpu.registers.l;
            cpu.registers.l = cpu.registers.l.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.l, original_value);
        }

        // [0x35]
        AllRegisters::HLMEM => {
            // Increment value at bus location HL
            let hl_addr = cpu.registers.get_hl();
            let original_value = cpu.bus.read_byte(None, hl_addr);
            let value = cpu.bus.read_byte(None, hl_addr).wrapping_sub(1);
            cpu.bus.write_byte(None, hl_addr, value);
            set_flags_after_dec(cpu, value, original_value);
        }
        // 16-bit register increments (don't need to Set Flags for these)
        // [0x0B]
        AllRegisters::BC => {
            let new_bc = cpu.registers.get_bc().wrapping_sub(1);
            cpu.registers.set_bc(new_bc);
        }
        // [0x1B]
        AllRegisters::DE => {
            let new_de = cpu.registers.get_de().wrapping_sub(1);
            cpu.registers.set_de(new_de);
        }
        // [0x2B]
        AllRegisters::HL => {
            let new_hl = cpu.registers.get_hl().wrapping_sub(1);
            cpu.registers.set_hl(new_hl);
        }
        // [0x3B]
        AllRegisters::SP => {
            cpu.sp = cpu.sp.wrapping_sub(1);
        }
    }
    cpu.pc.wrapping_add(1)
}

// [0x03, 0x04, 0x0C, 0x13, 0x14, 0x1C, 0x23, 0x24, 0x2C, 0x33, 0x34, 0x3C]
pub fn op_inc(cpu: &mut CPU, target: AllRegisters) -> u16 {
    match target {
        // Increment 8-bit registers and Set Flags
        // [0x3C]
        AllRegisters::A => {
            cpu.registers.a = cpu.registers.a.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.a);
        }
        // [0x04]
        AllRegisters::B => {
            cpu.registers.b = cpu.registers.b.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.b);
        }
        // [0x0C]
        AllRegisters::C => {
            cpu.registers.c = cpu.registers.c.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.c);
        }
        // [0x14]
        AllRegisters::D => {
            cpu.registers.d = cpu.registers.d.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.d);
        }
        // [0x1C]
        AllRegisters::E => {
            cpu.registers.e = cpu.registers.e.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.e);
        }
        // [0x24]
        AllRegisters::H => {
            cpu.registers.h = cpu.registers.h.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.h);
        }
        // [0x2C]
        AllRegisters::L => {
            cpu.registers.l = cpu.registers.l.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.l);
        }
        // [0x34]
        AllRegisters::HLMEM => {
            // Increment value at bus location HL
            let hl_addr = cpu.registers.get_hl();
            let value = cpu.bus.read_byte(None, hl_addr).wrapping_add(1);
            cpu.bus.write_byte(None, hl_addr, value);
            set_flags_after_inc(cpu, value);
        }
        // 16-bit register increments (don't need to Set Flags for these)
        // [0x03]
        AllRegisters::BC => {
            let new_bc = cpu.registers.get_bc().wrapping_add(1);
            cpu.registers.set_bc(new_bc);
        }
        // [0x13]
        AllRegisters::DE => {
            let new_de = cpu.registers.get_de().wrapping_add(1);
            cpu.registers.set_de(new_de);
        }
        // [0x23]
        AllRegisters::HL => {
            let new_hl = cpu.registers.get_hl().wrapping_add(1);
            cpu.registers.set_hl(new_hl);
        }
        // [0x33]
        AllRegisters::SP => {
            cpu.sp = cpu.sp.wrapping_add(1);
        }
    }
    cpu.pc.wrapping_add(1)
}

// MAYBE CHANGE TO GOTO_ADDR IN FUTURE?
// [0x18, 0x20, 0x28, 0x30, 0x38]
pub fn op_jr(cpu: &mut CPU, target: JumpTest) -> u16 {
    let jump_distance = cpu.bus.read_byte(None, cpu.pc + 1) as i8;
    match target {
        // [0x20]
        JumpTest::NotZero => {
            if !cpu.registers.f.zero {
                cpu.pc = cpu.pc.wrapping_add(jump_distance as u16)
            }
        }
        // [0x30]
        JumpTest::NotCarry => {
            if !cpu.registers.f.carry {
                cpu.pc = cpu.pc.wrapping_add(jump_distance as u16)
            }
        }
        // [0x18]
        JumpTest::Always => cpu.pc = cpu.pc.wrapping_add(jump_distance as u16),
        // [0x28]
        JumpTest::Zero => {
            if cpu.registers.f.zero {
                cpu.pc = cpu.pc.wrapping_add(jump_distance as u16)
            }
        }
        // [0x38]
        JumpTest::Carry => {
            if cpu.registers.f.carry {
                cpu.pc = cpu.pc.wrapping_add(jump_distance as u16)
            }
        }
        JumpTest::HL => panic!("Invalid JumpTest::HL {:?} in JR instruction", target),
    }
    cpu.pc.wrapping_add(2)
}

// [0xC1, 0xD1, 0xE1, 0xF1]
pub fn op_pop(cpu: &mut CPU, target: StackTarget) -> u16 {
    // Pop Low and High Bytes
    let low: u16 = stack_pop(cpu) as u16;
    //Cycle
    let high: u16 = stack_pop(cpu) as u16;
    //Cycle

    // Combine Bytes
    let combined: u16 = (high << 8) | low;

    // Perform Operation
    match target {
        // [0xF1]
        StackTarget::AF => {
            cpu.registers.set_af(combined & 0xFFF0);
        }
        // [0xC1]
        StackTarget::BC => {
            cpu.registers.set_bc(combined);
        }
        // [0xD1]
        StackTarget::DE => {
            cpu.registers.set_de(combined);
        }
        // [0xE1]
        StackTarget::HL => {
            cpu.registers.set_hl(combined);
        }
    }

    cpu.pc.wrapping_add(1)
}

// [0xC5, 0xD5, 0xE5, 0xF5]
pub fn op_push(cpu: &mut CPU, target: StackTarget) -> u16 {
    match target {
        // [0xF5]
        StackTarget::AF => {
            let high: u16 = (cpu.registers.get_af() >> 8) & 0xFF as u16;
            // Cycle
            stack_push(cpu, high as u8);

            let low: u16 = cpu.registers.get_af() & 0xFF as u16;
            // Cycle
            stack_push(cpu, low as u8);

            // Cycle
        }
        // [0xC5]
        StackTarget::BC => {
            let high: u16 = (cpu.registers.get_bc() >> 8) & 0xFF as u16;
            // Cycle
            stack_push(cpu, high as u8);

            let low: u16 = cpu.registers.get_bc() & 0xFF as u16;
            // Cycle
            stack_push(cpu, low as u8);

            // Cycle
        }
        // [0xD5]
        StackTarget::DE => {
            let high: u16 = (cpu.registers.get_de() >> 8) & 0xFF as u16;
            // Cycle
            stack_push(cpu, high as u8);

            let low: u16 = cpu.registers.get_de() & 0xFF as u16;
            // Cycle
            stack_push(cpu, low as u8);

            // Cycle
        }
        // [0xE5]
        StackTarget::HL => {
            let high: u16 = (cpu.registers.get_hl() >> 8) & 0xFF as u16;
            // Cycle
            stack_push(cpu, high as u8);

            let low: u16 = cpu.registers.get_hl() & 0xFF as u16;
            // Cycle
            stack_push(cpu, low as u8);

            // Cycle
        }
    }
    cpu.pc.wrapping_add(1)
}

// [0xC0, 0xD0, 0xD8, 0xC8, 0xC9]
pub fn op_ret(cpu: &mut CPU, target: JumpTest) -> u16 {
    // Maybe Cycle Here?? RESEARCH

    // Get Condition
    let jump = match_jump(cpu, target);

    if jump {
        let low: u16 = stack_pop(cpu) as u16;
        // Cycle
        let high: u16 = stack_pop(cpu) as u16;
        // Cycle

        cpu.pc = (high << 8) | low;

        
        cpu.pc
    } else {
        cpu.pc.wrapping_add(3) // maybe not correct
    }
}

// [0xD9]
pub fn op_reti(cpu: &mut CPU) -> u16 {
    // Update Interrupt
    cpu.master_enabled = true;

    // Call RET Logic w Always so it executes
    op_ret(cpu, JumpTest::Always)
}

// [0xC7, 0xD7, 0xE7, 0xF7, 0xFC, 0xFD, 0xFE, 0xFF]
pub fn op_rst(cpu: &mut CPU, target: RestTarget) -> u16 {
    let low: u16 = match target {
        RestTarget::Zero => 0x00,
        RestTarget::One => 0x08,
        RestTarget::Two => 0x10,
        RestTarget::Three => 0x18,
        RestTarget::Four => 0x20,
        RestTarget::Five => 0x28,
        RestTarget::Six => 0x30,
        RestTarget::Seven => 0x38,
    };

    // Perform Operation & Implicit Return
    goto_addr(cpu, 0x0000 | low, true, true)
}