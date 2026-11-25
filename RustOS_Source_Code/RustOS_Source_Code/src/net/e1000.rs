//! Intel 82540EM (e1000) Network Interface Card Driver for VirtualBox

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::net::pci::{PciDevice, pci_write_u32, pci_read_u32};
use crate::serial_println;

const NUM_RX_DESCRIPTORS: usize = 16;
const NUM_TX_DESCRIPTORS: usize = 16;
const BUFFER_SIZE: usize = 2048;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct RxDescriptor {
    pub address: u64,
    pub length: u16,
    pub checksum: u16,
    pub status: u8,
    pub errors: u8,
    pub special: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct TxDescriptor {
    pub address: u64,
    pub length: u16,
    pub cso: u8,
    pub cmd: u8,
    pub status: u8,
    pub css: u8,
    pub special: u16,
}

pub struct E1000Device {
    pub mmio_base: usize,
    pub mac_address: [u8; 6],
    pub rx_cur: AtomicUsize,
    pub tx_cur: AtomicUsize,
    pub link_up: bool,
    pub packets_rx: AtomicUsize,
    pub packets_tx: AtomicUsize,
}

// Statically allocated descriptor rings and packet buffers
static mut RX_DESCS: [RxDescriptor; NUM_RX_DESCRIPTORS] = [RxDescriptor {
    address: 0, length: 0, checksum: 0, status: 0, errors: 0, special: 0
}; NUM_RX_DESCRIPTORS];

static mut TX_DESCS: [TxDescriptor; NUM_TX_DESCRIPTORS] = [TxDescriptor {
    address: 0, length: 0, cso: 0, cmd: 0, status: 0, css: 0, special: 0
}; NUM_TX_DESCRIPTORS];

static mut RX_BUFFERS: [[u8; BUFFER_SIZE]; NUM_RX_DESCRIPTORS] = [[0; BUFFER_SIZE]; NUM_RX_DESCRIPTORS];
static mut TX_BUFFERS: [[u8; BUFFER_SIZE]; NUM_TX_DESCRIPTORS] = [[0; BUFFER_SIZE]; NUM_TX_DESCRIPTORS];

pub static mut E1000: Option<E1000Device> = None;

impl E1000Device {
    #[inline(always)]
    unsafe fn write_reg(&self, reg: usize, val: u32) {
        let ptr = (self.mmio_base + reg) as *mut u32;
        ptr.write_volatile(val);
    }

    #[inline(always)]
    unsafe fn read_reg(&self, reg: usize) -> u32 {
        let ptr = (self.mmio_base + reg) as *const u32;
        ptr.read_volatile()
    }

    pub unsafe fn init_from_pci(dev: &PciDevice) -> Option<Self> {
        serial_println!("[e1000] Initializing Intel 82540EM (PCI {}:{}.{})", dev.bus, dev.device, dev.function);

        // Enable Bus Mastering and Memory Space in PCI Command Register (offset 0x04)
        let pci_cmd = pci_read_u32(dev.bus, dev.device, dev.function, 0x04);
        pci_write_u32(dev.bus, dev.device, dev.function, 0x04, pci_cmd | 0x07);

        let mmio_base = (dev.bar0 & 0xFFFFFFF0) as usize;
        if mmio_base == 0 {
            serial_println!("[e1000] Error: BAR0 is 0");
            return None;
        }

        let mut instance = Self {
            mmio_base,
            mac_address: [0; 6],
            rx_cur: AtomicUsize::new(0),
            tx_cur: AtomicUsize::new(0),
            link_up: false,
            packets_rx: AtomicUsize::new(0),
            packets_tx: AtomicUsize::new(0),
        };

        // Reset device
        instance.write_reg(0x0000, instance.read_reg(0x0000) | 0x04000000); // Device Reset (CTRL.RST)
        crate::drivers::pit::sleep_ms(20);

        // Read MAC address from Receive Address Register (RAL0 / RAH0 at 0x5400 / 0x5404)
        let ral = instance.read_reg(0x5400);
        let rah = instance.read_reg(0x5404);

        if ral != 0 || (rah & 0xFFFF) != 0 {
            instance.mac_address[0] = (ral & 0xFF) as u8;
            instance.mac_address[1] = ((ral >> 8) & 0xFF) as u8;
            instance.mac_address[2] = ((ral >> 16) & 0xFF) as u8;
            instance.mac_address[3] = ((ral >> 24) & 0xFF) as u8;
            instance.mac_address[4] = (rah & 0xFF) as u8;
            instance.mac_address[5] = ((rah >> 8) & 0xFF) as u8;
        } else {
            // Default VirtualBox OUI fallback: 08:00:27:12:34:56
            instance.mac_address = [0x08, 0x00, 0x27, 0x12, 0x34, 0x56];
        }

        serial_println!(
            "[e1000] MAC Address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            instance.mac_address[0], instance.mac_address[1], instance.mac_address[2],
            instance.mac_address[3], instance.mac_address[4], instance.mac_address[5]
        );

        // Setup RX ring
        for i in 0..NUM_RX_DESCRIPTORS {
            RX_DESCS[i].address = RX_BUFFERS[i].as_ptr() as u64;
            RX_DESCS[i].status = 0;
        }
        let rx_descs_ptr = RX_DESCS.as_ptr() as u32;
        instance.write_reg(0x2800, rx_descs_ptr); // RDBAL
        instance.write_reg(0x2804, 0);            // RDBAH
        instance.write_reg(0x2808, (NUM_RX_DESCRIPTORS * core::mem::size_of::<RxDescriptor>()) as u32); // RDLEN
        instance.write_reg(0x2810, 0);            // RDH
        instance.write_reg(0x2818, (NUM_RX_DESCRIPTORS - 1) as u32); // RDT

        // Enable Receiver: RCTL = EN | BAM | MPE | BSIZE_2048
        instance.write_reg(0x0100, (1 << 1) | (1 << 15) | (1 << 4) | (0 << 16));

        // Setup TX ring
        for i in 0..NUM_TX_DESCRIPTORS {
            TX_DESCS[i].address = TX_BUFFERS[i].as_ptr() as u64;
            TX_DESCS[i].status = 1; // Mark as free
            TX_DESCS[i].cmd = 0;
        }
        let tx_descs_ptr = TX_DESCS.as_ptr() as u32;
        instance.write_reg(0x3800, tx_descs_ptr); // TDBAL
        instance.write_reg(0x3804, 0);            // TDBAH
        instance.write_reg(0x3808, (NUM_TX_DESCRIPTORS * core::mem::size_of::<TxDescriptor>()) as u32); // TDLEN
        instance.write_reg(0x3810, 0);            // TDH
        instance.write_reg(0x3818, 0);            // TDT

        // Enable Transmitter: TCTL = EN | PSP | COLD(0x200)
        instance.write_reg(0x0400, (1 << 1) | (1 << 3) | (0x200 << 12));

        // Check Link Status (STATUS.LU is bit 1)
        let status = instance.read_reg(0x0008);
        instance.link_up = (status & (1 << 1)) != 0;
        serial_println!("[e1000] Link Status: {}", if instance.link_up { "UP (Connected)" } else { "DOWN" });

        Some(instance)
    }

    pub fn send_packet(&self, data: &[u8]) -> bool {
        if data.len() > BUFFER_SIZE { return false; }

        unsafe {
            let cur = self.tx_cur.load(Ordering::SeqCst);
            let desc = &mut TX_DESCS[cur];

            // Wait until previous transmission done if needed
            let mut timeout = 100_000;
            while (desc.status & 1) == 0 && timeout > 0 {
                core::hint::spin_loop();
                timeout -= 1;
            }

            // Copy data to buffer
            let buf = &mut TX_BUFFERS[cur];
            buf[..data.len()].copy_from_slice(data);

            desc.length = data.len() as u16;
            desc.cmd = (1 << 0) | (1 << 3); // EOP (End of Packet) | RS (Report Status)
            desc.status = 0;

            let next = (cur + 1) % NUM_TX_DESCRIPTORS;
            self.tx_cur.store(next, Ordering::SeqCst);

            // Notify NIC of new packet by updating TDT
            self.write_reg(0x3818, next as u32);
            self.packets_tx.fetch_add(1, Ordering::Relaxed);
        }
        true
    }

    pub fn receive_packet(&self) -> Option<Vec<u8>> {
        unsafe {
            let cur = self.rx_cur.load(Ordering::SeqCst);
            let desc = &mut RX_DESCS[cur];

            if (desc.status & 1) != 0 { // DD (Descriptor Done) bit
                let len = desc.length as usize;
                let mut pkt = Vec::with_capacity(len);
                pkt.extend_from_slice(&RX_BUFFERS[cur][..len]);

                // Reset descriptor
                desc.status = 0;
                let next = (cur + 1) % NUM_RX_DESCRIPTORS;
                self.rx_cur.store(next, Ordering::SeqCst);

                // Advance RDT
                self.write_reg(0x2818, cur as u32);
                self.packets_rx.fetch_add(1, Ordering::Relaxed);

                Some(pkt)
            } else {
                None
            }
        }
    }
}
