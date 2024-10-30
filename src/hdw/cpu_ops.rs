/*

    Helper File to Contain Helper Utilization Functions For CPU Execute Operations

*/
use crate::hdw::cpu::*;
use crate::hdw::cpu_util::*;
use crate::hdw::instructions::*;
use crate::hdw::stack::*;

pub fn op_srl(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, target);

    // Get LSB For Carry Flag
    let lsb = reg_target & 0x1;

    // Shift Right
    reg_target = reg_target >> 1;

    // Update Flags
    cpu.registers.f.carry = lsb != 0;
    cpu.registers.f.zero = reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_swap(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, target);

    // Swap the the nibbles
    reg_target = (reg_target << 4) | (reg_target >> 4);

    // Upd Flags
    cpu.registers.f.zero = reg_target == 0;
    cpu.registers.f.carry = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_sra(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, target);

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
    cpu.registers.f.carry = lsb != 0;
    cpu.registers.f.zero = reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_sla(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, target);

    // Get Bit 7 For Carry
    let bit_7 = (reg_target & 0x80) != 0;

    // Shift Left
    reg_target <<= 1;

    // Update Flag
    cpu.registers.f.carry = bit_7;
    cpu.registers.f.zero = reg_target == 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_rlc(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, target);

    // Get Bit 7 For Carry
    let bit_7 = (reg_target >> 7) & 0x1;

    // Rotate Left With Carry
    reg_target = (reg_target << 1) | bit_7;

    // Update Flags
    set_flags_after_pref_op(cpu, bit_7, reg_target);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_rrc(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find target Register
    let mut reg_target = match_hl(cpu, target);

    // Get Bit 0 For Carry
    let bit_0 = reg_target & 0x1;

    // Rotate Right and Append bit 0
    reg_target = (reg_target >> 1) | (bit_0 >> 7);

    // Update Flags
    set_flags_after_pref_op(cpu, bit_0, reg_target);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_rl(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, target);

    // Store Previous Carry
    let prev_carry = cpu.registers.f.carry;

    // Store Bit 7 For Carry
    let bit_7 = (reg_target >> 7) & 0x1;

    // Rotate Left and Append
    reg_target = (reg_target << 1) | (prev_carry as u8);

    // Update Flags
    set_flags_after_pref_op(cpu, bit_7, reg_target);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_rr(cpu: &mut CPU, target: HLTarget) -> u16 {
    // Find Target Register
    let mut reg_target = match_hl(cpu, target);

    // Store Previous Carry
    let prev_carry = cpu.registers.f.carry;

    // Store Bit 0
    let bit_0 = reg_target & 0x1;

    // Rotate Right and append bit 0
    reg_target = (reg_target >> 1) | (prev_carry as u8) << 7;

    // Update Flags
    set_flags_after_pref_op(cpu, bit_0, reg_target);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_cpl(cpu: &mut CPU) -> u16 {
    // Flip all bits of register A
    cpu.registers.a = !cpu.registers.a;

    // Set flags
    set_flags_after_cpl(cpu);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

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

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_rra(cpu: &mut CPU) -> u16 {
    // Store the original bit 0 to set the carry flag
    let bit_0 = cpu.registers.a & 1;

    // Rotate right: shift right by 1 and add carry to bit 7
    cpu.registers.a = (cpu.registers.a >> 1) | (cpu.registers.f.carry as u8) << 7;

    // Update Flags
    set_flags_after_no_pre_rl_rr(cpu, bit_0);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_rla(cpu: &mut CPU) -> u16 {
    // Store the original bit 7 to set the carry flag
    let bit_7 = (cpu.registers.a & 0x80) >> 7;

    // Rotate left: shift left by 1 and add carry to bit 0
    cpu.registers.a = (cpu.registers.a << 1) | (cpu.registers.f.carry as u8);

    // Update Flags
    set_flags_after_no_pre_rl_rr(cpu, bit_7);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_rrca(cpu: &mut CPU) -> u16 {
    // Store the original bit 0 to set the carry flag and bit 7
    let bit_0 = cpu.registers.a & 1;

    // Rotate right: shift right by 1 and add bit 0 to bit 7
    cpu.registers.a = (cpu.registers.a >> 1) | (bit_0 << 7);

    // Update Flags
    set_flags_after_no_pre_rl_rr(cpu, bit_0);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_rlca(cpu: &mut CPU) -> u16 {
    // Store the original bit 7 to set the Carry flag and bit 0
    let bit_7 = (cpu.registers.a >> 7) & 1;

    // Rotate left: shift left by 1 and add bit 7 to bit 0
    cpu.registers.a = (cpu.registers.a << 1) | bit_7;

    // Update Flags
    set_flags_after_no_pre_rl_rr(cpu, bit_7);

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

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

// Jump to addr in bus or increment pc
pub fn op_call(cpu: &mut CPU, target: JumpTest) -> u16 {
    // Match Jump
    let jump = match_jump(cpu, target);

    // Get Bytes
    let least_significant = cpu.bus.read_byte(None, cpu.pc + 1) as u16;
    let most_significant = cpu.bus.read_byte(None, cpu.pc + 2) as u16;

    // Perform Operation & Implicit Return
    goto_addr(cpu, (most_significant << 8) | least_significant, jump, true)
}

pub fn op_bit(cpu: &mut CPU, target: ByteTarget) -> u16 {
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

    // Prefixed Return
    cpu.pc.wrapping_add(2)
}

pub fn op_res(cpu: &mut CPU, target: ByteTarget) -> u16 {
    let mask: u8;
    let mut target_register: u8;
    let mut is_mem: bool = false;
    let found_target: HLTarget;

    match target {
        ByteTarget::Zero(hl_target) => {
            mask = 0b11111110; // Byte Mask
            found_target = hl_target;
        }
        ByteTarget::One(hl_target) => {
            mask = 0b11111101; // Byte Mask
            found_target = hl_target;
        }
        ByteTarget::Two(hl_target) => {
            mask = 0b11111011; // Byte Mask
            found_target = hl_target;
        }
        ByteTarget::Three(hl_target) => {
            mask = 0b11110111; // Byte Mask
            found_target = hl_target;
        }
        ByteTarget::Four(hl_target) => {
            mask = 0b11101111; // Byte Mask
            found_target = hl_target;
        }
        ByteTarget::Five(hl_target) => {
            mask = 0b11011111; // Byte Mask
            found_target = hl_target;
        }
        ByteTarget::Six(hl_target) => {
            mask = 0b10111111; // Byte Mask
            found_target = hl_target;
        }
        ByteTarget::Seven(hl_target) => {
            mask = 0b01111111; // Byte Mask
            found_target = hl_target;
        }
    }

    // Now see if found target is hl -> memory location
    match found_target {
        HLTarget::HL => {
            is_mem = true;
        }
        _ => {}
    }

    // Get Target Register
    target_register = match_hl(cpu, found_target);

    // Perform Operation
    if is_mem {
        // if were updating memory write back to grabbed location the new value
        cpu.bus
            .write_byte(None, cpu.registers.get_hl(), target_register & mask);
    } else {
        target_register &= mask;
    }

    // Prefixed Return
    cpu.pc.wrapping_add(2)
}

pub fn op_set(cpu: &mut CPU, target: ByteTarget) -> u16 {
    let mask: u8;
    let mut target_register: u8;
    let mut is_mem: bool = false;
    let found_target: HLTarget;

    match target {
        ByteTarget::Zero(hl_target) => {
            mask = 0b00000001; // Byte Mask
            found_target = hl_target;
        }
        ByteTarget::One(hl_target) => {
            mask = 0b00000010;
            found_target = hl_target;
        }
        ByteTarget::Two(hl_target) => {
            mask = 0b00000100;
            found_target = hl_target;
        }
        ByteTarget::Three(hl_target) => {
            mask = 0b00001000;
            found_target = hl_target;
        }
        ByteTarget::Four(hl_target) => {
            mask = 0b00010000;
            found_target = hl_target;
        }
        ByteTarget::Five(hl_target) => {
            mask = 0b00100000;
            found_target = hl_target;
        }
        ByteTarget::Six(hl_target) => {
            mask = 0b01000000;
            found_target = hl_target;
        }
        ByteTarget::Seven(hl_target) => {
            mask = 0b10000000;
            found_target = hl_target;
        }
    }
    // Determine if were using memory
    match found_target {
        HLTarget::HL => {
            is_mem = true; // flag that were grabbing memory
        }
        _ => {}
    }

    // Find Target
    target_register = match_hl(cpu, found_target);

    // Perform Operation
    if is_mem {
        // if were updating memory write back to grabbed location the new value
        cpu.bus
            .write_byte(None, cpu.registers.get_hl(), target_register & mask);
    } else {
        target_register &= mask;
    }

    // Prefixed Return
    cpu.pc.wrapping_add(2)
}

pub fn op_cp(cpu: &mut CPU, target: OPTarget) -> u16 {
    match target {
        OPTarget::B => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.b);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::C => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.c);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::D => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.d);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::E => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.e);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::H => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.h);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::L => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.l);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::HL => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.get_hl() as u8);

            cpu.pc.wrapping_add(3)
        }
        OPTarget::A => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.registers.a);
            cpu.pc.wrapping_add(1)
        }
        OPTarget::D8 => {
            // CP -> Set Flags
            set_flags_after_cp(cpu, cpu.registers.a, cpu.bus.read_byte(None, cpu.pc + 1));
            cpu.pc.wrapping_add(2)
        }
    }
}

