//! About RustOS Application

use crate::drivers::vga::{VGA_WRITER, Color, VGA_WIDTH};
use crate::drivers::keyboard::{pop_key, KeyEvent};

pub fn launch_about() {
    unsafe {
        VGA_WRITER.clear_screen();

        // Top Banner
        VGA_WRITER.fill_rect(0, 0, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 0, "ABOUT RUSTOS: MODERN EDUCATIONAL OPERATING SYSTEM", Color::LightCyan, Color::DarkGray);
        VGA_WRITER.print_at(2, 1, "Designed & Engineered in 100% Rust for Oracle VirtualBox", Color::LightGray, Color::DarkGray);

        // Center Box
        VGA_WRITER.draw_box(4, 3, 72, 18, Color::LightCyan, Color::Black, true);
        VGA_WRITER.print_at(6, 4, "RUSTOS ARCHITECTURAL MISSION & OVERVIEW", Color::Yellow, Color::Black);

        VGA_WRITER.print_at(6, 6, "RustOS demonstrates that memory safety, zero-cost abstractions,", Color::White, Color::Black);
        VGA_WRITER.print_at(6, 7, "and modern software engineering principles can be applied directly", Color::White, Color::Black);
        VGA_WRITER.print_at(6, 8, "to bare-metal x86 operating system design without C legacy traps.", Color::White, Color::Black);

        VGA_WRITER.print_at(6, 10, "CORE SYSTEM HIGHLIGHTS:", Color::Yellow, Color::Black);
        VGA_WRITER.print_at(8, 11, "- Multiboot 1 compliant x86 bootloader & linker script", Color::LightGreen, Color::Black);
        VGA_WRITER.print_at(8, 12, "- Intel 82540EM (e1000) PCI driver with full DHCP & DNS stack", Color::LightGreen, Color::Black);
        VGA_WRITER.print_at(8, 13, "- Built-in text-mode web browser with HTML tokenization", Color::LightGreen, Color::Black);
        VGA_WRITER.print_at(8, 14, "- Deadlock Detection Research Lab with empirical reduction suite", Color::LightGreen, Color::Black);
        VGA_WRITER.print_at(8, 15, "- In-memory RAMFS with 33 standard Unix-compatible shell commands", Color::LightGreen, Color::Black);

        VGA_WRITER.print_at(6, 17, "Engineered by: RustOS Core Architecture Team", Color::LightCyan, Color::Black);
        VGA_WRITER.print_at(6, 18, "License:       MIT / Open Source Academic License", Color::LightGray, Color::Black);

        // Bottom Bar
        VGA_WRITER.fill_rect(0, 23, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 23, "[Esc] Return to Desktop", Color::Yellow, Color::DarkGray);
    }

    loop {
        if let Some(key) = pop_key() {
            if key == KeyEvent::Esc {
                break;
            }
        }
        crate::drivers::pit::sleep_ms(100);
    }
}
