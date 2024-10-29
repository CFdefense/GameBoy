use crate::hdw::cpu::CPU;

pub fn stack_push(cpu: &mut CPU, value: u8) {
    // Decrement Stack Pointer
    cpu.sp -= 1;
    // Create a temporary mutable reference for the write operation
    {
        let cpu_ref = cpu as *mut CPU;
        // SAFETY: We're only creating a temporary reference and not modifying any state
        // The CPU reference is valid for the duration of this scope
        // We ensure no other mutable references exist during this time
        cpu.bus
            .write_byte(Some(unsafe { &mut *cpu_ref }), cpu.sp, value);
    }
}

pub fn stack_push16(cpu: &mut CPU, value: u16) {
    // Push high byte
    stack_push(cpu, (value >> 8) as u8);
    // Push low byte
    stack_push(cpu, (value & 0xFF) as u8);
}

pub fn stack_pop(cpu: &mut CPU) -> u8 {
    // Grab Original Address
    let address = cpu.sp;

    // Increment SP
    cpu.sp += 1;

    // Create a temporary mutable reference for the write operation
    {
        let cpu_ref = cpu as *mut CPU;
        // SAFETY: We're only creating a temporary reference and not modifying any state
        // The CPU reference is valid for the duration of this scope
        // We ensure no other mutable references exist during this time
        cpu.bus.read_byte(Some(unsafe { &mut *cpu_ref }), address)
    }
}

pub fn stack_pop16(cpu: &mut CPU) -> u16 {
    // Pop Low and High Bytes
    let low: u16 = stack_pop(cpu) as u16;
    let high: u16 = stack_pop(cpu) as u16;

    // Implicit Return Combined Bytes
    (high << 8) | low
}
