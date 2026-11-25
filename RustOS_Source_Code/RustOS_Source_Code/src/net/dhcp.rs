//! Dynamic Host Configuration Protocol (DHCP) Client (RFC 2131)

use alloc::vec::Vec;

pub const DHCP_CLIENT_PORT: u16 = 68;
pub const DHCP_SERVER_PORT: u16 = 67;
pub const DHCP_MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99]; // 0x63825363

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DhcpState {
    Unconfigured,
    DiscoverSent,
    OfferReceived,
    RequestSent,
    Bound,
}

#[derive(Debug, Clone)]
pub struct DhcpConfig {
    pub ip: [u8; 4],
    pub subnet_mask: [u8; 4],
    pub gateway: [u8; 4],
    pub dns_server: [u8; 4],
    pub server_ip: [u8; 4],
    pub lease_time_secs: u32,
    pub state: DhcpState,
}

impl DhcpConfig {
    pub const fn fallback() -> Self {
        Self {
            ip: [10, 0, 2, 15],              // Standard VirtualBox NAT Guest IP
            subnet_mask: [255, 255, 255, 0], // Standard Subnet Mask
            gateway: [10, 0, 2, 2],          // VirtualBox NAT Gateway
            dns_server: [10, 0, 2, 3],       // VirtualBox NAT DNS Server
            server_ip: [10, 0, 2, 2],
            lease_time_secs: 86400,
            state: DhcpState::Bound,
        }
    }
}

pub struct DhcpMessage {
    pub op: u8,
    pub xid: u32,
    pub yiaddr: [u8; 4],
    pub chaddr: [u8; 6],
    pub options: Vec<u8>,
}

impl DhcpMessage {
    pub fn build_discover(mac: [u8; 6], xid: u32) -> Vec<u8> {
        let mut buf = Vec::with_capacity(300);
        buf.push(1); // op: BOOTREQUEST
        buf.push(1); // htype: Ethernet
        buf.push(6); // hlen: 6
        buf.push(0); // hops: 0
        buf.extend_from_slice(&xid.to_be_bytes()); // xid
        buf.extend_from_slice(&[0, 0]);             // secs
        buf.extend_from_slice(&[0x80, 0x00]);       // flags: broadcast
        buf.extend_from_slice(&[0; 4]);             // ciaddr
        buf.extend_from_slice(&[0; 4]);             // yiaddr
        buf.extend_from_slice(&[0; 4]);             // siaddr
        buf.extend_from_slice(&[0; 4]);             // giaddr
        buf.extend_from_slice(&mac);                // chaddr
        buf.extend_from_slice(&[0; 10]);            // chaddr padding up to 16
        buf.extend_from_slice(&[0; 64]);            // sname
        buf.extend_from_slice(&[0; 128]);           // file
        buf.extend_from_slice(&DHCP_MAGIC_COOKIE);  // Magic cookie

        // Options:
        // Opt 53: DHCP Message Type (1 = Discover)
        buf.extend_from_slice(&[53, 1, 1]);
        // Opt 55: Parameter Request List (1=Subnet, 3=Router, 6=DNS)
        buf.extend_from_slice(&[55, 3, 1, 3, 6]);
        // Opt 255: End
        buf.push(255);

        // Pad to min 300 bytes
        while buf.len() < 300 {
            buf.push(0);
        }
        buf
    }

    pub fn build_request(mac: [u8; 6], xid: u32, requested_ip: [u8; 4], server_ip: [u8; 4]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(300);
        buf.push(1); // BOOTREQUEST
        buf.push(1);
        buf.push(6);
        buf.push(0);
        buf.extend_from_slice(&xid.to_be_bytes());
        buf.extend_from_slice(&[0, 0]);
        buf.extend_from_slice(&[0x80, 0x00]); // broadcast
        buf.extend_from_slice(&[0; 4]);
        buf.extend_from_slice(&[0; 4]);
        buf.extend_from_slice(&[0; 4]);
        buf.extend_from_slice(&[0; 4]);
        buf.extend_from_slice(&mac);
        buf.extend_from_slice(&[0; 10]);
        buf.extend_from_slice(&[0; 64]);
        buf.extend_from_slice(&[0; 128]);
        buf.extend_from_slice(&DHCP_MAGIC_COOKIE);

        // Opt 53: DHCP Request (3)
        buf.extend_from_slice(&[53, 1, 3]);
        // Opt 50: Requested IP
        buf.extend_from_slice(&[50, 4]);
        buf.extend_from_slice(&requested_ip);
        // Opt 54: Server Identifier
        buf.extend_from_slice(&[54, 4]);
        buf.extend_from_slice(&server_ip);
        // Opt 55: Parameter Request List
        buf.extend_from_slice(&[55, 3, 1, 3, 6]);
        // End
        buf.push(255);

        while buf.len() < 300 {
            buf.push(0);
        }
        buf
    }

    pub fn parse_reply(raw: &[u8]) -> Option<(u8, [u8; 4], [u8; 4], [u8; 4], [u8; 4], [u8; 4])> {
        if raw.len() < 240 { return None; }
        if raw[0] != 2 { return None; } // BOOTREPLY
        if &raw[236..240] != &DHCP_MAGIC_COOKIE { return None; }

        let mut yiaddr = [0u8; 4];
        yiaddr.copy_from_slice(&raw[16..20]);

        let mut msg_type = 0u8;
        let mut subnet = [255, 255, 255, 0];
        let mut router = [10, 0, 2, 2];
        let mut dns = [10, 0, 2, 3];
        let mut server_id = [10, 0, 2, 2];

        let mut idx = 240;
        while idx < raw.len() {
            let opt = raw[idx];
            if opt == 255 { break; } // End
            if opt == 0 { idx += 1; continue; } // Pad
            if idx + 1 >= raw.len() { break; }
            let len = raw[idx + 1] as usize;
            let val_idx = idx + 2;
            if val_idx + len > raw.len() { break; }

            match opt {
                53 => { // Message Type
                    if len >= 1 { msg_type = raw[val_idx]; }
                }
                1 => { // Subnet Mask
                    if len >= 4 { subnet.copy_from_slice(&raw[val_idx..val_idx+4]); }
                }
                3 => { // Router
                    if len >= 4 { router.copy_from_slice(&raw[val_idx..val_idx+4]); }
                }
                6 => { // DNS
                    if len >= 4 { dns.copy_from_slice(&raw[val_idx..val_idx+4]); }
                }
                54 => { // Server Identifier
                    if len >= 4 { server_id.copy_from_slice(&raw[val_idx..val_idx+4]); }
                }
                _ => {}
            }
            idx = val_idx + len;
        }

        Some((msg_type, yiaddr, subnet, router, dns, server_id))
    }
}