pub fn op_or(cpu: &mut CPU, target: OPTarget) -> u16 {
    let result_pc: u16;
    match target {
        OPTarget::B => {
            // OR
            cpu.registers.a |= cpu.registers.b;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::C => {
            // OR
            cpu.registers.a |= cpu.registers.c;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::D => {
            // OR
            cpu.registers.a |= cpu.registers.d;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::E => {
            // OR
            cpu.registers.a |= cpu.registers.e;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::H => {
            // OR
            cpu.registers.a |= cpu.registers.h;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::L => {
            // OR
            cpu.registers.a |= cpu.registers.l;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::HL => {
            // OR
            cpu.registers.a |= cpu.bus.read_byte(None, cpu.registers.get_hl());

            result_pc = cpu.pc.wrapping_add(3);
        }
        OPTarget::A => {
            // OR
            cpu.registers.a |= cpu.registers.a;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::D8 => {
            // OR
            cpu.registers.a = cpu.bus.read_byte(None, cpu.pc + 1);

            result_pc = cpu.pc.wrapping_add(2);
        }
    }
    // Set Flags
    set_flags_after_xor_or(cpu, cpu.registers.a);

    // Implicit Return
    result_pc
}

pub fn op_xor(cpu: &mut CPU, target: OPTarget) -> u16 {
    let result_pc: u16;
    match target {
        OPTarget::B => {
            // XOR
            cpu.registers.a ^= cpu.registers.b;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::C => {
            // XOR
            cpu.registers.a ^= cpu.registers.c;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::D => {
            // XOR
            cpu.registers.a ^= cpu.registers.d;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::E => {
            // XOR
            cpu.registers.a ^= cpu.registers.e;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::H => {
            // XOR
            cpu.registers.a ^= cpu.registers.h;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::L => {
            // XOR
            cpu.registers.a ^= cpu.registers.l;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::HL => {
            // XOR
            cpu.registers.a ^= cpu.bus.read_byte(None, cpu.registers.get_hl());

            result_pc = cpu.pc.wrapping_add(3);
        }
        OPTarget::A => {
            // XOR
            cpu.registers.a ^= cpu.registers.a;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::D8 => {
            // XOR
            cpu.registers.a ^= cpu.bus.read_byte(None, cpu.pc + 1);

            result_pc = cpu.pc.wrapping_add(2);
        }
    }
    // Set Flags
    set_flags_after_xor_or(cpu, cpu.registers.a);

    // Implicit Return
    result_pc
}

pub fn op_and(cpu: &mut CPU, target: OPTarget) -> u16 {
    let result_pc: u16;
    match target {
        OPTarget::B => {
            // AND
            cpu.registers.a &= cpu.registers.b;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::C => {
            // AND
            cpu.registers.a &= cpu.registers.c;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::D => {
            // AND
            cpu.registers.a &= cpu.registers.d;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::E => {
            // AND
            cpu.registers.a &= cpu.registers.e;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::H => {
            // AND
            cpu.registers.a &= cpu.registers.h;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::L => {
            // AND
            cpu.registers.a &= cpu.registers.l;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::HL => {
            // AND
            cpu.registers.a &= cpu.bus.read_byte(None, cpu.registers.get_hl());

            result_pc = cpu.pc.wrapping_add(3);
        }
        OPTarget::A => {
            // AND
            cpu.registers.a &= cpu.registers.a;

            result_pc = cpu.pc.wrapping_add(1);
        }
        OPTarget::D8 => {
            // AND
            cpu.registers.a &= cpu.bus.read_byte(None, cpu.pc + 1);

            result_pc = cpu.pc.wrapping_add(2);
        }
    }
    // Set Flags
    set_flags_after_and(cpu, cpu.registers.a);

    // Implicit Return
    result_pc
}

pub fn op_sbc(cpu: &mut CPU, target: OPTarget) -> u16 {
    let original_value = cpu.registers.a;
    match target {
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

pub fn op_sub(cpu: &mut CPU, target: OPTarget) -> u16 {
    // Get Original Value
    let original_value = cpu.registers.a;
    match target {
        OPTarget::B => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.b);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.b);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::C => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.c);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.c);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::D => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.d);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.d);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::E => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.e);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.e);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::H => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.h);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.h);

            cpu.pc.wrapping_add(1)
        }
        OPTarget::L => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.l);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, cpu.registers.l);

            cpu.pc.wrapping_add(1)
        }
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
        OPTarget::A => {
            // SUB
            cpu.registers.a = cpu.registers.a.wrapping_sub(cpu.registers.a);

            // Set Flags
            set_flags_after_sub(cpu, cpu.registers.a, original_value, original_value);

            cpu.pc.wrapping_add(1)
        }
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
            // Store Original Value
            let original_value = cpu.registers.a;
            // ADC
            cpu.registers.a = cpu.registers.b.wrapping_add(cpu.registers.f.carry as u8);

            // Set Flags
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.b);
            cpu.pc.wrapping_add(1)
        }
        // [0x89]
        OPTarget::C => {
            // Store Original Value
            let original_value = cpu.registers.a;
            // ADC
            cpu.registers.a = cpu.registers.c.wrapping_add(cpu.registers.f.carry as u8);
            // Set Flags
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.c);
            cpu.pc.wrapping_add(1)
        }
        // [0x8A]
        OPTarget::E => {
            // Store Original Value
            let original_value = cpu.registers.a;
            // ADC
            cpu.registers.a = cpu.registers.e.wrapping_add(cpu.registers.f.carry as u8);

            // Set Flags
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.e);
            cpu.pc.wrapping_add(1)
        }
        // [0x8B]
        OPTarget::D => {
            // Store Original Value
            let original_value = cpu.registers.a;

            // ADC
            cpu.registers.a = cpu.registers.d.wrapping_add(cpu.registers.f.carry as u8);

            // Set Flags
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.d);
            cpu.pc.wrapping_add(1)
        }
        // [0x8C]
        OPTarget::H => {
            // Store Original Value
            let original_value = cpu.registers.a;

            // ADC
            cpu.registers.a = cpu.registers.h.wrapping_add(cpu.registers.f.carry as u8);

            // Set Flags
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.h);
            cpu.pc.wrapping_add(1)
        }
        // [0x8D]
        OPTarget::L => {
            // Store Original Value
            let original_value = cpu.registers.a;

            // ADC
            cpu.registers.a = cpu.registers.l.wrapping_add(cpu.registers.f.carry as u8);

            // Set Flags
            set_flags_after_adc(cpu, cpu.registers.a, original_value, cpu.registers.l);
            cpu.pc.wrapping_add(1)
        }
        // [0x8E]
        OPTarget::HL => {
            // Store Original Value
            let original_value = cpu.registers.a;

            // ADC
            cpu.registers.a = cpu
                .bus
                .read_byte(None, cpu.registers.get_hl())
                .wrapping_add(cpu.registers.f.carry as u8);

            // Set Flags
            set_flags_after_adc(
                cpu,
                cpu.registers.a,
                original_value,
                cpu.bus.read_byte(None, cpu.registers.get_hl()),
            );
            cpu.pc.wrapping_add(1)
        }
        // [0x8E]
        OPTarget::A => {
            // Store Original Value
            let original_value = cpu.registers.a;

            // ADC
            cpu.registers.a = cpu.registers.a.wrapping_add(cpu.registers.f.carry as u8);

            // Set Flags
            set_flags_after_adc(cpu, cpu.registers.a, original_value, original_value);
            cpu.pc.wrapping_add(1)
        }
        // [0xCE]
        OPTarget::D8 => {
            // Store Original Values
            let original_value = cpu.registers.a;

            // ADC
            cpu.registers.a = cpu
                .bus
                .read_byte(None, cpu.pc + 1)
                .wrapping_add(cpu.registers.f.carry as u8);

            // Set Flags
            set_flags_after_adc(
                cpu,
                cpu.registers.a,
                original_value,
                cpu.bus.read_byte(None, cpu.pc + 1),
            );
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
            let reg_target = match_hl(cpu, target);

            // Store the original value of A
            let original = cpu.registers.a;

            // ADD
            cpu.registers.a = original.wrapping_add(reg_target);

            // Set Flags [Z 0 H CY]
            cpu.registers.f.zero = cpu.registers.a == 0; // Zero Flag: Set if the result is zero
            cpu.registers.f.subtract = false; // Subtract Flag: Not set for ADD operations
            cpu.registers.f.half_carry = (original & 0x0F) + (reg_target & 0x0F) > 0x0F; // Half-Carry Flag: Set if there was a carry from bit 3 to bit 4
            cpu.registers.f.carry = cpu.registers.a < original; // Carry Flag: Set if the addition overflowed an 8-bit value

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
            cpu.registers.f.carry =
                ((cpu.registers.get_hl() as u32) + (reg_target as u32)) > 0xFFFF; // Carry Flag: Check for carry from the addition
            cpu.registers.f.half_carry =
                ((cpu.registers.get_hl() & 0x0FFF) + (reg_target & 0x0FFF)) > 0x0FFF; // Half-Carry Flag: Check if there was a carry from bit 11 to bit 12
            cpu.registers.f.subtract = false; // Subtract Flag: Not set for ADD operations
            cpu.registers.f.zero = false; // Zero Flag: Not affected, but set to false

            cpu.pc.wrapping_add(1)
        }
        // [0xE8]
        OPType::LoadSP => {
            // Find and Sign-extend the immediate operand to 16 bits
            let signed_value = (cpu.bus.read_byte(None, cpu.pc + 1) as i8) as i16;

            // ADD
            cpu.sp = cpu.sp.wrapping_add(signed_value as u16);

            // Set Flags [0 0 H CY]
            cpu.registers.f.zero = cpu.sp == 0; // zero
            cpu.registers.f.subtract = false; // subtract
            cpu.registers.f.carry = (cpu.sp as i16) < (signed_value as i16); // Carry Flag: Check if there's a carry out (would occur if SP > 0xFFFF)
            cpu.registers.f.half_carry =
                ((cpu.sp & 0x0F) as i16 + (signed_value & 0x0F) as i16) > 0x0F; // Half-Carry Flag: Check if there's a carry from bit 11 to bit 12 this check is done based on the lower 4 bits

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
            cpu.registers.f.zero = cpu.registers.a == 0;
            cpu.registers.f.subtract = false;
            // Half-Carry Flag: Set if there was a carry from bit 3 to bit 4
            cpu.registers.f.half_carry = ((original & 0x0F) + (cpu.registers.a & 0x0F)) > 0x0F;
            // Carry Flag: Set if there was a carry out from the most significant bit
            cpu.registers.f.carry =
                (cpu.registers.a < original) || (cpu.registers.a < immediate_operand);

            cpu.pc.wrapping_add(2)
        }
    }
}

pub fn op_ld(cpu: &mut CPU, target: LoadType) -> u16 {
    match target {
        LoadType::RegInReg(target, source) => match target {
            HLTarget::B => match source {
                HLTarget::B => {
                    cpu.registers.b = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::C => {
                    cpu.registers.b = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::D => {
                    cpu.registers.b = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::E => {
                    cpu.registers.b = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::H => {
                    cpu.registers.b = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::L => {
                    cpu.registers.b = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::HL => {
                    cpu.registers.b = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::A => {
                    cpu.registers.b = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            HLTarget::C => match target {
                HLTarget::B => {
                    cpu.registers.c = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::C => {
                    cpu.registers.c = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::D => {
                    cpu.registers.c = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::E => {
                    cpu.registers.c = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::H => {
                    cpu.registers.c = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::L => {
                    cpu.registers.c = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::HL => {
                    cpu.registers.c = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::A => {
                    cpu.registers.c = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            HLTarget::D => match target {
                HLTarget::B => {
                    cpu.registers.d = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::C => {
                    cpu.registers.d = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::D => {
                    cpu.registers.d = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::E => {
                    cpu.registers.d = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::H => {
                    cpu.registers.d = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::L => {
                    cpu.registers.d = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::HL => {
                    cpu.registers.d = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::A => {
                    cpu.registers.d = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            HLTarget::E => match target {
                HLTarget::B => {
                    cpu.registers.e = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::C => {
                    cpu.registers.e = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::D => {
                    cpu.registers.e = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::E => {
                    cpu.registers.e = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::H => {
                    cpu.registers.e = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::L => {
                    cpu.registers.e = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::HL => {
                    cpu.registers.e = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::A => {
                    cpu.registers.e = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            HLTarget::H => match target {
                HLTarget::B => {
                    cpu.registers.h = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::C => {
                    cpu.registers.h = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::D => {
                    cpu.registers.h = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::E => {
                    cpu.registers.h = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::H => {
                    cpu.registers.h = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::L => {
                    cpu.registers.h = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::HL => {
                    cpu.registers.h = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::A => {
                    cpu.registers.h = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            HLTarget::L => match target {
                HLTarget::B => {
                    cpu.registers.l = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::C => {
                    cpu.registers.l = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::D => {
                    cpu.registers.l = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::E => {
                    cpu.registers.l = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::H => {
                    cpu.registers.l = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::L => {
                    cpu.registers.l = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::HL => {
                    cpu.registers.l = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::A => {
                    cpu.registers.l = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
            HLTarget::HL => match target {
                HLTarget::B => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.b);
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::C => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.c);
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::D => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.d);
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::E => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.e);
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::H => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.h);
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::L => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.l);
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::A => {
                    cpu.bus
                        .write_byte(None, cpu.registers.get_hl(), cpu.registers.a);
                    cpu.pc.wrapping_add(1)
                }
                _ => panic!("Getting LD HL HL Should be HALT"),
            },
            HLTarget::A => match target {
                HLTarget::B => {
                    cpu.registers.a = cpu.registers.b;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::C => {
                    cpu.registers.a = cpu.registers.c;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::D => {
                    cpu.registers.a = cpu.registers.d;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::E => {
                    cpu.registers.a = cpu.registers.e;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::H => {
                    cpu.registers.a = cpu.registers.h;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::L => {
                    cpu.registers.a = cpu.registers.l;
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::HL => {
                    cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_hl());
                    cpu.pc.wrapping_add(1)
                }
                HLTarget::A => {
                    cpu.registers.a = cpu.registers.a;
                    cpu.pc.wrapping_add(1)
                }
            },
        },
        LoadType::Word(target, source) => {
            // Read the next two bytes from bus at the current PC
            let low_byte = cpu.bus.read_byte(None, cpu.pc + 1); // Read the low byte
            let high_byte = cpu.bus.read_byte(None, cpu.pc + 2); // Read the high byte

            // Combine the low and high bytes into a 16-bit value
            let word_value = ((high_byte as u16) << 8) | (low_byte as u16);

            match target {
                LoadWordTarget::BC => match source {
                    LoadWordSource::N16 => {
                        cpu.registers.set_bc(word_value as u16);
                        cpu.pc.wrapping_add(3)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
                LoadWordTarget::HL => match source {
                    LoadWordSource::N16 => {
                        cpu.registers.set_hl(word_value as u16);

                        cpu.pc.wrapping_add(3)
                    }
                    LoadWordSource::SPE8 => {
                        cpu.registers.set_hl(
                            ((cpu.sp as i16)
                                .wrapping_add((cpu.bus.read_byte(None, cpu.pc + 1) as i8) as i16))
                                as u16,
                        );

                        // Set Flags
                        cpu.registers.f.subtract = false;

                        cpu.registers.f.half_carry = ((cpu.sp & 0x0F)
                            + (cpu.bus.read_byte(None, cpu.pc + 1) as u16 & 0x0F))
                            > 0x0F;

                        cpu.registers.f.carry = ((cpu.sp & 0xFF)
                            + (cpu.bus.read_byte(None, cpu.pc + 1) as u16 & 0xFF))
                            > 0xFF;

                        cpu.pc.wrapping_add(2)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
                LoadWordTarget::DE => match source {
                    LoadWordSource::N16 => {
                        cpu.registers
                            .set_de(cpu.bus.read_byte(None, word_value) as u16);
                        cpu.pc.wrapping_add(3)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
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
                LoadWordTarget::SP => match source {
                    LoadWordSource::HL => {
                        cpu.registers.set_hl(cpu.sp);
                        cpu.pc.wrapping_add(1)
                    }
                    LoadWordSource::N16 => {
                        cpu.sp = word_value;
                        cpu.pc.wrapping_add(3)
                    }
                    _ => panic!("LD WORD BAD MATCH"),
                },
            }
        }
        LoadType::AStoreInN16(target) => match target {
            LoadN16::BC => {
                cpu.bus
                    .write_byte(None, cpu.registers.get_bc(), cpu.registers.a);
                cpu.pc.wrapping_add(1)
            }
            LoadN16::DE => {
                cpu.bus
                    .write_byte(None, cpu.registers.get_de(), cpu.registers.a);
                cpu.pc.wrapping_add(1)
            }
            LoadN16::HLDEC => {
                cpu.bus
                    .write_byte(None, cpu.registers.get_hl(), cpu.registers.a);
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1));
                cpu.pc.wrapping_add(1)
            }
            LoadN16::HLINC => {
                cpu.bus
                    .write_byte(None, cpu.registers.get_hl(), cpu.registers.a);
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(1));
                cpu.pc.wrapping_add(1)
            }
        },
        LoadType::N16StoreInA(source) => match source {
            LoadN16::BC => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_bc());
                cpu.pc.wrapping_add(1)
            }
            LoadN16::DE => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_de());
                cpu.pc.wrapping_add(1)
            }
            LoadN16::HLDEC => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_hl());
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1));
                cpu.pc.wrapping_add(1)
            }
            LoadN16::HLINC => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.registers.get_hl());
                cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(1));
                cpu.pc.wrapping_add(1)
            }
        },
        LoadType::D8StoreInReg(target) => match target {
            HLTarget::B => {
                cpu.registers.b = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            HLTarget::C => {
                cpu.registers.c = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            HLTarget::D => {
                cpu.registers.d = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            HLTarget::E => {
                cpu.registers.e = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            HLTarget::H => {
                cpu.registers.h = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            HLTarget::L => {
                cpu.registers.l = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
            HLTarget::HL => {
                cpu.bus.write_byte(
                    None,
                    cpu.registers.get_hl(),
                    cpu.bus.read_byte(None, cpu.pc + 1),
                );
                cpu.pc.wrapping_add(2)
            }
            HLTarget::A => {
                cpu.registers.a = cpu.bus.read_byte(None, cpu.pc + 1);
                cpu.pc.wrapping_add(2)
            }
        },
        LoadType::AWithA8(target) => match target {
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
        LoadType::AWithA16(target) => {
            let low_byte = cpu.bus.read_byte(None, cpu.pc + 1); // Read the low byte
            let high_byte = cpu.bus.read_byte(None, cpu.pc + 2); // Read the high byte

            // Combine the low and high bytes into a 16-bit value
            let address = ((high_byte as u16) << 8) | (low_byte as u16);

            match target {
                LoadA16Target::A => {
                    cpu.registers.a = cpu.bus.read_byte(None, address);
                    cpu.pc.wrapping_add(3)
                }
                LoadA16Target::A16 => {
                    cpu.bus.write_byte(None, address, cpu.registers.a);
                    cpu.pc.wrapping_add(3)
                }
            }
        }
        LoadType::AWithAC(target) => match target {
            LoadACTarget::A => {
                cpu.bus
                    .write_byte(None, 0xFF00 + cpu.registers.c as u16, cpu.registers.a);
                cpu.pc.wrapping_add(2)
            }
            LoadACTarget::C => {
                cpu.registers.a = cpu.bus.read_byte(None, 0xFF00 + cpu.registers.c as u16);
                cpu.pc.wrapping_add(2)
            }
        },
    }
}

pub fn op_dec(cpu: &mut CPU, target: AllRegisters) -> u16 {
    match target {
        // Increment 8-bit registers and Set Flags
        AllRegisters::A => {
            let original_value = cpu.registers.a;
            cpu.registers.a = cpu.registers.a.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.a, original_value);
        }
        AllRegisters::B => {
            let original_value = cpu.registers.b;
            cpu.registers.b = cpu.registers.b.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.b, original_value);
        }
        AllRegisters::C => {
            let original_value = cpu.registers.c;
            cpu.registers.c = cpu.registers.c.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.c, original_value);
        }
        AllRegisters::D => {
            let original_value = cpu.registers.d;
            cpu.registers.d = cpu.registers.d.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.d, original_value);
        }
        AllRegisters::E => {
            let original_value = cpu.registers.e;
            cpu.registers.e = cpu.registers.e.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.e, original_value);
        }
        AllRegisters::H => {
            let original_value = cpu.registers.h;
            cpu.registers.h = cpu.registers.h.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.h, original_value);
        }
        AllRegisters::L => {
            let original_value = cpu.registers.l;
            cpu.registers.l = cpu.registers.l.wrapping_sub(1);
            set_flags_after_dec(cpu, cpu.registers.l, original_value);
        }
        // Increment value at bus location HL
        AllRegisters::HLMEM => {
            let hl_addr = cpu.registers.get_hl();
            let original_value = cpu.bus.read_byte(None, hl_addr);
            let value = cpu.bus.read_byte(None, hl_addr).wrapping_sub(1);
            cpu.bus.write_byte(None, hl_addr, value);
            set_flags_after_dec(cpu, value, original_value);
        }
        // 16-bit register increments (don't need to Set Flags for these)
        AllRegisters::BC => {
            let new_bc = cpu.registers.get_bc().wrapping_sub(1);
            cpu.registers.set_bc(new_bc);
        }
        AllRegisters::DE => {
            let new_de = cpu.registers.get_de().wrapping_sub(1);
            cpu.registers.set_de(new_de);
        }
        AllRegisters::HL => {
            let new_hl = cpu.registers.get_hl().wrapping_sub(1);
            cpu.registers.set_hl(new_hl);
        }
        AllRegisters::SP => {
            cpu.sp = cpu.sp.wrapping_sub(1);
        }
    }
    cpu.pc.wrapping_add(1)
}

pub fn op_inc(cpu: &mut CPU, target: AllRegisters) -> u16 {
    match target {
        // Increment 8-bit registers and Set Flags
        AllRegisters::A => {
            cpu.registers.a = cpu.registers.a.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.a);
        }
        AllRegisters::B => {
            cpu.registers.b = cpu.registers.b.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.b);
        }
        AllRegisters::C => {
            cpu.registers.c = cpu.registers.c.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.c);
        }
        AllRegisters::D => {
            cpu.registers.d = cpu.registers.d.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.d);
        }
        AllRegisters::E => {
            cpu.registers.e = cpu.registers.e.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.e);
        }
        AllRegisters::H => {
            cpu.registers.h = cpu.registers.h.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.h);
        }
        AllRegisters::L => {
            cpu.registers.l = cpu.registers.l.wrapping_add(1);
            set_flags_after_inc(cpu, cpu.registers.l);
        }
        // Increment value at bus location HL
        AllRegisters::HLMEM => {
            let hl_addr = cpu.registers.get_hl();
            let value = cpu.bus.read_byte(None, hl_addr).wrapping_add(1);
            cpu.bus.write_byte(None, hl_addr, value);
            set_flags_after_inc(cpu, value);
        }
        // 16-bit register increments (don't need to Set Flags for these)
        AllRegisters::BC => {
            let new_bc = cpu.registers.get_bc().wrapping_add(1);
            cpu.registers.set_bc(new_bc);
        }
        AllRegisters::DE => {
            let new_de = cpu.registers.get_de().wrapping_add(1);
            cpu.registers.set_de(new_de);
        }
        AllRegisters::HL => {
            let new_hl = cpu.registers.get_hl().wrapping_add(1);
            cpu.registers.set_hl(new_hl);
        }
        AllRegisters::SP => {
            cpu.sp = cpu.sp.wrapping_add(1);
        }
    }
    cpu.pc.wrapping_add(1)
}

// MAYBE CHANGE TO GOTO_ADDR IN FUTURE?
pub fn op_jr(cpu: &mut CPU, target: JumpTest) -> u16 {
    let jump_distance = cpu.bus.read_byte(None, cpu.pc + 1) as i8;
    match target {
        JumpTest::NotZero => {
            if !cpu.registers.f.zero {
                cpu.pc = cpu.pc.wrapping_add(jump_distance as u16)
            }
        }
        JumpTest::NotCarry => {
            if !cpu.registers.f.carry {
                cpu.pc = cpu.pc.wrapping_add(jump_distance as u16)
            }
        }
        JumpTest::Always => cpu.pc = cpu.pc.wrapping_add(jump_distance as u16),
        JumpTest::Zero => {
            if cpu.registers.f.zero {
                cpu.pc = cpu.pc.wrapping_add(jump_distance as u16)
            }
        }
        JumpTest::Carry => {
            if cpu.registers.f.carry {
                cpu.pc = cpu.pc.wrapping_add(jump_distance as u16)
            }
        }
        JumpTest::HL => {
            panic!("BAD JR REQUEST");
        }
    }
    cpu.pc.wrapping_add(1)
}

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
        StackTarget::AF => {
            cpu.registers.set_af(combined & 0xFFF0);
        }
        StackTarget::BC => {
            cpu.registers.set_bc(combined);
        }
        StackTarget::DE => {
            cpu.registers.set_de(combined);
        }
        StackTarget::HL => {
            cpu.registers.set_hl(combined);
        }
    }

    // Implicit Return
    cpu.pc.wrapping_add(1)
}

pub fn op_push(cpu: &mut CPU, target: StackTarget) -> u16 {
    match target {
        StackTarget::AF => {
            let high: u16 = (cpu.registers.get_af() >> 8) & 0xFF as u16;
            // Cycle
            stack_push(cpu, high as u8);

            let low: u16 = cpu.registers.get_af() & 0xFF as u16;
            // Cycle
            stack_push(cpu, low as u8);

            // Cycle
        }
        StackTarget::BC => {
            let high: u16 = (cpu.registers.get_bc() >> 8) & 0xFF as u16;
            // Cycle
            stack_push(cpu, high as u8);

            let low: u16 = cpu.registers.get_bc() & 0xFF as u16;
            // Cycle
            stack_push(cpu, low as u8);

            // Cycle
        }
        StackTarget::DE => {
            let high: u16 = (cpu.registers.get_de() >> 8) & 0xFF as u16;
            // Cycle
            stack_push(cpu, high as u8);

            let low: u16 = cpu.registers.get_de() & 0xFF as u16;
            // Cycle
            stack_push(cpu, low as u8);

            // Cycle
        }
        StackTarget::HL => {
            let high: u16 = (cpu.registers.get_hl() >> 8) & 0xFF as u16;
            // Cycle
            stack_push(cpu, high as u8);

            let low: u16 = cpu.registers.get_hl() & 0xFF as u16;
            // Cycle
            stack_push(cpu, low as u8);

            // Cycle
        }
    };
    cpu.pc.wrapping_add(1)
}

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

        // Implicit Return
        cpu.pc
    } else {
        cpu.pc.wrapping_add(3) // maybe not correct
    }
}

pub fn op_reti(cpu: &mut CPU) -> u16 {
    // Update Interrupt
    cpu.master_enabled = true;

    // Call RET Logic w Always so it executes
    op_ret(cpu, JumpTest::Always)
}

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
