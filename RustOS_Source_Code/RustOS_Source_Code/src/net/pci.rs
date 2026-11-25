//! PCI Bus Enumerator and Configuration Space Access

use crate::arch::x86::io::{inl, outl};

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16    = 0xCFC;

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_id: u8,
    pub subclass_id: u8,
    pub bar0: u32,
    pub irq_line: u8,
}

pub unsafe fn pci_read_u32(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000;

    outl(PCI_CONFIG_ADDRESS, address);
    inl(PCI_CONFIG_DATA)
}

pub unsafe fn pci_write_u32(bus: u8, device: u8, func: u8, offset: u8, val: u32) {
    let address = ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000;

    outl(PCI_CONFIG_ADDRESS, address);
    outl(PCI_CONFIG_DATA, val);
}

pub unsafe fn pci_read_u16(bus: u8, device: u8, func: u8, offset: u8) -> u16 {
    let val = pci_read_u32(bus, device, func, offset);
    ((val >> ((offset & 2) * 8)) & 0xFFFF) as u16
}

pub unsafe fn scan_pci() -> alloc::vec::Vec<PciDevice> {
    let mut devices = alloc::vec::Vec::new();

    for bus in 0..=31 { // scan first 32 buses for speed
        for device in 0..32 {
            let vendor = pci_read_u16(bus, device, 0, 0x00);
            if vendor == 0xFFFF { continue; } // Device does not exist

            let dev_id = pci_read_u16(bus, device, 0, 0x02);
            let class_info = pci_read_u32(bus, device, 0, 0x08);
            let class_id = ((class_info >> 24) & 0xFF) as u8;
            let subclass_id = ((class_info >> 16) & 0xFF) as u8;
            let bar0 = pci_read_u32(bus, device, 0, 0x10);
            let irq_info = pci_read_u32(bus, device, 0, 0x3C);
            let irq_line = (irq_info & 0xFF) as u8;

            devices.push(PciDevice {
                bus,
                device,
                function: 0,
                vendor_id: vendor,
                device_id: dev_id,
                class_id,
                subclass_id,
                bar0,
                irq_line,
            });
        }
    }

    devices
}
