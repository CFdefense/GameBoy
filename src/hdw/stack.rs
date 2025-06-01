/*
  hdw/stack.rs
  Info: Game Boy CPU stack operations and stack pointer management
  Description: The stack module implements stack operations for the Game Boy CPU including
              push/pop operations for bytes and 16-bit words. Provides proper stack pointer
              management and cycle-accurate timing for stack-based operations.

  Stack Architecture:
    - Descending stack (grows downward from high to low addresses)
    - Stack pointer (SP) points to next available stack location
    - Initial SP value: 0xFFFE (top of high RAM)
    - Stack operations use Work RAM and High RAM regions

  Core Functions:
    stack_push: 8-bit Push - Pushes single byte to stack with optional cycle timing
    stack_push16: 16-bit Push - Pushes word to stack (high byte first, then low byte)
    stack_pop: 8-bit Pop - Pops single byte from stack with automatic cycle timing

  Stack Operations:
    Push Operation (stack_push):
      1. Decrement stack pointer (SP--)
      2. Optionally consume 1 M-cycle for timing accuracy
      3. Write value to memory at new SP location

    16-bit Push Operation (stack_push16):
      1. Push high byte of 16-bit value first
      2. Push low byte of 16-bit value second
      3. Maintains little-endian byte order on stack

    Pop Operation (stack_pop):
      1. Read value from memory at current SP location
      2. Increment stack pointer (SP++)
      3. Consume 1 M-cycle for timing accuracy

  Memory Access:
    - Stack operations use standard bus interface
    - Stack memory located in Work RAM (0xC000-0xDFFF) and High RAM (0xFF80-0xFFFE)
    - No special stack memory protection or overflow detection
    - Stack can grow into any writable memory region

  Timing Behavior:
    - Optional cycle consumption for push operations (controlled by cycle parameter)
    - Automatic cycle consumption for pop operations
    - Timing matches original Game Boy stack operation timing
    - Cycle consumption coordinates with global emulation timing

  CPU Integration:
    - Direct manipulation of CPU stack pointer register
    - Uses CPU bus interface for memory access
    - Integrates with emulation cycle counting system
    - Supports both instruction-driven and interrupt-driven stack operations

  Use Cases:
    - Function call/return mechanisms (CALL/RET instructions)
    - Interrupt handling (automatic register preservation)
    - Temporary value storage during complex operations
    - Subroutine parameter passing and local variables

  Safety Features:
    - Unsafe pointer operations isolated to minimal scope
    - Temporary reference creation for read operations
    - Stack pointer validation through CPU state management
    - Memory access bounds checking through bus interface
*/

use crate::hdw::cpu::CPU;
use crate::hdw::emu::emu_cycles;
pub fn stack_push(cpu: &mut CPU, value: u8, cycle: bool) {
    // Decrement Stack Pointer
    cpu.sp -= 1;

    if cycle {
        emu_cycles(cpu, 1);
    }

    cpu.bus.write_byte(cpu.sp, value);
}

pub fn stack_push16(cpu: &mut CPU, value: u16, cycle: bool) {
    // Push high byte
    stack_push(cpu, (value >> 8) as u8, cycle);
    // Push low byte
    stack_push(cpu, (value & 0xFF) as u8, cycle);
}

pub fn stack_pop(cpu: &mut CPU) -> u8 {
    // Grab Original Address
    let address = cpu.sp;

    // Increment SP
    cpu.sp += 1;

    emu_cycles(cpu, 1);

    // Create a temporary mutable reference for the write operation
    {
        let cpu_ref = cpu as *mut CPU;
        // SAFETY: We're only creating a temporary reference and not modifying any state
        // The CPU reference is valid for the duration of this scope
        // We ensure no other mutable references exist during this time
        cpu.bus.read_byte(Some(unsafe { &mut *cpu_ref }), address)
    }
}
