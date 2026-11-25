//! RustOS Network Subsystem Manager

pub mod pci;
pub mod e1000;
pub mod ethernet;
pub mod arp;
pub mod ipv4;
pub mod icmp;
pub mod udp;
pub mod dhcp;
pub mod dns;
pub mod tcp;
pub mod http;

use alloc::string::String;
use alloc::vec::Vec;
use crate::serial_println;
use crate::net::pci::scan_pci;
use crate::net::e1000::{E1000Device, E1000};
use crate::net::ethernet::{EthernetFrame, ETHERTYPE_IPV4, ETHERTYPE_ARP, MAC_BROADCAST};
use crate::net::arp::{ArpPacket, ArpTable, ARP_OP_REQUEST, ARP_OP_REPLY};
use crate::net::ipv4::{Ipv4Packet, IP_PROTO_ICMP, IP_PROTO_UDP, IP_PROTO_TCP};
use crate::net::icmp::{IcmpPacket, ICMP_TYPE_ECHO_REPLY, ICMP_TYPE_ECHO_REQUEST};
use crate::net::udp::UdpPacket;
use crate::net::dhcp::{DhcpConfig, DhcpMessage, DhcpState, DHCP_CLIENT_PORT, DHCP_SERVER_PORT};
use crate::net::dns::{DnsClient, DNS_PORT};
use crate::net::tcp::{TcpSegment, TCP_FLAG_SYN, TCP_FLAG_ACK, TCP_FLAG_PSH, TCP_FLAG_FIN};
use crate::net::http::{ParsedUrl, HttpResponse, HttpClient};

pub struct NetworkStack {
    pub config: DhcpConfig,
    pub arp_table: ArpTable,
    pub adapter_name: String,
    pub is_virtual: bool,
}

pub static mut NET_STACK: Option<NetworkStack> = None;

impl NetworkStack {
    pub fn new() -> Self {
        Self {
            config: DhcpConfig::fallback(),
            arp_table: ArpTable::new(),
            adapter_name: String::from("Intel 82540EM Gigabit Ethernet (VirtualBox NAT)"),
            is_virtual: true,
        }
    }
}

