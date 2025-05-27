use crate::hdw::cpu::CPU;
use crate::hdw::emu::emu_cycles;
pub fn stack_push(cpu: &mut CPU, value: u8, cycle: bool) {
    // Decrement Stack Pointer
    cpu.sp -= 1;

    if cycle {
        emu_cycles(cpu, 1);
    }
    // Create a temporary mutable reference for the write operation
    {
        let cpu_ref = cpu as *mut CPU;
        // SAFETY: We're only creating a temporary reference and not modifying any state
        // The CPU reference is valid for the duration of this scope
        // We ensure no other mutable references exist during this time
        cpu.bus
            .write_byte(Some(unsafe { &mut *cpu_ref }), cpu.sp, value);
        print!("Stack push wrote to {:?} the value {:?}", cpu.sp, value);
        print!(
            "Got from {:?} the value {:?}",
            cpu.sp,
            cpu.bus.read_byte(Some(unsafe { &mut *cpu_ref }), cpu.sp),
        );
    }
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
