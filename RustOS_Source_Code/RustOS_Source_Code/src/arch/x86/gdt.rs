//! Global Descriptor Table (GDT) for 32-bit x86 Protected Mode

use core::arch::asm;

#[repr(C, packed)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    pub const fn new(base: u32, limit: u32, access: u8, gran: u8) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | (gran & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    pub const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }
}

#[repr(C, packed)]
pub struct GdtPointer {
    limit: u16,
    base: u32,
}

static mut GDT_ENTRIES: [GdtEntry; 3] = [
    GdtEntry::null(),                                         // 0x00: Null
    GdtEntry::new(0, 0xFFFFF, 0x9A, 0xCF),                   // 0x08: Kernel Code (32-bit, ring 0, RX)
    GdtEntry::new(0, 0xFFFFF, 0x92, 0xCF),                   // 0x10: Kernel Data (32-bit, ring 0, RW)
];

static mut GDT_PTR: GdtPointer = GdtPointer { limit: 0, base: 0 };

pub unsafe fn init_gdt() {
    GDT_PTR.limit = (core::mem::size_of_val(&GDT_ENTRIES) - 1) as u16;
    GDT_PTR.base = core::ptr::addr_of!(GDT_ENTRIES) as *const _ as u32;

    asm!(
        "lgdt [{0}]",
        "push 0x08",
        "lea {tmp}, [2f]",
        "push {tmp}",
        "retf",
        "2:",
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ss, ax",
        in(reg) core::ptr::addr_of!(GDT_PTR),
        tmp = out(reg) _,
    );
}
