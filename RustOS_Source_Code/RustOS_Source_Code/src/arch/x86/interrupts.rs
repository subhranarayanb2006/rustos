//! Rust Interrupt Dispatcher and Exception Handlers

use crate::arch::x86::idt::InterruptContext;
use crate::drivers::pic::send_eoi;
use crate::drivers::pit;
use crate::drivers::keyboard;
use crate::serial_println;

static EXCEPTION_MESSAGES: [&str; 32] = [
    "Divide by Zero",
    "Debug",
    "Non-Maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available (No Math Coprocessor)",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Unknown Interrupt",
    "x87 FPU Floating-Point Error",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Control Protection Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Hypervisor Injection Exception",
    "VMM Communication Exception",
    "Security Exception",
    "Reserved",
];

#[no_mangle]
pub extern "C" fn rust_interrupt_handler(ctx: &InterruptContext) {
    if ctx.int_no < 32 {
        // CPU Exception
        let msg = if (ctx.int_no as usize) < EXCEPTION_MESSAGES.len() {
            EXCEPTION_MESSAGES[ctx.int_no as usize]
        } else {
            "Unknown Exception"
        };
        serial_println!(
            "[CPU EXCEPTION #{}] {}: ErrorCode={:#x}, EIP={:#x}, CS={:#x}, EFLAGS={:#x}",
            ctx.int_no, msg, ctx.err_code, ctx.eip, ctx.cs, ctx.eflags
        );
        panic!(
            "CPU EXCEPTION #{}: {} at EIP={:#010x}, ErrorCode={:#x}",
            ctx.int_no, msg, ctx.eip, ctx.err_code
        );
    } else {
        // Hardware IRQ (32-47)
        let irq = (ctx.int_no - 32) as u8;
        match irq {
            0 => {
                // PIT Timer Tick
                pit::on_timer_tick();
            }
            1 => {
                // PS/2 Keyboard Event
                keyboard::on_keyboard_interrupt();
            }
            12 => {
                // PS/2 Mouse Event: drain port 0x60 so output buffer doesn't clog
                unsafe {
                    if (crate::arch::x86::io::inb(0x64) & 1) != 0 {
                        let _ = crate::arch::x86::io::inb(0x60);
                    }
                }
            }
            // Other IRQs (e.g. NIC on IRQ 10/11)
            _ => {}
        }
        send_eoi(irq);
    }
}
