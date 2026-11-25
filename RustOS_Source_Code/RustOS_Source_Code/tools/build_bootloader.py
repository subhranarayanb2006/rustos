import os
import subprocess
import struct

boot_s = """
#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

core::arch::global_asm!(
    ".code16",
    ".global _start",
    "_start:",
    "    cli",
    "    cld",
    "    xor ax, ax",
    "    mov ds, ax",
    "    mov es, ax",
    "    mov ss, ax",
    "    mov sp, 0x7c00",
    "    mov [boot_drive], dl",

    // Print loading message
    "    mov si, offset msg_boot",
    "print_msg:",
    "    lodsb",
    "    test al, al",
    "    jz a20_enable",
    "    mov ah, 0x0e",
    "    int 0x10",
    "    jmp print_msg",

    // Enable A20 Line
    "a20_enable:",
    "    mov ax, 0x2401",
    "    int 0x15",
    "    in al, 0x92",
    "    or al, 2",
    "    and al, 0xFE",
    "    out 0x92, al",

    // Read kernel using BIOS Int 13h Extensions + Int 15h Block Move
    // We read CHUNK_SECTORS (16 sectors = 32KB) at a time
    "load_kernel:",
    "    mov ax, [sectors_left]",
    "    test ax, ax",
    "    jz boot_done",
    "    cmp ax, 16",
    "    jbe set_chunk",
    "    mov ax, 16",
    "set_chunk:",
    "    mov [dap_count], ax",
    "    shl ax, 10",            // ax * 1024 = word count for Int 15h AH=87h (16 * 1024 = 16384 words = 32KB)
    "    mov [word_count], ax",

    // 1. Read from CD into low buffer 0x1000:0000 (0x10000 physical)
    "    mov dl, [boot_drive]",
    "    mov si, offset dap_packet",
    "    mov ah, 0x42",
    "    int 0x13",
    "    jc disk_error",

    // 2. Prepare Int 15h AH=87h GDT descriptors
    // Source: 0x00010000
    // Dest:   [dest_phys]
    "    mov eax, [dest_phys]",
    "    mov [desc_dst_base_low], ax",
    "    shr eax, 16",
    "    mov [desc_dst_base_mid], al",
    "    mov [desc_dst_base_high], ah",

    "    mov ah, 0x87",
    "    mov cx, [word_count]",
    "    mov si, offset int15_gdt",
    "    int 0x15",
    "    jc copy_error",

    // 3. Advance pointers
    "    movzx eax, word ptr [dap_count]",
    "    sub [sectors_left], ax",
    "    add [dap_lba_low], eax",
    "    adc dword ptr [dap_lba_high], 0",
    "    shl eax, 11",           // sectors * 2048 bytes
    "    add [dest_phys], eax",
    "    jmp load_kernel",

    "boot_done:",
    // Re-enable A20 Line post-Int15h loop!
    "    mov ax, 0x2401",
    "    int 0x15",
    "    in al, 0x92",
    "    or al, 2",
    "    and al, 0xFE",
    "    out 0x92, al",

    // Transition to 32-bit Protected Mode
    "    cli",
    "    lgdt [pm_gdt_ptr]",
    "    mov eax, cr0",
    "    or eax, 1",
    "    mov cr0, eax",
    "    ljmp 0x08, offset prot_mode_entry",

    "disk_error:",
    "    mov si, offset msg_d_err",
    "    jmp print_err",
    "copy_error:",
    "    mov si, offset msg_c_err",
    "print_err:",
    "    lodsb",
    "    test al, al",
    "    jz halt_loop",
    "    mov ah, 0x0e",
    "    int 0x10",
    "    jmp print_err",
    "halt_loop:",
    "    hlt",
    "    jmp halt_loop",

    // 32-bit Protected Mode
    ".code32",
    "prot_mode_entry:",
    "    mov ax, 0x10",
    "    mov ds, ax",
    "    mov es, ax",
    "    mov fs, ax",
    "    mov gs, ax",
    "    mov ss, ax",
    "    mov esp, 0x90000",

    // Re-verify Fast A20 in protected mode
    "    in al, 0x92",
    "    or al, 2",
    "    and al, 0xFE",
    "    out 0x92, al",

    // Setup Multiboot Info structure at 0x9000
    "    mov dword ptr [0x9000], 0x00000003",
    "    mov dword ptr [0x9004], 640",
    "    mov dword ptr [0x9008], 131072",
    "    mov dword ptr [0x900C], 0xE0000000",
    "    mov dword ptr [0x9010], 0x9100",
    "    mov dword ptr [0x9100], 0x74737572", // 'rust'
    "    mov word ptr [0x9104], 0x736F",      // 'os'
    "    mov byte ptr [0x9106], 0",

    // Jump to 32-bit Kernel Entry Point
    "    mov edx, [kernel_entry_addr]",
    "    mov eax, 0x2BADB002",
    "    mov ebx, 0x9000",
    "    jmp edx",

    // Data structures
    ".code16",
    ".align 4",
    "boot_drive:         .byte 0",
    "                    .byte 0",
    "word_count:         .word 0",
    "sectors_left:       .word 0",
    "dest_phys:          .long 0x00100000",
    "kernel_entry_addr:   .long 0x001011DF",

    "msg_boot:           .asciz \\\"Booting RustOS 1.0 (VirtualBox)...\\\\r\\\\n\\\"",
    "msg_d_err:          .asciz \\\"CD Read Error\\\\r\\\\n\\\"",
    "msg_c_err:          .asciz \\\"Memory Copy Error\\\\r\\\\n\\\"",

    // DAP Packet for Int 13h AH=42h
    ".align 4",
    "dap_packet:",
    "dap_size:           .byte 0x10",
    "dap_res:            .byte 0",
    "dap_count:          .word 1",
    "dap_buf_off:        .word 0x0000",
    "dap_buf_seg:        .word 0x1000",
    "dap_lba_low:        .long 0",
    "dap_lba_high:       .long 0",

    // Int 15h AH=87h GDT Table (48 bytes: 6 descriptors of 8 bytes)
    ".align 8",
    "int15_gdt:",
    "    .quad 0",                       // 0x00: Dummy descriptor
    "    .quad 0",                       // 0x08: GDT itself
    // Source: 0x00010000 (base=0x00010000, limit=0xFFFF, type=0x93)
    "    .word 0xFFFF",                  // limit 15..0
    "    .word 0x0000",                  // base 15..0
    "    .byte 0x01",                    // base 23..16
    "    .byte 0x93",                    // access: present, ring 0, data read/write
    "    .byte 0x00",                    // granularity/flags
    "    .byte 0x00",                    // base 31..24
    // Dest:   [dest_phys] (base dynamic, limit=0xFFFF, type=0x93)
    "    .word 0xFFFF",
    "desc_dst_base_low:",
    "    .word 0x0000",
    "desc_dst_base_mid:",
    "    .byte 0x10",
    "    .byte 0x93",
    "    .byte 0x00",
    "desc_dst_base_high:",
    "    .byte 0x00",
    "    .quad 0",                       // 0x20: BIOS CS
    "    .quad 0",                       // 0x28: BIOS SS

    // Protected Mode GDT for kernel handoff
    ".align 8",
    "pm_gdt:",
    "    .quad 0",                       // Null
    "    .quad 0x00CF9A000000FFFF",       // Code: 0x08
    "    .quad 0x00CF92000000FFFF",       // Data: 0x10
    "pm_gdt_ptr:",
    "    .word 23",
    "    .long pm_gdt"
);
"""

with open("target/bootloader.rs", "w") as f:
    f.write(boot_s)

print("Compiling bootloader with rustc...")
res = subprocess.run([
    r"C:\Users\Subhranshu\.rustup\toolchains\nightly-2024-05-15-x86_64-pc-windows-msvc\bin\rustc.exe",
    "--target", "i686-rustos.json",
    "-L", "target/sysroot",
    "--emit", "obj=target/bootloader.o",
    "target/bootloader.rs"
], capture_output=True, text=True)

if res.returncode != 0:
    print("Compilation failed:")
    print(res.stderr)
    exit(1)

print("Linking bootloader with rust-lld...")
res = subprocess.run([
    r"C:\Users\Subhranshu\.rustup\toolchains\nightly-2024-05-15-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe",
    "-flavor", "gnu",
    "-m", "elf_i386",
    "--oformat=binary",
    "--Ttext", "0x7C00",
    "-o", "target/bootloader.bin",
    "target/bootloader.o"
], capture_output=True, text=True)

if res.returncode != 0:
    print("Link failed:")
    print(res.stderr)
    exit(1)

boot_bin = "target/bootloader.bin"
size = os.path.getsize(boot_bin)
print(f"Bootloader compiled & linked successfully! Size: {size} bytes")
