//! Ethernet II Frame Framing and MAC Address Handling

use alloc::vec::Vec;

pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP:  u16 = 0x0806;

pub const MAC_BROADCAST: [u8; 6] = [0xFF; 6];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    pub const fn new(b: [u8; 6]) -> Self { Self(b) }
    pub const fn broadcast() -> Self { Self(MAC_BROADCAST) }
}

#[derive(Debug, Clone)]
pub struct EthernetFrame {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ether_type: u16,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < 14 { return None; }
        let mut dst_mac = [0u8; 6];
        let mut src_mac = [0u8; 6];
        dst_mac.copy_from_slice(&raw[0..6]);
        src_mac.copy_from_slice(&raw[6..12]);
        let ether_type = u16::from_be_bytes([raw[12], raw[13]]);
        let payload = raw[14..].to_vec();

        Some(Self {
            dst_mac,
            src_mac,
            ether_type,
            payload,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(14 + self.payload.len());
        buf.extend_from_slice(&self.dst_mac);
        buf.extend_from_slice(&self.src_mac);
        buf.extend_from_slice(&self.ether_type.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }
}
