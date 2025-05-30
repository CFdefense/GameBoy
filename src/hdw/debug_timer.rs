use std::fs::OpenOptions;
use std::io::Write;
use crate::hdw::cpu::CPU;
use crate::hdw::emu::EmuContext;
use std::sync::Arc;
use std::sync::Mutex;

pub fn log_timer_state(cpu: &CPU, ctx: &Arc<Mutex<EmuContext>>, message: &str) {
    // Only log if debug mode is enabled
    if !crate::hdw::emu::is_debug_enabled() {
        return;
    }
    
    let raw_int_flags = cpu.bus.interrupt_controller.get_int_flags();
    let masked_int_flags = cpu.bus.interrupt_controller.get_int_flags() | 0xE0;
    let (ticks, timer_div, timer_tima, timer_tma, timer_tac) = {
        let emu_ctx_locked = ctx.lock().unwrap();
        (
            emu_ctx_locked.ticks,
            emu_ctx_locked.timer.div,
            emu_ctx_locked.timer.tima,
            emu_ctx_locked.timer.tma,
            emu_ctx_locked.timer.tac
        )
    };

    let log_entry = format!(
        "TIMER_DEBUG - TICKS:{:08X} DIV:{:04X} TIMA:{:02X} TMA:{:02X} TAC:{:02X} INT_FLAGS(raw):{:02X} INT_FLAGS(masked):{:02X} IE_REG:{:02X} IME:{} PC:{:04X} - {}\n",
        ticks,
        timer_div,
        timer_tima,
        timer_tma,
        timer_tac,
        raw_int_flags,
        masked_int_flags,
        cpu.bus.interrupt_controller.get_ie_register(),
        cpu.bus.interrupt_controller.is_master_enabled(),
        cpu.pc,
        message
    );

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("timer_debug.txt")
    {
        let _ = file.write_all(log_entry.as_bytes());
    }
} 