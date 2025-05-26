use crate::hdw::cpu::CPU;
use crate::hdw::stack::*;

#[derive(Copy, Clone)]
pub enum Interrupts {
    VBLANK = 1,
    LCDSTART = 2,
    TIMER = 4,
    SERIAL = 8,
    JOYPAD = 16,
}

pub fn int_handle(cpu: &mut CPU, address: u16) {
    stack_push16(cpu, cpu.pc); 
    cpu.pc = address;
}

pub fn int_check(cpu: &mut CPU, address: u16, int_type: Interrupts) -> bool {
    if (cpu.int_flags & int_type as u8) != 0 && (cpu.ie_register & int_type as u8) != 0 {
        int_handle(cpu, address);
        cpu.int_flags &= !(int_type as u8);
        cpu.master_enabled = false;
        cpu.is_halted = false;
        return true;
    }
    false
}

pub fn cpu_handle_interrupts(cpu: &mut CPU) {
    if int_check(cpu, 0x40, Interrupts::VBLANK) {
    }
    if int_check(cpu, 0x48, Interrupts::LCDSTART) {
    }
    if int_check(cpu, 0x50, Interrupts::TIMER) {
    }
    if int_check(cpu, 0x58, Interrupts::SERIAL) {
    }
    if int_check(cpu, 0x60, Interrupts::JOYPAD) {
    }
}
