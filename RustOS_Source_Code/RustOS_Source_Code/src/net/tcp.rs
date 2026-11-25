//! Transmission Control Protocol (TCP) Client (RFC 793)

use alloc::vec::Vec;
use crate::net::ipv4::calculate_checksum;

pub const TCP_FLAG_FIN: u8 = 0x01;
pub const TCP_FLAG_SYN: u8 = 0x02;
pub const TCP_FLAG_RST: u8 = 0x04;
pub const TCP_FLAG_PSH: u8 = 0x08;
pub const TCP_FLAG_ACK: u8 = 0x10;

#[derive(Debug, Clone)]
pub struct TcpSegment {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub flags: u8,
    pub window: u16,
    pub payload: Vec<u8>,
}

impl TcpSegment {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < 20 { return None; }
        let src_port = u16::from_be_bytes([raw[0], raw[1]]);
        let dst_port = u16::from_be_bytes([raw[2], raw[3]]);
        let seq_num = u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let ack_num = u32::from_be_bytes([raw[8], raw[9], raw[10], raw[11]]);
        let offset = ((raw[12] >> 4) * 4) as usize;
        let flags = raw[13];
        let window = u16::from_be_bytes([raw[14], raw[15]]);

        if raw.len() < offset { return None; }
        let payload = raw[offset..].to_vec();

        Some(Self {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            flags,
            window,
            payload,
        })
    }

    pub fn to_bytes(&self, src_ip: [u8; 4], dst_ip: [u8; 4]) -> Vec<u8> {
        let total_tcp_len = (20 + self.payload.len()) as u16;

        // Construct pseudo-header for checksum
        let mut pseudo = Vec::with_capacity(12 + 20 + self.payload.len());
        pseudo.extend_from_slice(&src_ip);
        pseudo.extend_from_slice(&dst_ip);
        pseudo.push(0);
        pseudo.push(6); // Protocol: TCP
        pseudo.extend_from_slice(&total_tcp_len.to_be_bytes());

        // TCP Header
        let mut header = [0u8; 20];
        header[0..2].copy_from_slice(&self.src_port.to_be_bytes());
        header[2..4].copy_from_slice(&self.dst_port.to_be_bytes());
        header[4..8].copy_from_slice(&self.seq_num.to_be_bytes());
        header[8..12].copy_from_slice(&self.ack_num.to_be_bytes());
        header[12] = 0x50; // Data offset: 5 (20 bytes)
        header[13] = self.flags;
        header[14..16].copy_from_slice(&self.window.to_be_bytes());
        header[16..18].copy_from_slice(&[0, 0]); // Checksum placeholder
        header[18..20].copy_from_slice(&[0, 0]); // Urgent pointer

        pseudo.extend_from_slice(&header);
        pseudo.extend_from_slice(&self.payload);

        let chk = calculate_checksum(&pseudo);
        header[16..18].copy_from_slice(&chk.to_be_bytes());

        let mut buf = Vec::with_capacity(20 + self.payload.len());
        buf.extend_from_slice(&header);
        buf.extend_from_slice(&self.payload);
        buf
    }
}
