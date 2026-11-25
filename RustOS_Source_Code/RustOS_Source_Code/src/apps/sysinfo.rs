//! System Information Application

use alloc::format;
use crate::drivers::vga::{VGA_WRITER, Color, VGA_WIDTH};
use crate::drivers::keyboard::{pop_key, KeyEvent};
use crate::drivers::rtc::Rtc;
use crate::drivers::pit;
use crate::memory::allocator::{ALLOCATOR, HEAP_SIZE};
use crate::net::{get_net, get_mac, e1000::E1000};

pub fn launch_sysinfo() {
    loop {
        render_sysinfo();

        if let Some(key) = pop_key() {
            match key {
                KeyEvent::Esc => break,
                _ => {}
            }
        }
        pit::sleep_ms(100);
    }
}

fn render_sysinfo() {
    let dt = Rtc::read_datetime();
    let net = get_net();
    let mac = get_mac();
    let secs = pit::uptime_secs();
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;

    let heap_used = ALLOCATOR.allocated_bytes();
    let heap_free = ALLOCATOR.free_bytes();
    let alloc_count = ALLOCATOR.total_allocations();

    unsafe {
        VGA_WRITER.clear_screen();

        // Header
        VGA_WRITER.fill_rect(0, 0, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 0, "RUSTOS SYSTEM INFORMATION & HARDWARE DIAGNOSTICS", Color::LightCyan, Color::DarkGray);
        VGA_WRITER.print_at(2, 1, "Kernel Diagnostics, Memory Profiling & Peripheral Topology", Color::LightGray, Color::DarkGray);

        // System Box
        VGA_WRITER.draw_box(4, 3, 72, 18, Color::LightCyan, Color::Black, true);
        VGA_WRITER.print_at(6, 4, "SYSTEM SPECIFICATIONS", Color::Yellow, Color::Black);

        VGA_WRITER.print_at(6, 6, "Operating System: ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 6, "RustOS (Modern Educational 32-bit x86)", Color::LightCyan, Color::Black);

        VGA_WRITER.print_at(6, 7, "Kernel Version:   ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 7, "1.0.0-release (nightly-2024-05-15 bare-metal)", Color::LightGreen, Color::Black);

        VGA_WRITER.print_at(6, 8, "Target Platform:  ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 8, "Oracle VirtualBox Hypervisor (BIOS / El Torito)", Color::White, Color::Black);

        VGA_WRITER.print_at(6, 9, "CPU Architecture: ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 9, "i686 (32-bit x86 Protected Mode, GDT/IDT Active)", Color::LightCyan, Color::Black);

        VGA_WRITER.print_at(6, 11, "Kernel Heap RAM:  ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 11, &format!("{} MB total (16,777,216 bytes)", HEAP_SIZE / (1024 * 1024)), Color::Yellow, Color::Black);

        VGA_WRITER.print_at(6, 12, "Heap Allocation:  ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 12, &format!("{} KB used ({} ops)  |  {} KB free", heap_used / 1024, alloc_count, heap_free / 1024), Color::LightGreen, Color::Black);

        VGA_WRITER.print_at(6, 13, "System Uptime:    ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 13, &format!("{:02}:{:02}:{:02} (Timer Ticks: {})", hours, mins, s, pit::get_ticks()), Color::White, Color::Black);

        VGA_WRITER.print_at(6, 14, "RTC Date & Time:  ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 14, &format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second), Color::Yellow, Color::Black);

        VGA_WRITER.print_at(6, 16, "Network Card:     ", Color::White, Color::Black);
        VGA_WRITER.print_at(25, 16, &net.adapter_name, Color::LightCyan, Color::Black);

        VGA_WRITER.print_at(6, 17, "MAC / IPv4:       ", Color::White, Color::Black);
        let mac_str = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
        let ip_str = format!("{}.{}.{}.{}", net.config.ip[0], net.config.ip[1], net.config.ip[2], net.config.ip[3]);
        VGA_WRITER.print_at(25, 17, &format!("{} | IP: {}", mac_str, ip_str), Color::LightGreen, Color::Black);

        // Bottom Bar
        VGA_WRITER.fill_rect(0, 23, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 23, "[Esc] Return to Desktop", Color::Yellow, Color::DarkGray);
    }
}
