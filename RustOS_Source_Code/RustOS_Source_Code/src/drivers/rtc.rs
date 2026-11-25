//! CMOS Real-Time Clock (RTC) Driver

use crate::arch::x86::io::{inb, outb};

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16    = 0x71;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    pub const fn zero() -> Self {
        Self {
            year: 2026,
            month: 11,
            day: 19,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }
}

pub struct Rtc;

impl Rtc {
    unsafe fn get_update_in_progress() -> bool {
        outb(CMOS_ADDRESS, 0x0A);
        (inb(CMOS_DATA) & 0x80) != 0
    }

    unsafe fn read_register(reg: u8) -> u8 {
        outb(CMOS_ADDRESS, reg);
        inb(CMOS_DATA)
    }

    pub fn read_datetime() -> DateTime {
        unsafe {
            // Wait until RTC is not updating
            while Self::get_update_in_progress() {
                core::hint::spin_loop();
            }

            let mut sec = Self::read_register(0x00);
            let mut min = Self::read_register(0x02);
            let mut hour = Self::read_register(0x04);
            let mut day = Self::read_register(0x07);
            let mut month = Self::read_register(0x08);
            let mut year = Self::read_register(0x09);

            let reg_b = Self::read_register(0x0B);

            // If BCD format (bit 2 of Reg B is 0), convert to binary
            if (reg_b & 0x04) == 0 {
                sec = (sec & 0x0F) + ((sec / 16) * 10);
                min = (min & 0x0F) + ((min / 16) * 10);
                hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
                day = (day & 0x0F) + ((day / 16) * 10);
                month = (month & 0x0F) + ((month / 16) * 10);
                year = (year & 0x0F) + ((year / 16) * 10);
            }

            // Convert 12 hour clock to 24 hour if necessary
            if (reg_b & 0x02) == 0 && (hour & 0x80) != 0 {
                hour = ((hour & 0x7F) + 12) % 24;
            }

            // Calculate 4-digit year (assume 2000s)
            let full_year = 2000 + year as u16;

            DateTime {
                year: full_year,
                month,
                day,
                hour,
                minute: min,
                second: sec,
            }
        }
    }
}
