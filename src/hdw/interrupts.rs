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

#[derive(Default)]
pub struct InterruptController {
    pub ie_register: u8,     // Interrupt Enable register (0xFFFF)
    pub int_flags: u8,       // Interrupt Flags register (0xFF0F)
    pub master_enabled: bool, // IME (Interrupt Master Enable)
    pub enabling_ime: bool,   // Flag for delayed IME enabling after EI
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            ie_register: 0,
            int_flags: 0,
            master_enabled: false,
            enabling_ime: false,
        }
    }

    pub fn get_ie_register(&self) -> u8 {
        self.ie_register
    }

    pub fn set_ie_register(&mut self, value: u8) {
        self.ie_register = value;
    }

    pub fn get_int_flags(&self) -> u8 {
        self.int_flags
    }

    pub fn set_int_flags(&mut self, value: u8) {
        self.int_flags = value;
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupts) {
        self.int_flags |= interrupt as u8;
    }

    pub fn step_ime(&mut self) -> bool {
        if self.enabling_ime {
            self.master_enabled = true;
            self.enabling_ime = false;
            true
        } else {
            false
        }
    }

    pub fn is_master_enabled(&self) -> bool {
        self.master_enabled
    }

    pub fn set_master_enabled(&mut self, value: bool) {
        self.master_enabled = value;
    }

    pub fn set_enabling_ime(&mut self, value: bool) {
        self.enabling_ime = value;
    }
}

pub fn int_handle(cpu: &mut CPU, address: u16) {
    stack_push16(cpu, cpu.pc, false); 
    cpu.pc = address;
}

pub fn int_check(cpu: &mut CPU, int_controller: &mut InterruptController, ctx: &Arc<Mutex<EmuContext>>, address: u16, int_type: Interrupts) -> bool {
    if (int_controller.get_int_flags() & int_type as u8) != 0 && (int_controller.ie_register & int_type as u8) != 0 {
        if let Interrupts::TIMER = int_type {
            log_timer_state(cpu, ctx, "Timer interrupt triggered");
        }
        int_handle(cpu, address);
        int_controller.set_int_flags(int_controller.get_int_flags() & !(int_type as u8));
        int_controller.master_enabled = false;
        cpu.is_halted = false;
        return true;
    }
    false
}

pub fn cpu_handle_interrupts(cpu: &mut CPU, int_controller: &mut InterruptController, ctx: &Arc<Mutex<EmuContext>>) {
    if int_check(cpu, int_controller, ctx, 0x40, Interrupts::VBLANK) {
    } else if int_check(cpu, int_controller, ctx, 0x48, Interrupts::LCDSTART) {
    } else if int_check(cpu, int_controller, ctx, 0x50, Interrupts::TIMER) {
    } else if int_check(cpu, int_controller, ctx, 0x58, Interrupts::SERIAL) {
    } else if int_check(cpu, int_controller, ctx, 0x60, Interrupts::JOYPAD) {
    }
}
