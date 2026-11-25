//! VGA Text Mode Driver (80x25 Character Buffer at 0xB8000)

use core::fmt;
use crate::arch::x86::io::{inb, outb};

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
const VGA_BUFFER_ADDR: usize = 0xB8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(pub u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

pub struct VgaWriter {
    pub column: usize,
    pub row: usize,
    pub color_code: ColorCode,
}

pub static mut VGA_WRITER: VgaWriter = VgaWriter {
    column: 0,
    row: 0,
    color_code: ColorCode::new(Color::LightGray, Color::Black),
};

impl VgaWriter {
    pub unsafe fn init() {
        VGA_WRITER.color_code = ColorCode::new(Color::LightGray, Color::Black);
        VGA_WRITER.clear_screen();
        VGA_WRITER.enable_cursor(14, 15);
    }

    #[inline(always)]
    fn buffer() -> *mut ScreenChar {
        VGA_BUFFER_ADDR as *mut ScreenChar
    }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color_code = ColorCode::new(fg, bg);
    }

    pub fn clear_screen(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        unsafe {
            for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
                Self::buffer().add(i).write_volatile(blank);
            }
        }
        self.column = 0;
        self.row = 0;
        self.update_hardware_cursor();
    }

    pub fn clear_row(&mut self, row: usize) {
        if row >= VGA_HEIGHT { return; }
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        unsafe {
            for col in 0..VGA_WIDTH {
                Self::buffer().add(row * VGA_WIDTH + col).write_volatile(blank);
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => {
                self.column = 0;
                self.update_hardware_cursor();
            }
            b'\t' => {
                let spaces = 4 - (self.column % 4);
                for _ in 0..spaces {
                    self.write_byte(b' ');
                }
            }
            0x08 => {
                // Backspace
                if self.column > 0 {
                    self.column -= 1;
                    let blank = ScreenChar {
                        ascii_character: b' ',
                        color_code: self.color_code,
                    };
                    unsafe {
                        Self::buffer().add(self.row * VGA_WIDTH + self.column).write_volatile(blank);
                    }
                    self.update_hardware_cursor();
                }
            }
            _ => {
                if self.column >= VGA_WIDTH {
                    self.new_line();
                }
                let char_entry = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                unsafe {
                    Self::buffer().add(self.row * VGA_WIDTH + self.column).write_volatile(char_entry);
                }
                self.column += 1;
                self.update_hardware_cursor();
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn new_line(&mut self) {
        self.column = 0;
        if self.row < VGA_HEIGHT - 1 {
            self.row += 1;
        } else {
            self.scroll();
        }
        self.update_hardware_cursor();
    }

    pub fn scroll(&mut self) {
        unsafe {
            for row in 1..VGA_HEIGHT {
                for col in 0..VGA_WIDTH {
                    let from = Self::buffer().add(row * VGA_WIDTH + col).read_volatile();
                    Self::buffer().add((row - 1) * VGA_WIDTH + col).write_volatile(from);
                }
            }
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            for col in 0..VGA_WIDTH {
                Self::buffer().add((VGA_HEIGHT - 1) * VGA_WIDTH + col).write_volatile(blank);
            }
        }
        self.row = VGA_HEIGHT - 1;
        self.column = 0;
    }

    pub fn set_cursor(&mut self, col: usize, row: usize) {
        self.column = col.min(VGA_WIDTH - 1);
        self.row = row.min(VGA_HEIGHT - 1);
        self.update_hardware_cursor();
    }

    pub fn print_at(&mut self, col: usize, row: usize, text: &str, fg: Color, bg: Color) {
        let old_col = self.column;
        let old_row = self.row;
        let old_color = self.color_code;

        self.set_color(fg, bg);
        let mut cur_col = col;
        for byte in text.bytes() {
            if cur_col >= VGA_WIDTH { break; }
            let char_entry = ScreenChar {
                ascii_character: byte,
                color_code: self.color_code,
            };
            unsafe {
                Self::buffer().add(row * VGA_WIDTH + cur_col).write_volatile(char_entry);
            }
            cur_col += 1;
        }

        self.column = old_col;
        self.row = old_row;
        self.color_code = old_color;
        self.update_hardware_cursor();
    }

    pub fn fill_rect(&mut self, col: usize, row: usize, width: usize, height: usize, ch: u8, fg: Color, bg: Color) {
        let entry = ScreenChar {
            ascii_character: ch,
            color_code: ColorCode::new(fg, bg),
        };
        unsafe {
            for r in row..(row + height).min(VGA_HEIGHT) {
                for c in col..(col + width).min(VGA_WIDTH) {
                    Self::buffer().add(r * VGA_WIDTH + c).write_volatile(entry);
                }
            }
        }
    }

    pub fn draw_box(&mut self, x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color, double_border: bool) {
        if width < 2 || height < 2 { return; }
        let (tl, tr, bl, br, h_bar, v_bar) = if double_border {
            (201u8, 187u8, 200u8, 188u8, 205u8, 186u8) // ╔ ╗ ╚ ╝ ═ ║
        } else {
            (218u8, 191u8, 192u8, 217u8, 196u8, 179u8) // ┌ ┐ └ ┘ ─ │
        };

        let color = ColorCode::new(fg, bg);
        unsafe {
            // Corners
            Self::buffer().add(y * VGA_WIDTH + x).write_volatile(ScreenChar { ascii_character: tl, color_code: color });
            Self::buffer().add(y * VGA_WIDTH + (x + width - 1)).write_volatile(ScreenChar { ascii_character: tr, color_code: color });
            Self::buffer().add((y + height - 1) * VGA_WIDTH + x).write_volatile(ScreenChar { ascii_character: bl, color_code: color });
            Self::buffer().add((y + height - 1) * VGA_WIDTH + (x + width - 1)).write_volatile(ScreenChar { ascii_character: br, color_code: color });

            // Horizontal borders
            for c in (x + 1)..(x + width - 1) {
                Self::buffer().add(y * VGA_WIDTH + c).write_volatile(ScreenChar { ascii_character: h_bar, color_code: color });
                Self::buffer().add((y + height - 1) * VGA_WIDTH + c).write_volatile(ScreenChar { ascii_character: h_bar, color_code: color });
            }

            // Vertical borders
            for r in (y + 1)..(y + height - 1) {
                Self::buffer().add(r * VGA_WIDTH + x).write_volatile(ScreenChar { ascii_character: v_bar, color_code: color });
                Self::buffer().add(r * VGA_WIDTH + (x + width - 1)).write_volatile(ScreenChar { ascii_character: v_bar, color_code: color });
            }
        }
    }

    pub fn enable_cursor(&self, cursor_start: u8, cursor_end: u8) {
        unsafe {
            outb(0x3D4, 0x0A);
            outb(0x3D5, (inb(0x3D5) & 0xC0) | cursor_start);
            outb(0x3D4, 0x0B);
            outb(0x3D5, (inb(0x3D5) & 0xE0) | cursor_end);
        }
    }

    pub fn disable_cursor(&self) {
        unsafe {
            outb(0x3D4, 0x0A);
            outb(0x3D5, 0x20);
        }
    }

    fn update_hardware_cursor(&self) {
        let pos = (self.row * VGA_WIDTH + self.column) as u16;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    unsafe {
        let _ = VGA_WRITER.write_fmt(args);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
