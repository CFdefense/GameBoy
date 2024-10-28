/*


    Helper File to Contain Helper Utilization Functions For CPU


*/
use crate::hdw::cpu::CPU;
use crate::hdw::instructions::*;

// Method to match a N16 Target
pub fn match_n16(cpu: &mut CPU, target: AddN16Target) -> u16 {
    let reg_target = match target {
        AddN16Target::BC => cpu.registers.get_bc(),
        AddN16Target::DE => cpu.registers.get_de(),
        AddN16Target::HL => cpu.registers.get_hl(),
        AddN16Target::SP => cpu.sp,
    };
    reg_target
}

// Method to match Jump Condition
pub fn match_jump(cpu: &mut CPU, test: JumpTest) -> bool {
    let jump_condition = match test {
        JumpTest::NotZero => !cpu.registers.f.zero,
        JumpTest::NotCarry => !cpu.registers.f.carry,
        JumpTest::Zero => !cpu.registers.f.zero,
        JumpTest::Carry => !cpu.registers.f.carry,
        JumpTest::Always => true,
        JumpTest::HL => panic!("HL BAD"),
    };
    jump_condition
}

// Method to match a hl target to its register
pub fn match_hl(cpu: &mut CPU, target: HLTarget) -> u8 {
    let reg_target = match target {
        HLTarget::A => cpu.registers.a,
        HLTarget::B => cpu.registers.b,
        HLTarget::C => cpu.registers.c,
        HLTarget::D => cpu.registers.d,
        HLTarget::E => cpu.registers.e,
        HLTarget::H => cpu.registers.h,
        HLTarget::L => cpu.registers.l,
        HLTarget::HL => cpu.bus.read_byte(None, cpu.registers.get_hl()),
    };
    reg_target
}

// Jump to addr in bus or increment pc
pub fn jump(cpu: &mut CPU, jump: bool) -> u16 {
    if jump {
        let least_significant = cpu.bus.read_byte(None, cpu.pc + 1) as u16;
        let most_significant = cpu.bus.read_byte(None, cpu.pc + 2) as u16;

        // combine and return 2 byte addr in lil endian
        (most_significant << 8) | least_significant
    } else {
        // return next pc
        cpu.pc.wrapping_add(3)
    }
}

// Method to update relevant flags after INC instructions
pub fn set_flags_after_inc(cpu: &mut CPU, result: u8) {
    // Zero Flag: Set if the result is zero
    cpu.registers.f.zero = result == 0;

    // Subtract Flag: Reset (INC is an addition)
    cpu.registers.f.subtract = false;

    // Half-Carry Flag: Set if there was a carry from bit 3 to bit 4
    let half_carry = (result & 0x0F) == 0;
    cpu.registers.f.half_carry = half_carry;
}

// Method to update relevant flags after DEC instructions
pub fn set_flags_after_dec(cpu: &mut CPU, result: u8, original_value: u8) {
    // Zero Flag: Set if the result is zero
    cpu.registers.f.zero = result == 0;

    // Subtract Flag: SET (DEC is a subtraction)
    cpu.registers.f.subtract = true;

    // Half-Carry Flag: Set if there was a borrow from bit 4 to bit 3
    let half_carry = (original_value & 0x0F) == 0x00; // Borrow occurs if lower nibble was 0 before decrement
    cpu.registers.f.half_carry = half_carry;
}

// Method to update relevant flags after ADC instructions
pub fn set_flags_after_adc(cpu: &mut CPU, result: u8, original_value: u8, immediate_operand: u8) {
    // Zero Flag: Set if the result is zero
    cpu.registers.f.zero = result == 0;

    // Subtract Flag: SET (ADC is not a subtraction)
    cpu.registers.f.subtract = false;

    // Half-Carry Flag: Set if there was a carry from bit 4 to bit 3
    cpu.registers.f.half_carry = ((original_value & 0x0F) + (immediate_operand & 0x0F)) > 0x0F; // Check for carry from the lower nibble

    // Carry Flag: Set if there was a carry from the 8th bit
    cpu.registers.f.carry = (result < original_value) || (result < immediate_operand);
}

// Method to update relevant flags after SUB instructions
pub fn set_flags_after_sub(cpu: &mut CPU, result: u8, original_value: u8, immediate_operand: u8) {
    // Zero Flag
    cpu.registers.f.zero = result == 0;

    // Subtract Flag Always set because we SUB
    cpu.registers.f.subtract = true;

    // Half-Carry Flag
    cpu.registers.f.half_carry = (original_value & 0xF) < (immediate_operand & 0xF);

    // Carry Flag
    cpu.registers.f.carry = original_value < immediate_operand;
}

pub fn set_flags_after_and(cpu: &mut CPU, result: u8) {
    // Zero Flag: Set if result is zero, otherwise cleared
    cpu.registers.f.zero = result == 0;

    // Subtract Flag: Always cleared (AND is not a subtraction)
    cpu.registers.f.subtract = false;

    // Half-Carry Flag: Always set for an AND operation
    cpu.registers.f.half_carry = true;

    // Carry Flag: Always cleared (AND does not affect carry)
    cpu.registers.f.carry = false;
}

pub fn set_flags_after_xor_or(cpu: &mut CPU, result: u8) {
    // Zero Flag: Set if the result is zero, otherwise cleared
    cpu.registers.f.zero = result == 0;

    // Subtract Flag: Always cleared (XOR is not a subtraction)
    cpu.registers.f.subtract = false;

    // Half-Carry Flag: Always cleared (XOR does not involve a carry)
    cpu.registers.f.half_carry = false;

    // Carry Flag: Always cleared (XOR does not affect the carry)
    cpu.registers.f.carry = false;
}

pub fn set_flags_after_cp(cpu: &mut CPU, a: u8, b: u8) {
    // Calculate the result of A - B, but don't store it
    let result = a.wrapping_sub(b);

    // Zero Flag: Set if A == B
    cpu.registers.f.zero = result == 0;

    // Subtract Flag: Always set because this is a subtraction
    cpu.registers.f.subtract = true;

    // Half-Carry Flag: Set if there was a borrow from bit 4
    cpu.registers.f.half_carry = (a & 0xF) < (b & 0xF);

    // Carry Flag: Set if there was a borrow from bit 8 (A < B)
    cpu.registers.f.carry = a < b;
}

pub fn set_flags_after_bit(cpu: &mut CPU, bit: u8, target_register: u8) {
    // Set Flags
    cpu.registers.f.zero = (target_register & bit) == 0; // Z flag is set if bit 0 is 0
    cpu.registers.f.subtract = false; // N flag is always cleared
    cpu.registers.f.half_carry = true; // H flag is always set
}
