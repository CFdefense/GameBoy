/*


    Helper File to Contain Helper Utilization Functions For CPU


*/
use super::stack::stack_push16;
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

// Method to match a Jump Condition
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

// Method to match a HL Target
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

// INC FLAGS [0x04, 0x14, 0x24, 0x34, 0x0C, 0x1C, 0x2C, 0x3C]
pub fn set_flags_after_inc(cpu: &mut CPU, result: u8) {
    // [Z 0 H -]
    cpu.registers.f.zero = result == 0; // Zero Flag: Set if the result is zero
    cpu.registers.f.subtract = false; // Subtract Flag: Reset (INC is an addition)
    cpu.registers.f.half_carry = (result & 0x0F) == 0; // Half-Carry Flag: Set if there was a carry from bit 3 to bit 4
}

// DEC FLAGS [0x05, 0x15, 0x25, 0x35, 0x0D, 0x1D, 0x2D, 0x3D]
pub fn set_flags_after_dec(cpu: &mut CPU, result: u8, original_value: u8) {
    // [Z 1 H -]
    cpu.registers.f.zero = result == 0; // Zero Flag: Set if the result is zero
    cpu.registers.f.subtract = true; // Subtract Flag: SET (DEC is a subtraction)
    cpu.registers.f.half_carry = (original_value & 0x0F) == 0x00;
    // ^^ Half-Carry Flag: Set if there was a borrow from bit 4 to bit 3
}

// ADC FLAGS [0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F, 0xCE]
pub fn set_flags_after_adc(cpu: &mut CPU, result: u8, original_value: u8, immediate_operand: u8) {
    // [Z 0 H CY]
    cpu.registers.f.zero = result == 0; // Zero Flag: Set if the result is zero
    cpu.registers.f.subtract = false; // Subtract Flag: SET (ADC is not a subtraction)
    cpu.registers.f.half_carry = ((original_value & 0x0F) + (immediate_operand & 0x0F)) > 0x0F; // Half-Carry Flag: Set if there was a carry from bit 4 to bit 3
    cpu.registers.f.carry = (result < original_value) || (result < immediate_operand);
    // ^^ Carry Flag: Set if there was a carry from the 8th bit
}

// SUB SBC FLAGS [0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0xD6, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0xDE]
pub fn set_flags_after_sub(cpu: &mut CPU, result: u8, original_value: u8, immediate_operand: u8) {
    // [Z 1 H CY]
    cpu.registers.f.zero = result == 0; // Zero Flag
    cpu.registers.f.subtract = true; // Subtract Flag Always set because we SUB
    cpu.registers.f.half_carry = (original_value & 0xF) < (immediate_operand & 0xF); // Half-Carry Flag
    cpu.registers.f.carry = original_value < immediate_operand; // Carry Flag
}

// AND FLAGS [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xE6]
pub fn set_flags_after_and(cpu: &mut CPU, result: u8) {
    // [Z 0 1 0]
    cpu.registers.f.zero = result == 0; // Zero Flag
    cpu.registers.f.subtract = false; // Subtract Flag: Always cleared (AND is not a subtraction)
    cpu.registers.f.half_carry = true; // Half-Carry Flag: Always set for an AND operation
    cpu.registers.f.carry = false; // Carry Flag: Always cleared (AND does not affect carry)
}

// XOR FLAGS [0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF, 0xEE]
// OR FLAGS [0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xF6]
pub fn set_flags_after_xor_or(cpu: &mut CPU, result: u8) {
    // [Z, 0, 0, 0]
    cpu.registers.f.zero = result == 0; // Zero Flag: Set if the result is zero, otherwise cleared
    cpu.registers.f.subtract = false; // Subtract Flag: Always cleared (XOR is not a subtraction)
    cpu.registers.f.half_carry = false; // Half-Carry Flag: Always cleared (XOR does not involve a carry)
    cpu.registers.f.carry = false; // Carry Flag: Always cleared (XOR does not affect the carry)
}

// CP FLAGS [0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xFE]
pub fn set_flags_after_cp(cpu: &mut CPU, a: u8, b: u8) {
    // Calculate the result of A - B, but don't store it
    let result = a.wrapping_sub(b);

    // [Z 1 H CY]
    cpu.registers.f.zero = result == 0; // Zero Flag: Set if A == B
    cpu.registers.f.subtract = true; // Subtract Flag: Always set because this is a subtraction
    cpu.registers.f.half_carry = (a & 0xF) < (b & 0xF); // Half-Carry Flag: Set if there was a borrow from bit 4
    cpu.registers.f.carry = a < b; // Carry Flag: Set if there was a borrow from bit 8 (A < B)
}

