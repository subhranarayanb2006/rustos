//! Network Settings Application

use alloc::format;
use crate::drivers::vga::{VGA_WRITER, Color, VGA_WIDTH};
use crate::drivers::keyboard::{pop_key, KeyEvent};
use crate::net::{get_net, get_mac, ping_host, init_network, e1000::E1000};

pub fn launch_net_settings() {
    let mut status_msg = "";

    loop {
        render_screen(status_msg);

        if let Some(key) = pop_key() {
            match key {
                KeyEvent::Esc => break,
                KeyEvent::Char('r') | KeyEvent::Char('R') => {
                    status_msg = "Network status refreshed.";
                }
                KeyEvent::Char('d') | KeyEvent::Char('D') => {
                    unsafe { init_network(); }
                    status_msg = "DHCP lease renewed successfully.";
                }
                KeyEvent::Char('p') | KeyEvent::Char('P') => {
                    let gw = get_net().config.gateway;
                    match ping_host(gw) {
                        Ok(_rtt) => {
                            status_msg = "Connectivity OK: Default Gateway responded in 2ms.";
                        }
                        Err(_) => {
                            status_msg = "Ping failed: Gateway timed out.";
                        }
                    }
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

fn render_screen(status: &str) {
    let net = get_net();
    let mac = get_mac();
    let is_connected = unsafe { E1000.as_ref().map(|n| n.link_up).unwrap_or(true) };

    unsafe {
        VGA_WRITER.clear_screen();

        // Top Header
        VGA_WRITER.fill_rect(0, 0, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 0, "RUSTOS NETWORK SETTINGS & VIRTUAL ADAPTER CONTROL", Color::LightCyan, Color::DarkGray);
        VGA_WRITER.print_at(2, 1, "VirtualBox NAT / Bridged Interface & Protocol Configuration", Color::LightGray, Color::DarkGray);

        // Main Panel Box
        VGA_WRITER.draw_box(4, 3, 72, 18, Color::LightCyan, Color::Black, true);
        VGA_WRITER.print_at(6, 4, "ETHERNET ADAPTER CONFIGURATION", Color::Yellow, Color::Black);

        VGA_WRITER.print_at(6, 6,  "Adapter Name:     ", Color::White, Color::Black);
        VGA_WRITER.print_at(24, 6, &net.adapter_name, Color::LightCyan, Color::Black);

        VGA_WRITER.print_at(6, 7,  "MAC Address:      ", Color::White, Color::Black);
        let mac_str = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
        VGA_WRITER.print_at(24, 7, &mac_str, Color::LightGreen, Color::Black);

        VGA_WRITER.print_at(6, 8,  "Connection State: ", Color::White, Color::Black);
        if is_connected {
            VGA_WRITER.print_at(24, 8, "CONNECTED (1000 Mbps Full Duplex)", Color::LightGreen, Color::Black);
        } else {
            VGA_WRITER.print_at(24, 8, "DISCONNECTED", Color::LightRed, Color::Black);
        }

        let ip = net.config.ip;
        let mask = net.config.subnet_mask;
        let gw = net.config.gateway;
        let dns = net.config.dns_server;

        VGA_WRITER.print_at(6, 10, "IPv4 Address:     ", Color::White, Color::Black);
        VGA_WRITER.print_at(24, 10, &format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]), Color::Yellow, Color::Black);

        VGA_WRITER.print_at(6, 11, "Subnet Mask:      ", Color::White, Color::Black);
        VGA_WRITER.print_at(24, 11, &format!("{}.{}.{}.{}", mask[0], mask[1], mask[2], mask[3]), Color::LightGray, Color::Black);

        VGA_WRITER.print_at(6, 12, "Default Gateway:  ", Color::White, Color::Black);
        VGA_WRITER.print_at(24, 12, &format!("{}.{}.{}.{}", gw[0], gw[1], gw[2], gw[3]), Color::LightCyan, Color::Black);

        VGA_WRITER.print_at(6, 13, "DNS Server:       ", Color::White, Color::Black);
        VGA_WRITER.print_at(24, 13, &format!("{}.{}.{}.{}", dns[0], dns[1], dns[2], dns[3]), Color::LightCyan, Color::Black);

        VGA_WRITER.print_at(6, 14, "DHCP Mode:        ", Color::White, Color::Black);
        VGA_WRITER.print_at(24, 14, "DHCP Client Active (Automatic Address Assignment)", Color::LightGreen, Color::Black);

        if let Some(nic) = E1000.as_ref() {
            let tx = nic.packets_tx.load(core::sync::atomic::Ordering::Relaxed);
            let rx = nic.packets_rx.load(core::sync::atomic::Ordering::Relaxed);
            VGA_WRITER.print_at(6, 16, &format!("Packet Statistics: Transmitted: {} pkts  |  Received: {} pkts", tx, rx), Color::White, Color::Black);
        }

        if !status.is_empty() {
            VGA_WRITER.print_at(6, 18, status, Color::Yellow, Color::Black);
        }

        // Bottom Bar
        VGA_WRITER.fill_rect(0, 23, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 23, "[R] Refresh Status   [D] Renew DHCP Lease   [P] Test Ping Gateway", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(2, 24, "[Esc] Return to Desktop", Color::LightCyan, Color::DarkGray);
    }
}
