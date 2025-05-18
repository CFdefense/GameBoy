use crate::hdw::cpu::CPU;
use crate::hdw::stack::*;
use crate::hdw::emu::emu_cycles;

#[derive(Copy, Clone)]
pub enum Interrupts {
    VBLANK = 1,
    LCDSTART = 2,
    TIMER = 4,
    SERIAL = 8,
    JOYPAD = 16,
}


pub fn int_check(cpu: &mut CPU, address: u16, int_type: Interrupts) -> bool {
    if (cpu.int_flags & int_type as u8) != 0 && (cpu.ie_register & int_type as u8) != 0 {
        // An interrupt is pending, enabled, and IME is true (checked in CPU::step).
        
        // Standard Interrupt Sequence (5 M-cycles total):
        // 1. Disable IME, Un-halt CPU, Clear IF flag for this interrupt.
        // 2. Two M-cycles are consumed (internal operations, SP decrement).
        // 3. Push PCH onto stack (this memory write takes 1 M-cycle).
        // 4. Push PCL onto stack (this memory write takes 1 M-cycle).
        // 5. One M-cycle is consumed (internal operation, jump to vector).

        cpu.master_enabled = false;
        cpu.is_halted = false;
        cpu.int_flags &= !(int_type as u8);

        emu_cycles(cpu, 2); // Cycles for initial internal work and SP preparation.
        
        // stack_push16 pushes PC (PCH then PCL).
        // We assume stack_push16 calls stack_push twice, and stack_push
        // itself does *not* call emu_cycles. The cycles for the memory writes
        // are accounted for by the emu_cycles calls here in int_check.
        stack_push16(cpu, cpu.pc); 
        
        emu_cycles(cpu, 2); // Cycles for the two 1-byte memory writes of PCH and PCL to stack.
        
        cpu.pc = address;      // Set PC to interrupt vector.
        emu_cycles(cpu, 1);    // Cycle for the jump itself / internal processing.
                               // Total = 2 + 2 + 1 = 5 M-cycles.
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
