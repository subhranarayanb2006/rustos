//! Address Resolution Protocol (ARP) Implementation

use alloc::vec::Vec;

pub const ARP_OP_REQUEST: u16 = 1;
pub const ARP_OP_REPLY:   u16 = 2;

#[derive(Debug, Clone)]
pub struct ArpPacket {
    pub opcode: u16,
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

impl ArpPacket {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < 28 { return None; }
        let htype = u16::from_be_bytes([raw[0], raw[1]]);
        let ptype = u16::from_be_bytes([raw[2], raw[3]]);
        let hlen = raw[4];
        let plen = raw[5];
        let opcode = u16::from_be_bytes([raw[6], raw[7]]);

        if htype != 1 || ptype != 0x0800 || hlen != 6 || plen != 4 {
            return None;
        }

        let mut sender_mac = [0u8; 6];
        let mut sender_ip = [0u8; 4];
        let mut target_mac = [0u8; 6];
        let mut target_ip = [0u8; 4];

        sender_mac.copy_from_slice(&raw[8..14]);
        sender_ip.copy_from_slice(&raw[14..18]);
        target_mac.copy_from_slice(&raw[18..24]);
        target_ip.copy_from_slice(&raw[24..28]);

        Some(Self {
            opcode,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(28);
        buf.extend_from_slice(&1u16.to_be_bytes());       // Hardware type: Ethernet
        buf.extend_from_slice(&0x0800u16.to_be_bytes());   // Protocol: IPv4
        buf.push(6);                                       // HW len: 6
        buf.push(4);                                       // Proto len: 4
        buf.extend_from_slice(&self.opcode.to_be_bytes()); // Opcode
        buf.extend_from_slice(&self.sender_mac);
        buf.extend_from_slice(&self.sender_ip);
        buf.extend_from_slice(&self.target_mac);
        buf.extend_from_slice(&self.target_ip);
        buf
    }
}

pub struct ArpEntry {
    pub ip: [u8; 4],
    pub mac: [u8; 6],
    pub updated_tick: u32,
}

pub struct ArpTable {
    entries: Vec<ArpEntry>,
}

impl ArpTable {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn insert(&mut self, ip: [u8; 4], mac: [u8; 6]) {
        let now = crate::drivers::pit::get_ticks();
        if let Some(entry) = self.entries.iter_mut().find(|e| e.ip == ip) {
            entry.mac = mac;
            entry.updated_tick = now;
        } else {
            if self.entries.len() >= 64 {
                self.entries.remove(0);
            }
            self.entries.push(ArpEntry { ip, mac, updated_tick: now });
        }
    }

    pub fn lookup(&self, ip: [u8; 4]) -> Option<[u8; 6]> {
        self.entries.iter().find(|e| e.ip == ip).map(|e| e.mac)
    }
}
