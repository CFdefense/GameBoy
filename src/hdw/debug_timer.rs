use std::fs::OpenOptions;
use std::io::Write;
use crate::hdw::cpu::CPU;

pub fn log_timer_state(cpu: &CPU, message: &str) {
    let log_entry = format!(
        "TIMER_DEBUG - DIV:{:04X} TIMA:{:02X} TMA:{:02X} TAC:{:02X} INT_FLAGS:{:02X} IE_REG:{:02X} IME:{} PC:{:04X} - {}\n",
        cpu.timer.div,
        cpu.timer.tima,
        cpu.timer.tma,
        cpu.timer.tac,
        cpu.int_flags,
        cpu.ie_register,
        cpu.master_enabled,
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