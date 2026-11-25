//! Serial Port Driver (COM1 at 0x3F8) for Debug Logging

use core::fmt;
use crate::arch::x86::io::{inb, outb};

const PORT_COM1: u16 = 0x3F8;

pub struct SerialPort;

impl SerialPort {
    pub unsafe fn init() {
        outb(PORT_COM1 + 1, 0x00); // Disable all interrupts
        outb(PORT_COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(PORT_COM1 + 0, 0x01); // Set divisor to 1 (lo byte) 115200 baud
        outb(PORT_COM1 + 1, 0x00); //                  (hi byte)
        outb(PORT_COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(PORT_COM1 + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(PORT_COM1 + 4, 0x03); // RTS/DTR set, OUT2=0 (no hardware IRQ generation)
    }

    fn is_transmit_empty() -> bool {
        unsafe { (inb(PORT_COM1 + 5) & 0x20) != 0 }
    }

    pub fn write_byte(byte: u8) {
        while !Self::is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe {
            outb(PORT_COM1, byte);
        }
    }

    pub fn write_str(s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                Self::write_byte(b'\r');
            }
            Self::write_byte(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        SerialPort::write_str(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _serial_print(args: fmt::Arguments) {
    use fmt::Write;
    let mut serial = SerialPort;
    let _ = serial.write_fmt(args);
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_serial_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! klog_info {
    ($($arg:tt)*) => ($crate::serial_println!("[INFO] {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! klog_warn {
    ($($arg:tt)*) => ($crate::serial_println!("[WARN] {}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! klog_err {
    ($($arg:tt)*) => ($crate::serial_println!("[ERROR] {}", format_args!($($arg)*)));
}

