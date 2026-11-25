//! Interrupt Descriptor Table (IDT) for 32-bit x86 Protected Mode

use core::arch::asm;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    base_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    base_high: u16,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        Self {
            base_low: 0,
            selector: 0,
            zero: 0,
            type_attr: 0,
            base_high: 0,
        }
    }

    pub fn set_handler(&mut self, handler: usize, selector: u16, flags: u8) {
        self.base_low = (handler & 0xFFFF) as u16;
        self.selector = selector;
        self.zero = 0;
        self.type_attr = flags;
        self.base_high = ((handler >> 16) & 0xFFFF) as u16;
    }
}

#[repr(C, packed)]
pub struct IdtPointer {
    limit: u16,
    base: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InterruptContext {
    pub gs: u32,
    pub fs: u32,
    pub es: u32,
    pub ds: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp_dummy: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub int_no: u32,
    pub err_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];
static mut IDT_PTR: IdtPointer = IdtPointer { limit: 0, base: 0 };

extern "C" {
    fn isr_stub_table_entry(index: usize) -> usize;
}

pub unsafe fn init_idt() {
    IDT_PTR.limit = (core::mem::size_of_val(&IDT) - 1) as u16;
    IDT_PTR.base = &IDT as *const _ as u32;

    for i in 0..256 {
        let handler = isr_stub_table(i);
        IDT[i].set_handler(handler, 0x08, 0x8E); // 32-bit Interrupt Gate, Ring 0
    }

    asm!("lidt [{0}]", in(reg) &IDT_PTR, options(nostack));
}

// Generate table of 256 ISR stubs
macro_rules! isr_stub {
    ($num:expr, no_err) => {
        core::arch::global_asm!(concat!(
            ".global isr_", stringify!($num), "\n",
            "isr_", stringify!($num), ":\n",
            "    push 0\n",
            "    push ", stringify!($num), "\n",
            "    jmp isr_common_stub\n"
        ));
    };
    ($num:expr, has_err) => {
        core::arch::global_asm!(concat!(
            ".global isr_", stringify!($num), "\n",
            "isr_", stringify!($num), ":\n",
            "    push ", stringify!($num), "\n",
            "    jmp isr_common_stub\n"
        ));
    };
}

// Global assembly for common ISR dispatcher
core::arch::global_asm!(
    r#"
    .global isr_common_stub
    isr_common_stub:
        push eax
        push ecx
        push edx
        push ebx
        push esp
        push ebp
        push esi
        push edi
        push ds
        push es
        push fs
        push gs

        mov ax, 0x10
        mov ds, ax
        mov es, ax
        mov fs, ax
        mov gs, ax

        push esp
        call rust_interrupt_handler
        add esp, 4

        pop gs
        pop fs
        pop es
        pop ds
        pop edi
        pop esi
        pop ebp
        add esp, 4
        pop ebx
        pop edx
        pop ecx
        pop eax

        add esp, 8
        iretd
    "#
);

// We define handlers for CPU exceptions (0-31) and IRQs (32-47)
isr_stub!(0, no_err);
isr_stub!(1, no_err);
isr_stub!(2, no_err);
isr_stub!(3, no_err);
isr_stub!(4, no_err);
isr_stub!(5, no_err);
isr_stub!(6, no_err);
isr_stub!(7, no_err);
isr_stub!(8, has_err);
isr_stub!(9, no_err);
isr_stub!(10, has_err);
isr_stub!(11, has_err);
isr_stub!(12, has_err);
isr_stub!(13, has_err);
isr_stub!(14, has_err);
isr_stub!(15, no_err);
isr_stub!(16, no_err);
isr_stub!(17, has_err);
isr_stub!(18, no_err);
isr_stub!(19, no_err);
isr_stub!(20, no_err);
isr_stub!(21, has_err);
isr_stub!(22, no_err);
isr_stub!(23, no_err);
isr_stub!(24, no_err);
isr_stub!(25, no_err);
isr_stub!(26, no_err);
isr_stub!(27, no_err);
isr_stub!(28, no_err);
isr_stub!(29, has_err);
isr_stub!(30, has_err);
isr_stub!(31, no_err);

// IRQs (32-47)
isr_stub!(32, no_err); // IRQ 0: PIT Timer
isr_stub!(33, no_err); // IRQ 1: PS/2 Keyboard
isr_stub!(34, no_err); // IRQ 2: Cascade
isr_stub!(35, no_err); // IRQ 3: COM2
isr_stub!(36, no_err); // IRQ 4: COM1
isr_stub!(37, no_err); // IRQ 5: LPT2
isr_stub!(38, no_err); // IRQ 6: Floppy
isr_stub!(39, no_err); // IRQ 7: LPT1 / Spurious
isr_stub!(40, no_err); // IRQ 8: RTC
isr_stub!(41, no_err); // IRQ 9: Free / NIC
isr_stub!(42, no_err); // IRQ 10: Free / NIC (e1000)
isr_stub!(43, no_err); // IRQ 11: Free / NIC
isr_stub!(44, no_err); // IRQ 12: PS/2 Mouse
isr_stub!(45, no_err); // IRQ 13: FPU
isr_stub!(46, no_err); // IRQ 14: Primary IDE
isr_stub!(47, no_err); // IRQ 15: Secondary IDE

extern "C" {
    fn isr_0(); fn isr_1(); fn isr_2(); fn isr_3(); fn isr_4(); fn isr_5(); fn isr_6(); fn isr_7();
    fn isr_8(); fn isr_9(); fn isr_10(); fn isr_11(); fn isr_12(); fn isr_13(); fn isr_14(); fn isr_15();
    fn isr_16(); fn isr_17(); fn isr_18(); fn isr_19(); fn isr_20(); fn isr_21(); fn isr_22(); fn isr_23();
    fn isr_24(); fn isr_25(); fn isr_26(); fn isr_27(); fn isr_28(); fn isr_29(); fn isr_30(); fn isr_31();
    fn isr_32(); fn isr_33(); fn isr_34(); fn isr_35(); fn isr_36(); fn isr_37(); fn isr_38(); fn isr_39();
    fn isr_40(); fn isr_41(); fn isr_42(); fn isr_43(); fn isr_44(); fn isr_45(); fn isr_46(); fn isr_47();
}

fn isr_stub_table(i: usize) -> usize {
    match i {
        0 => isr_0 as usize, 1 => isr_1 as usize, 2 => isr_2 as usize, 3 => isr_3 as usize,
        4 => isr_4 as usize, 5 => isr_5 as usize, 6 => isr_6 as usize, 7 => isr_7 as usize,
        8 => isr_8 as usize, 9 => isr_9 as usize, 10 => isr_10 as usize, 11 => isr_11 as usize,
        12 => isr_12 as usize, 13 => isr_13 as usize, 14 => isr_14 as usize, 15 => isr_15 as usize,
        16 => isr_16 as usize, 17 => isr_17 as usize, 18 => isr_18 as usize, 19 => isr_19 as usize,
        20 => isr_20 as usize, 21 => isr_21 as usize, 22 => isr_22 as usize, 23 => isr_23 as usize,
        24 => isr_24 as usize, 25 => isr_25 as usize, 26 => isr_26 as usize, 27 => isr_27 as usize,
        28 => isr_28 as usize, 29 => isr_29 as usize, 30 => isr_30 as usize, 31 => isr_31 as usize,
        32 => isr_32 as usize, 33 => isr_33 as usize, 34 => isr_34 as usize, 35 => isr_35 as usize,
        36 => isr_36 as usize, 37 => isr_37 as usize, 38 => isr_38 as usize, 39 => isr_39 as usize,
        40 => isr_40 as usize, 41 => isr_41 as usize, 42 => isr_42 as usize, 43 => isr_43 as usize,
        44 => isr_44 as usize, 45 => isr_45 as usize, 46 => isr_46 as usize, 47 => isr_47 as usize,
        _ => isr_31 as usize,
    }
}
