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

pub fn request_interrupt(req_int: Interrupts) {}

pub fn handle_interrupts(cpu: &mut CPU, address: u16) {
    // Push current PC
    stack_push16(cpu, cpu.pc);

    // Set PC to new address
    cpu.pc = address;
}

pub fn int_check(cpu: &mut CPU, address: u16, int_type: Interrupts) -> bool {
    // Check if the specified interrupt type is set and enabled
    if (cpu.int_flags & int_type as u8) != 0 && (cpu.ie_register & int_type as u8) != 0 {
        // Handle the interrupt by pushing the current PC and setting the new address
        handle_interrupts(cpu, address);

        // Clear the interrupt flag for this type, un-halt the CPU, and disable master interrupt
        cpu.int_flags &= !(int_type as u8);
        cpu.is_halted = false;
        cpu.master_enabled = false;

        return true;
    }
    false
}

pub fn cpu_handle_interrupts(cpu: &mut CPU) {
    if int_check(cpu, 0x40, Interrupts::VBLANK) {
        return;
    }
    if int_check(cpu, 0x48, Interrupts::LCDSTART) {
        return;
    }
    if int_check(cpu, 0x50, Interrupts::TIMER) {
        return;
    }
    if int_check(cpu, 0x58, Interrupts::SERIAL) {
        return;
    }
    if int_check(cpu, 0x60, Interrupts::JOYPAD) {
        return;
    }
}
