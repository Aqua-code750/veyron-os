//! CMOS Real-Time Clock (RTC) Driver
//! Reads system time and date from hardware CMOS registers.

use crate::serial::{inb, outb};

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16    = 0x71;

#[derive(Debug, Copy, Clone)]
pub struct SystemTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

pub fn read_rtc() -> SystemTime {
    unsafe {
        // Wait until RTC update is not in progress (Bit 7 of Register A)
        while is_updating() {}

        let mut sec = read_cmos_register(0x00);
        let mut min = read_cmos_register(0x02);
        let mut hr  = read_cmos_register(0x04);
        let mut d   = read_cmos_register(0x07);
        let mut m   = read_cmos_register(0x08);
        let mut y   = read_cmos_register(0x09) as u16;

        let reg_b = read_cmos_register(0x0B);

        // Convert BCD to binary if not already in binary mode
        if (reg_b & 0x04) == 0 {
            sec = bcd_to_bin(sec);
            min = bcd_to_bin(min);
            hr  = ((hr & 0x7F) & 0x0F) + (((hr & 0x7F) / 16) * 10) | (hr & 0x80);
            d   = bcd_to_bin(d);
            m   = bcd_to_bin(m);
            y   = bcd_to_bin(y as u8) as u16;
        }

        // Convert 12 hour to 24 hour if necessary
        if (reg_b & 0x02) == 0 && (hr & 0x80) != 0 {
            hr = ((hr & 0x7F) + 12) % 24;
        }

        let full_year = 2000 + y;

        SystemTime {
            year: full_year,
            month: m,
            day: d,
            hours: hr,
            minutes: min,
            seconds: sec,
        }
    }
}

unsafe fn is_updating() -> bool {
    outb(CMOS_ADDRESS, 0x0A);
    (inb(CMOS_DATA) & 0x80) != 0
}

unsafe fn read_cmos_register(reg: u8) -> u8 {
    outb(CMOS_ADDRESS, reg);
    inb(CMOS_DATA)
}

fn bcd_to_bin(val: u8) -> u8 {
    ((val / 16) * 10) + (val & 0x0F)
}
