//! RustOS In-Memory RAM File System (RAMFS)
//! Supports file creation, deletion, copy, rename, editing, and permissions.
//! Files reside in kernel memory and do not persist across VM power-offs.

use alloc::string::String;
use alloc::vec::Vec;
use crate::drivers::rtc::{Rtc, DateTime};

pub const MAX_FILES: usize = 32;
pub const MAX_FILE_SIZE: usize = 2048;
pub const MAX_NAME_LEN: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FilePermissions {
    pub read: bool,
    pub write: bool,
    pub exec: bool,
}

impl FilePermissions {
    pub const fn default_rw() -> Self {
        Self { read: true, write: true, exec: false }
    }

    pub const fn read_only() -> Self {
        Self { read: true, write: false, exec: false }
    }

    pub fn to_string(&self) -> String {
        let r = if self.read { 'r' } else { '-' };
        let w = if self.write { 'w' } else { '-' };
        let x = if self.exec { 'x' } else { '-' };
        alloc::format!("-{}--{}--{}-", r, w, x)
    }
}

pub struct RamFile {
    pub name: String,
    pub data: Vec<u8>,
    pub permissions: FilePermissions,
    pub created_at: DateTime,
    pub modified_at: DateTime,
}

pub struct RamFs {
    files: Vec<RamFile>,
}

static mut RAMFS: Option<RamFs> = None;

impl RamFs {
    pub fn new() -> Self {
        let mut fs = Self {
            files: Vec::with_capacity(MAX_FILES),
        };
        fs.populate_initial_files();
        fs
    }

    fn populate_initial_files(&mut self) {
        let now = Rtc::read_datetime();

        self.create_file_internal(
            "readme.txt",
            b"=================================================================\n\
              Welcome to RustOS 1.0 - Modern 32-bit x86 Operating System\n\
              =================================================================\n\n\
              RustOS is an educational operating system developed in Rust for\n\
              Oracle VirtualBox with full LAN/Internet connectivity, a built-in\n\
              text web browser, and an interactive Deadlock Research Lab.\n\n\
              KEYBOARD NAVIGATION:\n\
                - Up/Down Arrows: Navigate desktop application menu\n\
                - Enter: Launch highlighted application\n\
                - Esc: Exit current application back to desktop\n\n\
              SYSTEM CAPABILITIES:\n\
                - Memory: 16 MB Kernel Heap with block-coalescing allocator\n\
                - Network: Intel 82540EM (e1000) Ethernet driver, DHCP & DNS\n\
                - Shell: 30+ Unix-compatible utilities (ls, cat, ping, httpget, etc.)\n\
                - File System: In-memory RAMFS with vi-style editor\n\
                - Research: Deadlock Detection & Banker's Algorithm Lab\n\n\
              NOTE: This filesystem is in-memory. Files are lost on VM shutdown.\n",
            FilePermissions::default_rw(),
            now,
        );

        self.create_file_internal(
            "notes.txt",
            b"=================================================================\n\
              RustOS Architectural Engineering Notes\n\
              =================================================================\n\n\
              1. Interrupt Subsystem:\n\
                 Dual 8259 PIC mapped to vectors 32-47. 8254 PIT running at 100 Hz\n\
                 (10ms ticks). PS/2 keyboard scancode set 1 interrupt-driven buffer.\n\n\
              2. Memory Management:\n\
                 Physical 16 MB heap mapped at 0x00400000. Free blocks are merged\n\
                 bidirectionally upon deallocation to eliminate external fragmentation\n\
                 and prevent double-free violations (OS Compendium Challenge K2).\n\n\
              3. Networking Engine:\n\
                 PCI Bus scanning detects Intel PRO/1000 MT (82540EM) in VirtualBox.\n\
                 Packet rings handle TX and RX. Full ARP cache, IPv4 checksums, UDP,\n\
                 ICMP echo reply, and DHCP negotiation for VirtualBox NAT.\n",
            FilePermissions::default_rw(),
            now,
        );

        self.create_file_internal(
            "todo.txt",
            b"[X] Multiboot-compatible kernel & linker script\n\
              [X] GDT, IDT, PIC, PIT, and RTC drivers\n\
              [X] 16 MB Coalescing Heap Allocator\n\
              [X] PS/2 Keyboard buffer with cursor control\n\
              [X] In-memory file system with 30+ shell commands\n\
              [X] Beautiful text-mode desktop interface\n\
              [X] Intel e1000 PCI network driver\n\
              [X] DHCP, DNS, ICMP ping, and TCP stack\n\
              [X] Built-in text-mode web browser with HTML renderer\n\
              [X] Deadlock Detection Research Lab & Banker's Algorithm\n\
              [ ] EXT2 persistent hard disk driver (Future Roadmap)\n",
            FilePermissions::default_rw(),
            now,
        );

        self.create_file_internal(
            "about.txt",
            b"RustOS Version: 1.0.0-release\n\
              Architecture:   i686 / 32-bit x86 Protected Mode\n\
              Compiler:       Rust nightly-2024-05-15\n\
              Target VM:      Oracle VirtualBox 7.x (Intel 82540EM NAT)\n\
              Build System:   ELF Multiboot with El Torito / GRUB ISO\n\
              Authors:        RustOS Core Architecture Team\n\
              License:        MIT / Educational Open Source\n",
            FilePermissions::read_only(),
            now,
        );
    }

