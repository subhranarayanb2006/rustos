//! 8259 Programmable Interrupt Controller (PIC) Driver

use crate::arch::x86::io::{inb, outb, io_wait};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16    = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16    = 0xA1;

const PIC_EOI: u8       = 0x20;

const ICW1_INIT: u8     = 0x10;
const ICW1_ICW4: u8     = 0x01;
const ICW4_8086: u8     = 0x01;

pub unsafe fn init_pic() {
    let _mask1 = inb(PIC1_DATA);
    let _mask2 = inb(PIC2_DATA);

    // ICW1: Start initialization in cascade mode
    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();

    // ICW2: Vector offsets (32 for master, 40 for slave)
    outb(PIC1_DATA, 0x20); // IRQ 0..7 -> int 32..39
    io_wait();
    outb(PIC2_DATA, 0x28); // IRQ 8..15 -> int 40..47
    io_wait();

    // ICW3: Tell Master PIC there is a slave at IRQ 2 (0000 0100)
    outb(PIC1_DATA, 0x04);
    io_wait();
    // Tell Slave PIC its cascade identity (2)
    outb(PIC2_DATA, 0x02);
    io_wait();

    // ICW4: Set 8086 mode
    outb(PIC1_DATA, ICW4_8086);
    io_wait();
    outb(PIC2_DATA, ICW4_8086);
    io_wait();

    // Restore masks, unmasking IRQ 0 (Timer) and IRQ 1 (Keyboard)
    // IRQ 2 (Cascade) must also be unmasked for slave IRQs
    outb(PIC1_DATA, 0b1111_1000); // Unmask IRQ 0 (Timer), IRQ 1 (Keyboard), IRQ 2 (Cascade)
    outb(PIC2_DATA, 0b1110_1111); // Unmask IRQ 10/11 or 12 for NIC/mouse if needed
}

pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

pub fn unmask_irq(irq: u8) {
    unsafe {
        let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
        let shift = if irq < 8 { irq } else { irq - 8 };
        let mask = inb(port) & !(1 << shift);
        outb(port, mask);
    }
}
