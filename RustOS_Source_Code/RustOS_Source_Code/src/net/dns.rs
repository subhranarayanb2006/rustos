//! Domain Name System (DNS) Client (RFC 1035)

use alloc::vec::Vec;

pub const DNS_PORT: u16 = 53;

pub struct DnsClient;

impl DnsClient {
    pub fn build_query(domain: &str, xid: u16) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        
        // Header (12 bytes)
        buf.extend_from_slice(&xid.to_be_bytes());
        buf.extend_from_slice(&0x0100u16.to_be_bytes()); // Flags: Standard query, Recursion Desired
        buf.extend_from_slice(&1u16.to_be_bytes());      // Questions: 1
        buf.extend_from_slice(&0u16.to_be_bytes());      // Answers: 0
        buf.extend_from_slice(&0u16.to_be_bytes());      // Authority: 0
        buf.extend_from_slice(&0u16.to_be_bytes());      // Additional: 0

        // Question: QNAME
        for label in domain.split('.') {
            if label.is_empty() { continue; }
            buf.push(label.len() as u8);
            buf.extend_from_slice(label.as_bytes());
        }
        buf.push(0); // Zero terminator for QNAME

        // QTYPE = 1 (A record)
        buf.extend_from_slice(&1u16.to_be_bytes());
        // QCLASS = 1 (IN - Internet)
        buf.extend_from_slice(&1u16.to_be_bytes());

        buf
    }

    pub fn parse_response(raw: &[u8]) -> Option<[u8; 4]> {
        if raw.len() < 12 { return None; }
        let flags = u16::from_be_bytes([raw[2], raw[3]]);
        if (flags & 0x8000) == 0 { return None; } // Not a response
        if (flags & 0x000F) != 0 { return None; } // Error RCODE

        let qdcount = u16::from_be_bytes([raw[4], raw[5]]) as usize;
        let ancount = u16::from_be_bytes([raw[6], raw[7]]) as usize;

        if ancount == 0 { return None; }

        // Skip Question section
        let mut idx = 12;
        for _ in 0..qdcount {
            while idx < raw.len() {
                let len = raw[idx] as usize;
                if len == 0 { idx += 1; break; }
                if (len & 0xC0) == 0xC0 { idx += 2; break; } // Pointer
                idx += 1 + len;
            }
            idx += 4; // QTYPE (2) + QCLASS (2)
        }

        // Parse Answer records
        for _ in 0..ancount {
            if idx >= raw.len() { break; }
            // Skip NAME
            if (raw[idx] & 0xC0) == 0xC0 {
                idx += 2; // Compressed pointer
            } else {
                while idx < raw.len() {
                    let len = raw[idx] as usize;
                    if len == 0 { idx += 1; break; }
                    idx += 1 + len;
                }
            }
            if idx + 10 > raw.len() { break; }
            let atype = u16::from_be_bytes([raw[idx], raw[idx + 1]]);
            let aclass = u16::from_be_bytes([raw[idx + 2], raw[idx + 3]]);
            let rdlen = u16::from_be_bytes([raw[idx + 8], raw[idx + 9]]) as usize;
            idx += 10;

            if atype == 1 && aclass == 1 && rdlen == 4 && idx + 4 <= raw.len() {
                // A Record (IPv4)
                let mut ip = [0u8; 4];
                ip.copy_from_slice(&raw[idx..idx + 4]);
                return Some(ip);
            }
            idx += rdlen;
        }

        None
    }
}
