//! IPv4 Protocol Implementation (RFC 791)

use alloc::vec::Vec;

pub const IP_PROTO_ICMP: u8 = 1;
pub const IP_PROTO_TCP:  u8 = 6;
pub const IP_PROTO_UDP:  u8 = 17;

pub fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        let word = u16::from_be_bytes([data[i], data[i + 1]]);
        sum = sum.wrapping_add(word as u32);
        i += 2;
    }
    if i < data.len() {
        let word = (data[i] as u32) << 8;
        sum = sum.wrapping_add(word);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

#[derive(Debug, Clone)]
pub struct Ipv4Packet {
    pub protocol: u8,
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
    pub payload: Vec<u8>,
}

impl Ipv4Packet {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < 20 { return None; }
        let ver_ihl = raw[0];
        let version = ver_ihl >> 4;
        let ihl = (ver_ihl & 0x0F) as usize * 4;
        if version != 4 || ihl < 20 || raw.len() < ihl { return None; }

        let total_len = u16::from_be_bytes([raw[2], raw[3]]) as usize;
        if raw.len() < total_len { return None; }

        let protocol = raw[9];
        let mut src_ip = [0u8; 4];
        let mut dst_ip = [0u8; 4];
        src_ip.copy_from_slice(&raw[12..16]);
        dst_ip.copy_from_slice(&raw[16..20]);

        let payload = raw[ihl..total_len].to_vec();

        Some(Self {
            protocol,
            src_ip,
            dst_ip,
            payload,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let total_len = (20 + self.payload.len()) as u16;
        let mut header = [0u8; 20];
        header[0] = 0x45; // Version 4, IHL 5 (20 bytes)
        header[1] = 0x00; // TOS
        header[2..4].copy_from_slice(&total_len.to_be_bytes());
        header[4..6].copy_from_slice(&0x1234u16.to_be_bytes()); // Identification
        header[6..8].copy_from_slice(&0x4000u16.to_be_bytes()); // Flags: Don't Fragment
        header[8] = 64;   // TTL
        header[9] = self.protocol;
        header[10..12].copy_from_slice(&[0, 0]); // Checksum placeholder
        header[12..16].copy_from_slice(&self.src_ip);
        header[16..20].copy_from_slice(&self.dst_ip);

        let checksum = calculate_checksum(&header);
        header[10..12].copy_from_slice(&checksum.to_be_bytes());

        let mut buf = Vec::with_capacity(20 + self.payload.len());
        buf.extend_from_slice(&header);
        buf.extend_from_slice(&self.payload);
        buf
    }
}