/* BIT FLAGS
[0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F]
*/
pub fn set_flags_after_bit(cpu: &mut CPU, bit: u8, target_register: u8) {
    // [!r2 0 1 -]
    cpu.registers.f.zero = (target_register & bit) == 0; // Z flag is set if bit 0 is 0
    cpu.registers.f.subtract = false; // N flag is always cleared
    cpu.registers.f.half_carry = true; // H flag is always set
}

/*  RLC RRC RL RR SLA SRA SRL FLAGS
[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F]
*/
pub fn set_flags_after_pref_op(cpu: &mut CPU, bit: u8, reg_target: u8) {
    // [Z 0 0 REG_BIT]
    cpu.registers.f.zero = reg_target == 0;
    cpu.registers.f.carry = bit != 0;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}

// CPL FLAGS [0x2F]
pub fn set_flags_after_cpl(cpu: &mut CPU) {
    // [- 1 1 -]
    cpu.registers.f.subtract = true;
    cpu.registers.f.half_carry = true;
}

// SWAP FLAGS [0x30. 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37]
pub fn set_flags_after_swap(cpu: &mut CPU, reg_target: u8) {
    // [Z 0 0 0]
    cpu.registers.f.zero = reg_target == 0;
    cpu.registers.f.carry = false;
    cpu.registers.f.half_carry = false;
    cpu.registers.f.subtract = false;
}
// DAA FLAGS [0x27]
pub fn set_flags_after_daa(cpu: &mut CPU, carry: bool) {
    // [Z - 0 CY]
    cpu.registers.f.half_carry = false; // Clear H flag and set C flag if carry occurred
    cpu.registers.f.carry = carry;
    cpu.registers.f.zero = cpu.registers.a == 0; // Set the zero flag if the result is 0
}

// RRA RLA RLCA RRCA FLAGS [0x07, 0x17, 0x0F, 0x1F]
pub fn set_flags_after_no_pre_rl_rr(cpu: &mut CPU, bit: u8) {
    // [0 0 0 REG_BIT]
    cpu.registers.f.zero = false; // reset
    cpu.registers.f.subtract = false; // reset
    cpu.registers.f.half_carry = false; // reset
    cpu.registers.f.carry = bit != 0; // Set Carry Flag to the value of bit 0
}

// ADD A FLAGS [0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87]
pub fn set_flags_after_add_a(cpu: &mut CPU, reg_target: u8, original: u8, is_d8: bool) {
    // [Z 0 H CY]
    if is_d8 {
        cpu.registers.f.zero = cpu.registers.a == 0;
        cpu.registers.f.subtract = false;
        cpu.registers.f.half_carry = ((original & 0x0F) + (cpu.registers.a & 0x0F)) > 0x0F; // Half-Carry Flag: Set if carry from bit 3 to bit 4
        cpu.registers.f.carry = (cpu.registers.a < original) || (cpu.registers.a < reg_target);
        // ^^ Carry Flag: Set if carry out from the most significant bit
    } else {
        cpu.registers.f.zero = cpu.registers.a == 0; // Zero Flag: Set if the result is zero
        cpu.registers.f.subtract = false; // Subtract Flag: Not set for ADD operations
        cpu.registers.f.half_carry = (original & 0x0F) + (reg_target & 0x0F) > 0x0F; // Half-Carry Flag: Set if there was a carry from bit 3 to bit 4
        cpu.registers.f.carry = cpu.registers.a < original; // Carry Flag: Set if the addition overflowed an 8-bit value
    }
}

// ADD SP FLAGS [0xE8]
pub fn set_flags_after_add_sp() {}

// ADD N16 FLAGS [0x09, 0x19, 0x29, 0x39]
pub fn set_flags_after_add_n16() {}

// LD SP FLAGS [0xF8]
pub fn set_flags_after_ld_sp() {}

// Function to help streamline alot of jumping instructions
pub fn goto_addr(cpu: &mut CPU, address: u16, jump: bool, push_pc: bool) -> u16 {
    if jump {
        if push_pc {
            // cycle 2
            stack_push16(cpu, cpu.pc);
        }
        // combine and set pc to 2 byte addr in lil endian
        cpu.pc = address;

        // Implicit Return
        cpu.pc
    } else {
        // Implicit Return
        cpu.pc.wrapping_add(3)
    }
}
