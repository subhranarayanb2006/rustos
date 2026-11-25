#!/usr/bin/env python3
"""
RustOS Unified Build System
Builds sysroot, compiles kernel, links ELF multiboot binary, and produces bootable ISO.
"""

import os
import shutil
import subprocess
import sys

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC_DIR = os.path.join(BASE_DIR, "src")
TARGET_DIR = os.path.join(BASE_DIR, "target")
SYSROOT_DIR = os.path.join(TARGET_DIR, "sysroot")
ISODIR = os.path.join(BASE_DIR, "isodir")
DELIVERABLES_DIR = os.path.join(BASE_DIR, "deliverables")

RUSTC = os.path.expanduser("~/.rustup/toolchains/nightly-2024-05-15-x86_64-pc-windows-msvc/bin/rustc.exe")
if not os.path.exists(RUSTC):
    # Try system rustc
    RUSTC = "rustc"

LLD = os.path.expanduser("~/.rustup/toolchains/nightly-2024-05-15-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe")
if not os.path.exists(LLD):
    LLD = "rust-lld"

TARGET_JSON = os.path.join(BASE_DIR, "i686-rustos.json")
LINKER_LD = os.path.join(BASE_DIR, "linker.ld")
KERNEL_BIN = os.path.join(ISODIR, "boot", "rustos.bin")
GRUB_CFG = os.path.join(ISODIR, "boot", "grub", "grub.cfg")
OUTPUT_ISO = os.path.join(DELIVERABLES_DIR, "rustos.iso")

def step(title: str):
    print(f"\n========================================================")
    print(f"[*] {title}")
    print(f"========================================================")

def run_cmd(cmd, desc=""):
    print(f"[RUN] {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        print(f"[ERROR] {desc} (code {res.returncode}):")
        if res.stderr:
            print(res.stderr)
        if res.stdout:
            print(res.stdout)
        sys.exit(res.returncode)
    return res

def build_sysroot():
    step("1. Verifying / Building 32-bit x86 Sysroot (core, compiler_builtins, alloc)")
    os.makedirs(SYSROOT_DIR, exist_ok=True)
    
    libcore = os.path.join(SYSROOT_DIR, "libcore.rlib")
    libcb = os.path.join(SYSROOT_DIR, "libcompiler_builtins.rlib")
    liballoc = os.path.join(SYSROOT_DIR, "liballoc.rlib")
    
    if os.path.exists(libcore) and os.path.exists(libcb) and os.path.exists(liballoc):
        print("[OK] Sysroot already built in:", SYSROOT_DIR)
        return

    # Find rust-src
    toolchain_dir = os.path.expanduser("~/.rustup/toolchains/nightly-2024-05-15-x86_64-pc-windows-msvc")
    core_src = os.path.join(toolchain_dir, "lib/rustlib/src/rust/library/core/src/lib.rs")
    alloc_src = os.path.join(toolchain_dir, "lib/rustlib/src/rust/library/alloc/src/lib.rs")
    cb_src = os.path.expanduser("~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/compiler_builtins-0.1.109/src/lib.rs")

    run_cmd([
        RUSTC, "--edition", "2021", "--target", TARGET_JSON,
        "-L", SYSROOT_DIR, "--crate-type", "rlib", "--crate-name", "core",
        core_src, "--out-dir", SYSROOT_DIR
    ], "Compiling core")

    run_cmd([
        RUSTC, "--edition", "2021", "--target", TARGET_JSON,
        "-L", SYSROOT_DIR, "--crate-type", "rlib", "--crate-name", "compiler_builtins",
        cb_src, "--cfg", 'feature="compiler-builtins"', "--cfg", 'feature="mem"',
        "--out-dir", SYSROOT_DIR
    ], "Compiling compiler_builtins")

    run_cmd([
        RUSTC, "--edition", "2021", "--target", TARGET_JSON,
        "-L", SYSROOT_DIR, "--crate-type", "rlib", "--crate-name", "alloc",
        alloc_src, "--out-dir", SYSROOT_DIR
    ], "Compiling alloc")

    print("[SUCCESS] Sysroot ready.")

def compile_kernel():
    step("2. Compiling RustOS Kernel (src/main.rs -> target/kernel.o)")
    os.makedirs(os.path.join(ISODIR, "boot"), exist_ok=True)
    os.makedirs(os.path.join(ISODIR, "boot", "grub"), exist_ok=True)
    os.makedirs(TARGET_DIR, exist_ok=True)
    
    kernel_obj = os.path.join(TARGET_DIR, "kernel.o")
    main_rs = os.path.join(SRC_DIR, "main.rs")

    run_cmd([
        RUSTC,
        "--edition", "2021",
        "--target", TARGET_JSON,
        "-L", SYSROOT_DIR,
        "-O",
        "-C", "panic=abort",
        "-C", "lto=no",
        "--emit", f"obj={kernel_obj}",
        main_rs
    ], "Compiling Kernel Object")

    step("3. Linking Kernel ELF Binary (rust-lld -> isodir/boot/rustos.bin)")
    run_cmd([
        LLD,
        "-flavor", "gnu",
        "-m", "elf_i386",
        "-T", LINKER_LD,
        "-o", KERNEL_BIN,
        kernel_obj,
        os.path.join(SYSROOT_DIR, "libcore.rlib"),
        os.path.join(SYSROOT_DIR, "libcompiler_builtins.rlib"),
        os.path.join(SYSROOT_DIR, "liballoc.rlib"),
    ], "Linking Kernel Binary")

    bin_size = os.path.getsize(KERNEL_BIN)
    print(f"[SUCCESS] Kernel binary built: {KERNEL_BIN} ({bin_size} bytes)")

def build_iso():
    step("4. Building Bootable ISO (deliverables/rustos.iso)")
    os.makedirs(DELIVERABLES_DIR, exist_ok=True)

    # Check if grub-mkrescue is available
    grub_rescue = shutil.which("grub-mkrescue")
    if grub_rescue:
        print(f"[INFO] Using host grub-mkrescue: {grub_rescue}")
        res = subprocess.run([grub_rescue, "-o", OUTPUT_ISO, ISODIR], capture_output=True, text=True)
        if res.returncode == 0 and os.path.exists(OUTPUT_ISO):
            print(f"[SUCCESS] GRUB ISO generated: {OUTPUT_ISO}")
            return
        else:
            print("[WARN] grub-mkrescue failed or returned error. Falling back to internal ISO generator...")

    # Internal ISO generator
    make_iso_py = os.path.join(BASE_DIR, "tools", "make_iso.py")
    run_cmd([
        sys.executable,
        make_iso_py,
        OUTPUT_ISO,
        KERNEL_BIN,
        GRUB_CFG
    ], "Generating Bootable El Torito ISO")

    # Also copy to isodir/rustos.iso for convenience
    shutil.copy2(OUTPUT_ISO, os.path.join(ISODIR, "rustos.iso"))
    print(f"[SUCCESS] Bootable ISO delivered to: {OUTPUT_ISO}")

def clean():
    step("Cleaning build artifacts")
    for d in [TARGET_DIR, DELIVERABLES_DIR]:
        if os.path.exists(d):
            shutil.rmtree(d)
            print(f"Removed {d}")

def main():
    if "--clean" in sys.argv:
        clean()
        return

    build_sysroot()
    compile_kernel()
    if "--iso" in sys.argv or True: # default build iso
        build_iso()

if __name__ == "__main__":
    main()
