//! 8254 Programmable Interval Timer (PIT) Driver

use core::sync::atomic::{AtomicU32, Ordering};
use crate::arch::x86::io::outb;

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16   = 0x43;
const PIT_FREQUENCY: u32 = 1193182;
pub const TARGET_HZ: u32 = 100; // 100 Hz = 10ms per tick

static TICKS: AtomicU32 = AtomicU32::new(0);

pub unsafe fn init_pit() {
    let divisor = (PIT_FREQUENCY / TARGET_HZ) as u16;

    // Channel 0, lobyte/hibyte, Mode 2 (rate generator), binary
    outb(PIT_COMMAND, 0x36);
    outb(PIT_CHANNEL_0, (divisor & 0xFF) as u8);
    outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xFF) as u8);
}

pub fn on_timer_tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn get_ticks() -> u32 {
    TICKS.load(Ordering::Relaxed)
}

pub fn uptime_ms() -> u64 {
    (get_ticks() as u64) * (1000 / TARGET_HZ as u64)
}

pub fn uptime_secs() -> u64 {
    (get_ticks() / TARGET_HZ) as u64
}

pub fn sleep_ms(ms: u64) {
    let start = uptime_ms();
    let mut spins = ms * 50_000;
    while (start == 0 || uptime_ms() < start + ms) && spins > 0 {
        core::hint::spin_loop();
        spins -= 1;
    }
}
