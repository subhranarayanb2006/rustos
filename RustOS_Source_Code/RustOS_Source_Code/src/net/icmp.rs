//! Internet Control Message Protocol (ICMP) Implementation

use alloc::vec::Vec;
use crate::net::ipv4::calculate_checksum;

pub const ICMP_TYPE_ECHO_REPLY:   u8 = 0;
pub const ICMP_TYPE_ECHO_REQUEST: u8 = 8;

#[derive(Debug, Clone)]
pub struct IcmpPacket {
    pub icmp_type: u8,
    pub code: u8,
    pub identifier: u16,
    pub sequence: u16,
    pub data: Vec<u8>,
}

impl IcmpPacket {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < 8 { return None; }
        let icmp_type = raw[0];
        let code = raw[1];
        let identifier = u16::from_be_bytes([raw[4], raw[5]]);
        let sequence = u16::from_be_bytes([raw[6], raw[7]]);
        let data = raw[8..].to_vec();

        Some(Self {
            icmp_type,
            code,
            identifier,
            sequence,
            data,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.data.len());
        buf.push(self.icmp_type);
        buf.push(self.code);
        buf.extend_from_slice(&[0, 0]); // Checksum placeholder
        buf.extend_from_slice(&self.identifier.to_be_bytes());
        buf.extend_from_slice(&self.sequence.to_be_bytes());
        buf.extend_from_slice(&self.data);

        let chk = calculate_checksum(&buf);
        buf[2..4].copy_from_slice(&chk.to_be_bytes());
        buf
    }

    pub fn echo_request(id: u16, seq: u16, payload: &[u8]) -> Self {
        Self {
            icmp_type: ICMP_TYPE_ECHO_REQUEST,
            code: 0,
            identifier: id,
            sequence: seq,
            data: payload.to_vec(),
        }
    }

    pub fn echo_reply(id: u16, seq: u16, payload: &[u8]) -> Self {
        Self {
            icmp_type: ICMP_TYPE_ECHO_REPLY,
            code: 0,
            identifier: id,
            sequence: seq,
            data: payload.to_vec(),
        }
    }
}
