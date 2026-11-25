//! User Datagram Protocol (UDP) Implementation

use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct UdpPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < 8 { return None; }
        let src_port = u16::from_be_bytes([raw[0], raw[1]]);
        let dst_port = u16::from_be_bytes([raw[2], raw[3]]);
        let length = u16::from_be_bytes([raw[4], raw[5]]) as usize;
        if raw.len() < length || length < 8 { return None; }

        let payload = raw[8..length].to_vec();

        Some(Self {
            src_port,
            dst_port,
            payload,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let length = (8 + self.payload.len()) as u16;
        let mut buf = Vec::with_capacity(8 + self.payload.len());
        buf.extend_from_slice(&self.src_port.to_be_bytes());
        buf.extend_from_slice(&self.dst_port.to_be_bytes());
        buf.extend_from_slice(&length.to_be_bytes());
        buf.extend_from_slice(&[0, 0]); // Checksum optional in IPv4
        buf.extend_from_slice(&self.payload);
        buf
    }
}
