//! PIT (Programmable Interval Timer) Driver
//! Configured for 1000 Hz (1 millisecond) system tick timer.

use crate::serial::outb;
use core::sync::atomic::{AtomicU64, Ordering};

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16   = 0x43;
const PIT_BASE_FREQ: u32 = 1193182;
const TARGET_FREQ: u32   = 1000; // 1 ms tick

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    let divisor = (PIT_BASE_FREQ / TARGET_FREQ) as u16;

    unsafe {
        // Channel 0, lobyte/hibyte, Rate Generator (Mode 2), Binary 16-bit
        outb(PIT_COMMAND, 0x34);
        outb(PIT_CHANNEL_0, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xFF) as u8);

        // Unmask IRQ0 on PIC
        crate::arch::x86_64::pic::clear_mask(0);
    }

    crate::serial_println!("[PIT] PIT Timer initialized at 1000 Hz (1 ms tick).");
}

pub fn on_timer_tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    // Hook for preemptive scheduler (Stage 3)
}

pub fn uptime_ms() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn sleep_ms(ms: u64) {
    let target = uptime_ms() + ms;
    while uptime_ms() < target {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
