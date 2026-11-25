//! RustOS Kernel Panic Handler

use core::panic::PanicInfo;
use core::arch::asm;
use crate::drivers::vga::{VGA_WRITER, Color};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }

    crate::serial_println!("\n=================== KERNEL PANIC ===================");
    if let Some(location) = info.location() {
        crate::serial_println!("Location: {}:{}:{}", location.file(), location.line(), location.column());
    }
    if let Some(msg) = info.message() {
        crate::serial_println!("Message:  {}", msg);
    } else {
        crate::serial_println!("Message:  (No panic message provided)");
    }
    crate::serial_println!("====================================================\n");

    unsafe {
        VGA_WRITER.clear_screen();
        VGA_WRITER.fill_rect(5, 4, 70, 16, b' ', Color::White, Color::Red);
        VGA_WRITER.draw_box(5, 4, 70, 16, Color::Yellow, Color::Red, true);
        
        VGA_WRITER.print_at(26, 6, " !! RUSTOS KERNEL PANIC !! ", Color::Yellow, Color::Red);
        
        if let Some(location) = info.location() {
            let mut buf = [0u8; 64];
            let mut idx = 0;
            let prefix = b"At: ";
            for &b in prefix { buf[idx] = b; idx += 1; }
            for &b in location.file().as_bytes() {
                if idx >= 45 { break; }
                buf[idx] = b;
                idx += 1;
            }
            if let Ok(s) = core::str::from_utf8(&buf[..idx]) {
                VGA_WRITER.print_at(8, 8, s, Color::White, Color::Red);
            }
        }

        VGA_WRITER.print_at(8, 10, "System halted safely to prevent hardware corruption.", Color::LightCyan, Color::Red);
        VGA_WRITER.print_at(8, 12, "Please inspect the COM1 serial log for detailed trace.", Color::LightGray, Color::Red);
        VGA_WRITER.print_at(8, 15, "Press Host Key + R in VirtualBox to reset VM.", Color::Yellow, Color::Red);
    }

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}