    fn create_file_internal(&mut self, name: &str, content: &[u8], perms: FilePermissions, ts: DateTime) {
        if self.files.len() >= MAX_FILES { return; }
        let mut data = Vec::with_capacity(content.len().min(MAX_FILE_SIZE));
        data.extend_from_slice(&content[..content.len().min(MAX_FILE_SIZE)]);
        self.files.push(RamFile {
            name: String::from(name),
            data,
            permissions: perms,
            created_at: ts,
            modified_at: ts,
        });
    }

    pub fn list(&self) -> &[RamFile] {
        &self.files
    }

    pub fn get(&self, name: &str) -> Option<&RamFile> {
        self.files.iter().find(|f| f.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut RamFile> {
        self.files.iter_mut().find(|f| f.name == name)
    }

    pub fn create(&mut self, name: &str) -> Result<(), &'static str> {
        if self.files.len() >= MAX_FILES {
            return Err("File system full: maximum 32 files reached");
        }
        if self.get(name).is_some() {
            return Err("File already exists");
        }
        if name.len() > MAX_NAME_LEN || name.is_empty() {
            return Err("Invalid filename length (1..32 chars)");
        }
        let now = Rtc::read_datetime();
        self.files.push(RamFile {
            name: String::from(name),
            data: Vec::new(),
            permissions: FilePermissions::default_rw(),
            created_at: now,
            modified_at: now,
        });
        Ok(())
    }

    pub fn write_file(&mut self, name: &str, content: &[u8]) -> Result<(), &'static str> {
        let now = Rtc::read_datetime();
        if let Some(file) = self.get_mut(name) {
            if !file.permissions.write {
                return Err("Permission denied: file is read-only");
            }
            if content.len() > MAX_FILE_SIZE {
                return Err("File size exceeds 2048 byte limit");
            }
            file.data.clear();
            file.data.extend_from_slice(content);
            file.modified_at = now;
            Ok(())
        } else {
            // Auto create
            if self.files.len() >= MAX_FILES {
                return Err("File system full: maximum 32 files reached");
            }
            if content.len() > MAX_FILE_SIZE {
                return Err("File size exceeds 2048 byte limit");
            }
            let mut data = Vec::new();
            data.extend_from_slice(content);
            self.files.push(RamFile {
                name: String::from(name),
                data,
                permissions: FilePermissions::default_rw(),
                created_at: now,
                modified_at: now,
            });
            Ok(())
        }
    }

    pub fn remove(&mut self, name: &str) -> Result<(), &'static str> {
        if let Some(pos) = self.files.iter().position(|f| f.name == name) {
            self.files.remove(pos);
            Ok(())
        } else {
            Err("No such file or directory")
        }
    }

    pub fn rename(&mut self, old_name: &str, new_name: &str) -> Result<(), &'static str> {
        if self.get(new_name).is_some() {
            return Err("Destination file already exists");
        }
        if let Some(file) = self.get_mut(old_name) {
            file.name = String::from(new_name);
            file.modified_at = Rtc::read_datetime();
            Ok(())
        } else {
            Err("Source file not found")
        }
    }

    pub fn copy(&mut self, src_name: &str, dst_name: &str) -> Result<(), &'static str> {
        if self.files.len() >= MAX_FILES {
            return Err("File system full: maximum 32 files reached");
        }
        if self.get(dst_name).is_some() {
            return Err("Destination file already exists");
        }
        let src_file = self.get(src_name).ok_or("Source file not found")?;
        let new_file = RamFile {
            name: String::from(dst_name),
            data: src_file.data.clone(),
            permissions: src_file.permissions,
            created_at: Rtc::read_datetime(),
            modified_at: Rtc::read_datetime(),
        };
        self.files.push(new_file);
        Ok(())
    }

    pub fn chmod(&mut self, name: &str, read: bool, write: bool, exec: bool) -> Result<(), &'static str> {
        if let Some(file) = self.get_mut(name) {
            file.permissions = FilePermissions { read, write, exec };
            file.modified_at = Rtc::read_datetime();
            Ok(())
        } else {
            Err("File not found")
        }
    }
}

pub unsafe fn init_ramfs() {
    RAMFS = Some(RamFs::new());
}

pub fn get_fs() -> &'static mut RamFs {
    unsafe {
        RAMFS.as_mut().expect("RAMFS not initialized")
    }
}
