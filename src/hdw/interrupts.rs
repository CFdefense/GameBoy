use crate::hdw::cpu::CPU;
use crate::hdw::stack::*;
use crate::hdw::debug_timer::log_timer_state;
use crate::hdw::emu::EmuContext;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Copy, Clone)]
pub enum Interrupts {
    VBLANK = 1,
    LCDSTART = 2,
    TIMER = 4,
    SERIAL = 8,
    JOYPAD = 16,
}

pub fn int_handle(cpu: &mut CPU, address: u16) {
    stack_push16(cpu, cpu.pc, false); 
    cpu.pc = address;
}

pub fn int_check(cpu: &mut CPU, ctx: &Arc<Mutex<EmuContext>>, address: u16, int_type: Interrupts) -> bool {
    if (cpu.get_int_flags() & int_type as u8) != 0 && (cpu.ie_register & int_type as u8) != 0 {
        if let Interrupts::TIMER = int_type {
            log_timer_state(cpu, ctx, "Timer interrupt triggered");
        }
        int_handle(cpu, address);
        cpu.set_int_flags(cpu.get_int_flags() & !(int_type as u8));
        cpu.master_enabled = false;
        cpu.is_halted = false;
        if let Interrupts::TIMER = int_type {
            log_timer_state(cpu, ctx, "Timer interrupt handled - IME disabled");
        }
        return true;
    }
    false
}

pub fn cpu_handle_interrupts(cpu: &mut CPU, ctx: &Arc<Mutex<EmuContext>>) {
    if int_check(cpu, ctx, 0x40, Interrupts::VBLANK) {
    } else if int_check(cpu, ctx, 0x48, Interrupts::LCDSTART) {
    } else if int_check(cpu, ctx, 0x50, Interrupts::TIMER) {
    } else if int_check(cpu, ctx, 0x58, Interrupts::SERIAL) {
    } else if int_check(cpu, ctx, 0x60, Interrupts::JOYPAD) {
    }
}
