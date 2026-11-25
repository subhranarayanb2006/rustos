//! RustOS: Modern Educational 32-bit x86 Operating System in Rust
//! Designed and engineered for Oracle VirtualBox with full LAN/Internet networking,
//! built-in text web browser, interactive shell, and Deadlock Detection Research Lab.

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

pub mod arch;
pub mod drivers;
pub mod memory;
pub mod fs;
pub mod net;
pub mod shell;
pub mod ui;
pub mod apps;
pub mod panic;

use arch::x86::gdt::init_gdt;
use arch::x86::idt::init_idt;
use drivers::serial::SerialPort;
use drivers::vga::VgaWriter;
use drivers::pic::init_pic;
use drivers::pit::init_pit;
use drivers::keyboard::init_keyboard;
use memory::allocator::{ALLOCATOR, HEAP_START, HEAP_SIZE};
use fs::ramfs::init_ramfs;
use net::init_network;
use ui::desktop::run_desktop;

// Multiboot 1 Header and Entry Point
core::arch::global_asm!(
    r#"
    .section .multiboot, "a"
    .align 4
    .long 0x1BADB002                  /* Multiboot Magic */
    .long 0x00000003                  /* Flags: ALIGN (bit 0) | MEMINFO (bit 1) */
    .long -(0x1BADB002 + 0x00000003)  /* Checksum */

    .section .text
    .global _start
    .type _start, @function
    _start:
        cli
        cld
        lea edi, [sbss]
        lea ecx, [ebss]
        sub ecx, edi
        shr ecx, 2
        xor eax, eax
        rep stosd

        lea esp, [stack_top]
        push ebx                      /* Pointer to Multiboot Info structure */
        push 0x2BADB002               /* Multiboot Magic value (0x2BADB002) */
        call kernel_main

    .hang:
        cli
        hlt
        jmp .hang

    .section .bss
    .align 16
    stack_bottom:
        .skip 65536                   /* 64 KB Kernel Stack */
    stack_top:
    "#
);

#[no_mangle]
pub extern "C" fn kernel_main(multiboot_magic: u32, multiboot_info_addr: usize) -> ! {
    // 1. Establish 32-bit Flat Model GDT, IDT & Remap PIC immediately (safe environment)
    unsafe {
        init_gdt();
        init_idt();
        init_pic();
        SerialPort::init();
        VgaWriter::init();
    }

    serial_println!("==============================================================");
    serial_println!("   RustOS 1.0 (32-bit x86) - Multiboot Kernel Entry Point    ");
    serial_println!("==============================================================");
    serial_println!("[BOOT] Multiboot Magic: {:#010x} (Expected: 0x2badb002)", multiboot_magic);
    serial_println!("[BOOT] Multiboot Info:  {:#010x}", multiboot_info_addr);
    serial_println!("[GDT] 32-bit Flat Model GDT loaded successfully.");
    serial_println!("[IDT] 256 IDT gate descriptors registered.");
    serial_println!("[PIC] Dual 8259 PIC cascaded & remapped to 0x20-0x2F.");

    crate::println!("==============================================================");
    crate::println!("   RustOS 1.0 (32-bit x86) - Booting into VirtualBox...      ");
    crate::println!("==============================================================");
    crate::println!("[BOOT] GDT (Kernel Code 0x08, Data 0x10)...              [ OK ]");
    crate::println!("[BOOT] IDT (256 Gates & Exception Handlers)...           [ OK ]");
    crate::println!("[BOOT] Dual 8259 PIC Remapped (IRQ 0-15 -> 32-47)...      [ OK ]");

    // 2. Initialize 8254 PIT Timer (100 Hz = 10ms ticks)
    crate::print!("[BOOT] Initializing 8254 PIT (100 Hz Scheduler Ticks)...    ");
    unsafe {
        init_pit();
    }
    crate::println!("OK");
    serial_println!("[PIT] Timer channel 0 programmed to 100 Hz.");

    // 7. Initialize PS/2 Keyboard Controller
    crate::print!("[BOOT] Initializing PS/2 Keyboard Driver...                 ");
    unsafe {
        init_keyboard();
    }
    crate::println!("OK");
    serial_println!("[KBD] PS/2 keyboard buffer initialized.");

    // 8. Initialize 16 MB Coalescing Heap Allocator
    crate::print!("[BOOT] Initializing Kernel Heap (16 MB at 0x00400000)...     ");
    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
    crate::println!("OK");
    serial_println!("[MEM] 16 MB heap registered. GlobalAlloc active.");

    // 9. Enable Hardware Interrupts (STI)
    crate::print!("[BOOT] Enabling Hardware CPU Interrupts (STI)...            ");
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
    crate::println!("OK");
    serial_println!("[CPU] Hardware interrupts enabled.");

    // 10. Initialize In-Memory RAMFS (Pre-populated with readme, notes, todo, about)
    crate::print!("[BOOT] Mounting In-Memory RAM Filesystem...                 ");
    unsafe {
        init_ramfs();
    }
    crate::println!("OK");
    serial_println!("[FS] RAMFS mounted with 4 default educational documents.");

    // 11. Initialize PCI Bus & Network Subsystem (Intel 82540EM / VirtualBox NAT)
    crate::print!("[BOOT] Scanning PCI & Initializing Ethernet Driver...       ");
    unsafe {
        init_network();
    }
    crate::println!("OK");
    serial_println!("[NET] Network subsystem online.");

    // 12. Pre-generate Deadlock Detection Research Benchmark File
    crate::print!("[BOOT] Running Deadlock Detection Benchmark (10 Trials)...  ");
    let _suite = apps::research::experiments::run_research_suite();
    crate::println!("OK");
    serial_println!("[RESEARCH] Deadlock research benchmark completed and written to /deadlock_results.txt");

    crate::println!("\n[BOOT] All core subsystems initialized successfully!");
    crate::println!("[BOOT] Launching Modern Desktop Environment in 1 second...");
    drivers::pit::sleep_ms(800);

    // 13. Launch Desktop User Interface
    run_desktop();
}