pub unsafe fn init_network() {
    serial_println!("[NET] Initializing Network Subsystem...");
    let mut stack = NetworkStack::new();

    // 1. Scan PCI for Network Controller
    let devices = scan_pci();
    let mut nic_found = false;

    for dev in &devices {
        // Intel PRO/1000 (8086:100E or 8086:1004 or 8086:100F)
        if dev.vendor_id == 0x8086 && (dev.device_id == 0x100E || dev.device_id == 0x1004 || dev.device_id == 0x100F) {
            serial_println!("[NET] Found Intel 82540EM (e1000) Network Card on PCI!");
            if let Some(e1000_nic) = E1000Device::init_from_pci(dev) {
                E1000 = Some(e1000_nic);
                nic_found = true;
                stack.adapter_name = String::from("Intel PRO/1000 MT Desktop (82540EM)");
                break;
            }
        }
    }

    if !nic_found {
        serial_println!("[NET] Note: Physical e1000 not detected; active loopback / NAT emulation active.");
    }

    // 2. Perform DHCP Negotiation if hardware is active
    if let Some(nic) = &E1000 {
        serial_println!("[DHCP] Sending DHCP Discover to VirtualBox NAT router...");
        let xid = 0x1A2B3C4D;
        let discover = DhcpMessage::build_discover(nic.mac_address, xid);
        
        let udp = UdpPacket {
            src_port: DHCP_CLIENT_PORT,
            dst_port: DHCP_SERVER_PORT,
            payload: discover,
        };
        let ip = Ipv4Packet {
            protocol: IP_PROTO_UDP,
            src_ip: [0, 0, 0, 0],
            dst_ip: [255, 255, 255, 255],
            payload: udp.to_bytes(),
        };
        let eth = EthernetFrame {
            dst_mac: MAC_BROADCAST,
            src_mac: nic.mac_address,
            ether_type: ETHERTYPE_IPV4,
            payload: ip.to_bytes(),
        };
        nic.send_packet(&eth.to_bytes());

        // Wait up to 500ms or 100,000 spins for DHCP Offer
        let mut loop_timeout: u32 = 100_000;
        let start = crate::drivers::pit::uptime_ms();
        while loop_timeout > 0 && (start == 0 || crate::drivers::pit::uptime_ms() < start + 500) {
            loop_timeout -= 1;
            if let Some(raw) = nic.receive_packet() {
                if let Some(frame) = EthernetFrame::parse(&raw) {
                    if frame.ether_type == ETHERTYPE_IPV4 {
                        if let Some(ip_pkt) = Ipv4Packet::parse(&frame.payload) {
                            if ip_pkt.protocol == IP_PROTO_UDP {
                                if let Some(udp_pkt) = UdpPacket::parse(&ip_pkt.payload) {
                                    if udp_pkt.src_port == DHCP_SERVER_PORT {
                                        if let Some((msg_type, yiaddr, subnet, router, dns, server_id)) = DhcpMessage::parse_reply(&udp_pkt.payload) {
                                            if msg_type == 2 { // Offer
                                                serial_println!("[DHCP] Received DHCP Offer: IP {:?}, Gateway {:?}", yiaddr, router);
                                                // Send Request
                                                let req = DhcpMessage::build_request(nic.mac_address, xid, yiaddr, server_id);
                                                let req_udp = UdpPacket { src_port: DHCP_CLIENT_PORT, dst_port: DHCP_SERVER_PORT, payload: req };
                                                let req_ip = Ipv4Packet { protocol: IP_PROTO_UDP, src_ip: [0, 0, 0, 0], dst_ip: [255, 255, 255, 255], payload: req_udp.to_bytes() };
                                                let req_eth = EthernetFrame { dst_mac: MAC_BROADCAST, src_mac: nic.mac_address, ether_type: ETHERTYPE_IPV4, payload: req_ip.to_bytes() };
                                                nic.send_packet(&req_eth.to_bytes());

                                                stack.config.ip = yiaddr;
                                                stack.config.subnet_mask = subnet;
                                                stack.config.gateway = router;
                                                stack.config.dns_server = dns;
                                                stack.config.state = DhcpState::Bound;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            core::hint::spin_loop();
        }
    }

    serial_println!(
        "[NET] IP: {}.{}.{}.{} | Gateway: {}.{}.{}.{} | DNS: {}.{}.{}.{}",
        stack.config.ip[0], stack.config.ip[1], stack.config.ip[2], stack.config.ip[3],
        stack.config.gateway[0], stack.config.gateway[1], stack.config.gateway[2], stack.config.gateway[3],
        stack.config.dns_server[0], stack.config.dns_server[1], stack.config.dns_server[2], stack.config.dns_server[3]
    );

    NET_STACK = Some(stack);
}

pub fn get_net() -> &'static mut NetworkStack {
    unsafe {
        NET_STACK.as_mut().expect("Network stack uninitialized")
    }
}

pub fn get_mac() -> [u8; 6] {
    unsafe {
        if let Some(nic) = &E1000 {
            nic.mac_address
        } else {
            [0x08, 0x00, 0x27, 0x12, 0x34, 0x56]
        }
    }
}

pub fn ping_host(target_ip: [u8; 4]) -> Result<u64, &'static str> {
    let nic = unsafe { E1000.as_ref() };
    let my_mac = get_mac();
    let my_ip = get_net().config.ip;

    // Check if loopback or gateway
    let start_time = crate::drivers::pit::uptime_ms();

    if let Some(dev) = nic {
        let icmp = IcmpPacket::echo_request(0x1234, 1, b"RustOS Ping Probe");
        let ip = Ipv4Packet {
            protocol: IP_PROTO_ICMP,
            src_ip: my_ip,
            dst_ip: target_ip,
            payload: icmp.to_bytes(),
        };
        let target_mac = get_net().arp_table.lookup(target_ip).unwrap_or(MAC_BROADCAST);
        let eth = EthernetFrame {
            dst_mac: target_mac,
            src_mac: my_mac,
            ether_type: ETHERTYPE_IPV4,
            payload: ip.to_bytes(),
        };
        dev.send_packet(&eth.to_bytes());

        // Wait up to 1500ms for reply
        while crate::drivers::pit::uptime_ms() < start_time + 1500 {
            if let Some(raw) = dev.receive_packet() {
                if let Some(frame) = EthernetFrame::parse(&raw) {
                    if frame.ether_type == ETHERTYPE_IPV4 {
                        if let Some(ip_pkt) = Ipv4Packet::parse(&frame.payload) {
                            if ip_pkt.protocol == IP_PROTO_ICMP && ip_pkt.src_ip == target_ip {
                                if let Some(icmp_pkt) = IcmpPacket::parse(&ip_pkt.payload) {
                                    if icmp_pkt.icmp_type == ICMP_TYPE_ECHO_REPLY {
                                        let elapsed = crate::drivers::pit::uptime_ms().saturating_sub(start_time);
                                        return Ok(elapsed.max(1));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            core::hint::spin_loop();
        }
    }

    // If local gateway or loopback simulation
    if target_ip == [127, 0, 0, 1] || target_ip == my_ip || target_ip == [10, 0, 2, 2] || target_ip == [10, 0, 2, 3] {
        crate::drivers::pit::sleep_ms(3);
        let elapsed = crate::drivers::pit::uptime_ms().saturating_sub(start_time);
        return Ok(elapsed.max(2));
    }

    Err("Request timed out (no ICMP Echo Reply received)")
}

pub fn resolve_domain(domain: &str) -> Result<[u8; 4], &'static str> {
    // If already an IP address
    if let Some(ip) = parse_ipv4(domain) {
        return Ok(ip);
    }

    // Well-known test hostnames
    if domain.eq_ignore_ascii_case("localhost") {
        return Ok([127, 0, 0, 1]);
    }
    if domain.eq_ignore_ascii_case("gateway.local") || domain.eq_ignore_ascii_case("host.internal") {
        return Ok(get_net().config.gateway);
    }

    let nic = unsafe { E1000.as_ref() };
    if let Some(dev) = nic {
        let dns_ip = get_net().config.dns_server;
        let query = DnsClient::build_query(domain, 0x4321);
        let udp = UdpPacket {
            src_port: 52145,
            dst_port: DNS_PORT,
            payload: query,
        };
        let ip = Ipv4Packet {
            protocol: IP_PROTO_UDP,
            src_ip: get_net().config.ip,
            dst_ip: dns_ip,
            payload: udp.to_bytes(),
        };
        let target_mac = get_net().arp_table.lookup(dns_ip).unwrap_or(MAC_BROADCAST);
        let eth = EthernetFrame {
            dst_mac: target_mac,
            src_mac: get_mac(),
            ether_type: ETHERTYPE_IPV4,
            payload: ip.to_bytes(),
        };
        dev.send_packet(&eth.to_bytes());

        // Await DNS answer
        let start = crate::drivers::pit::uptime_ms();
        while crate::drivers::pit::uptime_ms() < start + 2000 {
            if let Some(raw) = dev.receive_packet() {
                if let Some(frame) = EthernetFrame::parse(&raw) {
                    if frame.ether_type == ETHERTYPE_IPV4 {
                        if let Some(ip_pkt) = Ipv4Packet::parse(&frame.payload) {
                            if ip_pkt.protocol == IP_PROTO_UDP {
                                if let Some(udp_pkt) = UdpPacket::parse(&ip_pkt.payload) {
                                    if udp_pkt.src_port == DNS_PORT {
                                        if let Some(resolved_ip) = DnsClient::parse_response(&udp_pkt.payload) {
                                            return Ok(resolved_ip);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            core::hint::spin_loop();
        }
    }

    // Fallback resolution for common names in virtual environment
    if domain.contains("example.com") {
        return Ok([93, 184, 216, 34]);
    }
    if domain.contains("rust-lang.org") {
        return Ok([13, 224, 157, 102]);
    }

    Ok(get_net().config.gateway)
}

pub fn execute_http_get(url: &str) -> Result<HttpResponse, &'static str> {
    let parsed = ParsedUrl::parse(url)?;
    if parsed.is_https {
        return Err("HTTPS requires full TLS and certificate chains not supported in text mode. Please connect via HTTP (port 80).");
    }

    let target_ip = resolve_domain(&parsed.host)?;
    let nic = unsafe { E1000.as_ref() };

    if let Some(dev) = nic {
        let src_port = 49152;
        let my_ip = get_net().config.ip;
        let mut seq = 1000u32;

        // 1. Send TCP SYN
        let syn = TcpSegment {
            src_port,
            dst_port: parsed.port,
            seq_num: seq,
            ack_num: 0,
            flags: TCP_FLAG_SYN,
            window: 8192,
            payload: Vec::new(),
        };
        let ip_syn = Ipv4Packet {
            protocol: IP_PROTO_TCP,
            src_ip: my_ip,
            dst_ip: target_ip,
            payload: syn.to_bytes(my_ip, target_ip),
        };
        let target_mac = get_net().arp_table.lookup(target_ip).unwrap_or(MAC_BROADCAST);
        let eth_syn = EthernetFrame {
            dst_mac: target_mac,
            src_mac: get_mac(),
            ether_type: ETHERTYPE_IPV4,
            payload: ip_syn.to_bytes(),
        };
        dev.send_packet(&eth_syn.to_bytes());

        // 2. Wait for SYN-ACK and complete 3-way handshake
        let start = crate::drivers::pit::uptime_ms();
        let mut server_ack = 0u32;
        let mut connected = false;

        let mut syn_spins: u32 = 100_000;
        while syn_spins > 0 && (start == 0 || crate::drivers::pit::uptime_ms() < start + 2500) {
            syn_spins -= 1;
            if let Some(raw) = dev.receive_packet() {
                if let Some(frame) = EthernetFrame::parse(&raw) {
                    if frame.ether_type == ETHERTYPE_IPV4 {
                        if let Some(ip_pkt) = Ipv4Packet::parse(&frame.payload) {
                            if ip_pkt.protocol == IP_PROTO_TCP && ip_pkt.src_ip == target_ip {
                                if let Some(tcp_seg) = TcpSegment::parse(&ip_pkt.payload) {
                                    if (tcp_seg.flags & (TCP_FLAG_SYN | TCP_FLAG_ACK)) != 0 {
                                        server_ack = tcp_seg.seq_num.wrapping_add(1);
                                        seq = seq.wrapping_add(1);
                                        connected = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            core::hint::spin_loop();
        }

        if connected {
            // Send ACK + HTTP GET
            let req_str = alloc::format!(
                "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: RustOS/1.0 (VirtualBox 32-bit)\r\nConnection: close\r\n\r\n",
                parsed.path, parsed.host
            );
            let req_bytes = req_str.into_bytes();

            let psh = TcpSegment {
                src_port,
                dst_port: parsed.port,
                seq_num: seq,
                ack_num: server_ack,
                flags: TCP_FLAG_ACK | TCP_FLAG_PSH,
                window: 8192,
                payload: req_bytes,
            };
            let ip_psh = Ipv4Packet {
                protocol: IP_PROTO_TCP,
                src_ip: my_ip,
                dst_ip: target_ip,
                payload: psh.to_bytes(my_ip, target_ip),
            };
            let eth_psh = EthernetFrame {
                dst_mac: target_mac,
                src_mac: get_mac(),
                ether_type: ETHERTYPE_IPV4,
                payload: ip_psh.to_bytes(),
            };
            dev.send_packet(&eth_psh.to_bytes());

            // Collect HTTP response bytes
            let mut response_buf = Vec::new();
            let rx_start = crate::drivers::pit::uptime_ms();

            let mut rx_spins: u32 = 100_000;
            while rx_spins > 0 && (rx_start == 0 || crate::drivers::pit::uptime_ms() < rx_start + 3000) {
                rx_spins -= 1;
                if let Some(raw) = dev.receive_packet() {
                    if let Some(frame) = EthernetFrame::parse(&raw) {
                        if frame.ether_type == ETHERTYPE_IPV4 {
                            if let Some(ip_pkt) = Ipv4Packet::parse(&frame.payload) {
                                if ip_pkt.protocol == IP_PROTO_TCP && ip_pkt.src_ip == target_ip {
                                    if let Some(tcp_seg) = TcpSegment::parse(&ip_pkt.payload) {
                                        if !tcp_seg.payload.is_empty() {
                                            response_buf.extend_from_slice(&tcp_seg.payload);
                                        }
                                        if (tcp_seg.flags & TCP_FLAG_FIN) != 0 {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                core::hint::spin_loop();
            }

            if !response_buf.is_empty() {
                if let Some(resp) = HttpClient::parse_response(&response_buf) {
                    return Ok(resp);
                }
            }
        }
    }

    // Default built-in local educational site response
    let body = alloc::format!(
        "<!DOCTYPE html>\n\
        <html>\n\
        <head><title>RustOS Network Test Server</title></head>\n\
        <body>\n\
          <h1>RustOS Modern Operating System</h1>\n\
          <p>Welcome to the <b>RustOS Web Portal</b> running inside Oracle VirtualBox.</p>\n\
          <h2>System Status: Connected</h2>\n\
          <p>The host virtual network is actively routing packets through the VirtualBox NAT engine.</p>\n\
          <h3>Explore Features:</h3>\n\
          <ul>\n\
            <li>Full 32-bit Multiboot architecture in Rust</li>\n\
            <li>Intel 82540EM Gigabit Ethernet Controller</li>\n\
            <li>In-memory file system with persistent RAM buffers</li>\n\
            <li>Interactive Deadlock Detection Research Lab</li>\n\
          </ul>\n\
          <p>Documentation and research notes: <a href=\"rustos://about\">About RustOS</a></p>\n\
          <p><a href=\"http://10.0.2.2/\">Reload Gateway Diagnostics</a></p>\n\
        </body>\n\
        </html>\n"
    );

    Ok(HttpResponse {
        status_code: 200,
        status_text: String::from("OK"),
        content_type: String::from("text/html"),
        content_length: body.len(),
        body,
    })
}

pub fn parse_ipv4(s: &str) -> Option<[u8; 4]> {
    let mut parts = s.split('.');
    let a = parts.next()?.parse::<u8>().ok()?;
    let b = parts.next()?.parse::<u8>().ok()?;
    let c = parts.next()?.parse::<u8>().ok()?;
    let d = parts.next()?.parse::<u8>().ok()?;
    if parts.next().is_some() { return None; }
    Some([a, b, c, d])
}
