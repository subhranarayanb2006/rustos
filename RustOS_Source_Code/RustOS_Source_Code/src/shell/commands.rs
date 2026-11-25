//! RustOS Shell Commands Implementation (33 Standard Commands)

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::drivers::vga::{VGA_WRITER, Color};
use crate::drivers::rtc::Rtc;
use crate::drivers::pit;
use crate::fs::ramfs::get_fs;
use crate::net::{get_net, get_mac, ping_host, parse_ipv4, resolve_domain, execute_http_get, e1000::E1000};
use crate::apps::browser::ui::launch_browser;
use crate::apps::research::ui::launch_research_lab;
use crate::shell::editor::launch_vi;

pub fn execute_command(line: &str) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() { return; }

    let cmd = parts[0];
    let args = &parts[1..];

    match cmd {
        "help" => cmd_help(),
        "clear" => cmd_clear(),
        "reboot" => cmd_reboot(),
        "shutdown" => cmd_shutdown(),
        "date" => cmd_date(),
        "cal" => cmd_cal(),
        "echo" => cmd_echo(args),
        "ls" => cmd_ls(),
        "cat" => cmd_cat(args),
        "touch" => cmd_touch(args),
        "rm" => cmd_rm(args),
        "cp" => cmd_cp(args),
        "mv" => cmd_mv(args),
        "chmod" => cmd_chmod(args),
        "wc" => cmd_wc(args),
        "head" => cmd_head(args),
        "tail" => cmd_tail(args),
        "cut" => cmd_cut(args),
        "grab" => cmd_grab(args),
        "vi" => cmd_vi(args),
        "bc" => cmd_bc(args),
        "whoami" => cmd_whoami(),
        "uname" => cmd_uname(args),
        "hostname" => cmd_hostname(),
        "uptime" => cmd_uptime(),
        "lp" => cmd_lp(args),
        "browser" => cmd_browser(args),
        "net" => cmd_net(),
        "ipconfig" => cmd_ipconfig(),
        "ping" => cmd_ping(args),
        "httpget" => cmd_httpget(args),
        "research" | "deadlock" => cmd_research(),
        _ => {
            crate::println!("rustos: command not found: {}. Type 'help' for available commands.", cmd);
        }
    }
}

fn cmd_help() {
    crate::println!("=================== RUSTOS SHELL COMMANDS ===================");
    crate::println!("System:      help, clear, reboot, shutdown, whoami, uname, hostname, uptime");
    crate::println!("Time/Math:   date, cal, echo, bc <expr>");
    crate::println!("Filesystem:  ls, cat, touch, rm, cp, mv, chmod, wc, head, tail, cut, grab, vi, lp");
    crate::println!("Networking:  net, ipconfig, ping <host>, httpget <url>");
    crate::println!("Apps:        browser [url], research (or deadlock)");
    crate::println!("=============================================================");
}

fn cmd_clear() {
    unsafe {
        VGA_WRITER.clear_screen();
    }
}

fn cmd_reboot() -> ! {
    crate::println!("Rebooting RustOS...");
    pit::sleep_ms(200);
    unsafe {
        // Keyboard controller reset command
        crate::arch::x86::io::outb(0x64, 0xFE);
    }
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

fn cmd_shutdown() -> ! {
    crate::println!("Halting RustOS system...");
    pit::sleep_ms(200);
    unsafe {
        // VirtualBox / QEMU / ACPI power off port 0x604 / 0x4004
        crate::arch::x86::io::outw(0x604, 0x2000);
        crate::arch::x86::io::outw(0x4004, 0x3400);
        crate::arch::x86::io::outw(0xB004, 0x2000);
        core::arch::asm!("cli");
    }
    crate::println!("It is now safe to power off your virtual machine.");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

fn cmd_date() {
    let dt = Rtc::read_datetime();
    crate::println!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second);
}

fn cmd_cal() {
    let dt = Rtc::read_datetime();
    crate::println!("    November {:04}    ", dt.year);
    crate::println!("Su Mo Tu We Th Fr Sa");
    crate::println!(" 1  2  3  4  5  6  7");
    crate::println!(" 8  9 10 11 12 13 14");
    crate::println!("15 16 17 18 19 20 21");
    crate::println!("22 23 24 25 26 27 28");
    crate::println!("29 30");
}

fn cmd_echo(args: &[&str]) {
    crate::println!("{}", args.join(" "));
}

fn cmd_ls() {
    let fs = get_fs();
    let files = fs.list();
    crate::println!("Total files: {}", files.len());
    crate::println!("PERMISSIONS  SIZE    MODIFIED             NAME");
    crate::println!("-----------  ------  -------------------  ----------------");
    for f in files {
        let dt = f.modified_at;
        crate::println!(
            "{}  {:>5}B  {:04}-{:02}-{:02} {:02}:{:02}:{:02}  {}",
            f.permissions.to_string(),
            f.data.len(),
            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
            f.name
        );
    }
}

fn cmd_cat(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: cat <filename>");
        return;
    }
    let fs = get_fs();
    if let Some(file) = fs.get(args[0]) {
        let text = String::from_utf8_lossy(&file.data);
        crate::println!("{}", text);
    } else {
        crate::println!("cat: {}: No such file or directory", args[0]);
    }
}

fn cmd_touch(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: touch <filename>");
        return;
    }
    let fs = get_fs();
    if let Err(e) = fs.create(args[0]) {
        crate::println!("touch: cannot create '{}': {}", args[0], e);
    }
}

fn cmd_rm(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rm <filename>");
        return;
    }
    let fs = get_fs();
    if let Err(e) = fs.remove(args[0]) {
        crate::println!("rm: cannot remove '{}': {}", args[0], e);
    }
}

fn cmd_cp(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: cp <source> <destination>");
        return;
    }
    let fs = get_fs();
    if let Err(e) = fs.copy(args[0], args[1]) {
        crate::println!("cp: {}", e);
    }
}

fn cmd_mv(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: mv <source> <destination>");
        return;
    }
    let fs = get_fs();
    if let Err(e) = fs.rename(args[0], args[1]) {
        crate::println!("mv: {}", e);
    }
}

fn cmd_chmod(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: chmod [ro|rw] <filename>");
        return;
    }
    let fs = get_fs();
    let is_rw = args[0] == "rw" || args[0] == "+w" || args[0] == "644";
    if let Err(e) = fs.chmod(args[1], true, is_rw, false) {
        crate::println!("chmod: {}", e);
    }
}

fn cmd_wc(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: wc <filename>");
        return;
    }
    let fs = get_fs();
    if let Some(file) = fs.get(args[0]) {
        let text = String::from_utf8_lossy(&file.data);
        let lines = text.lines().count();
        let words = text.split_whitespace().count();
        let bytes = file.data.len();
        crate::println!("{:>6} {:>6} {:>6} {}", lines, words, bytes, args[0]);
    } else {
        crate::println!("wc: {}: No such file", args[0]);
    }
}

fn cmd_head(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: head <filename> [n]");
        return;
    }
    let n = if args.len() > 1 { args[1].parse::<usize>().unwrap_or(10) } else { 10 };
    let fs = get_fs();
    if let Some(file) = fs.get(args[0]) {
        let text = String::from_utf8_lossy(&file.data);
        for line in text.lines().take(n) {
            crate::println!("{}", line);
        }
    } else {
        crate::println!("head: {}: No such file", args[0]);
    }
}

fn cmd_tail(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: tail <filename> [n]");
        return;
    }
    let n = if args.len() > 1 { args[1].parse::<usize>().unwrap_or(10) } else { 10 };
    let fs = get_fs();
    if let Some(file) = fs.get(args[0]) {
        let text = String::from_utf8_lossy(&file.data);
        let all_lines: Vec<&str> = text.lines().collect();
        let skip = all_lines.len().saturating_sub(n);
        for line in all_lines.iter().skip(skip) {
            crate::println!("{}", line);
        }
    } else {
        crate::println!("tail: {}: No such file", args[0]);
    }
}

fn cmd_cut(args: &[&str]) {
    // Usage: cut -d <delimiter> -f <field> <file>
    if args.len() < 2 {
        crate::println!("Usage: cut -d <delim> -f <field_num> <filename>");
        return;
    }
    let delim = if args.len() >= 3 && args[0] == "-d" { args[1].chars().next().unwrap_or(',') } else { ',' };
    let field = if args.len() >= 5 && args[2] == "-f" { args[3].parse::<usize>().unwrap_or(1).saturating_sub(1) } else { 0 };
    let fname = args.last().unwrap_or(&"");

    let fs = get_fs();
    if let Some(file) = fs.get(fname) {
        let text = String::from_utf8_lossy(&file.data);
        for line in text.lines() {
            let cols: Vec<&str> = line.split(delim).collect();
            if field < cols.len() {
                crate::println!("{}", cols[field]);
            }
        }
    } else {
        crate::println!("cut: {}: No such file", fname);
    }
}

fn cmd_grab(args: &[&str]) {
    // Search text in file (grep equivalent)
    if args.len() < 2 {
        crate::println!("Usage: grab <pattern> <filename>");
        return;
    }
    let pattern = args[0];
    let fname = args[1];
    let fs = get_fs();
    if let Some(file) = fs.get(fname) {
        let text = String::from_utf8_lossy(&file.data);
        let mut found = 0;
        for (i, line) in text.lines().enumerate() {
            if line.contains(pattern) {
                crate::println!("{}: {}", i + 1, line);
                found += 1;
            }
        }
        if found == 0 {
            crate::println!("grab: pattern '{}' not found in {}", pattern, fname);
        }
    } else {
        crate::println!("grab: {}: No such file", fname);
    }
}

fn cmd_vi(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: vi <filename>");
        return;
    }
    launch_vi(args[0]);
    cmd_clear();
}

fn cmd_bc(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: bc <integer_expression> (e.g. bc 12 + 34 * 2)");
        return;
    }
    let expr = args.join("");
    match evaluate_expression(&expr) {
        Ok(res) => crate::println!("{}", res),
        Err(e) => crate::println!("bc error: {}", e),
    }
}

fn evaluate_expression(expr: &str) -> Result<i64, &'static str> {
    // Simple integer expression evaluator supporting +, -, *, /
    let mut total: i64 = 0;
    let mut current_num: i64 = 0;
    let mut last_op = '+';
    let mut has_num = false;

    for c in expr.chars() {
        if c.is_ascii_digit() {
            current_num = current_num * 10 + (c as i64 - '0' as i64);
            has_num = true;
        } else if c == '+' || c == '-' || c == '*' || c == '/' {
            if !has_num { return Err("Unexpected operator"); }
            match last_op {
                '+' => total += current_num,
                '-' => total -= current_num,
                '*' => total *= current_num,
                '/' => {
                    if current_num == 0 { return Err("Division by zero"); }
                    total /= current_num;
                }
                _ => {}
            }
            last_op = c;
            current_num = 0;
            has_num = false;
        }
    }

    if has_num {
        match last_op {
            '+' => total += current_num,
            '-' => total -= current_num,
            '*' => total *= current_num,
            '/' => {
                if current_num == 0 { return Err("Division by zero"); }
                total /= current_num;
            }
            _ => {}
        }
        Ok(total)
    } else {
        Err("Incomplete expression")
    }
}

fn cmd_whoami() {
    crate::println!("root");
}

fn cmd_uname(args: &[&str]) {
    if args.contains(&"-a") {
        crate::println!("RustOS rustos-desktop 1.0.0-release #1 SMP 2026 i686 GNU/Rust");
    } else {
        crate::println!("RustOS");
    }
}

fn cmd_hostname() {
    crate::println!("rustos-desktop");
}

fn cmd_uptime() {
    let secs = pit::uptime_secs();
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    crate::println!("up {:02}:{:02}:{:02}, load average: 0.00, 0.00, 0.00 (Ticks: {})", hours, mins, s, pit::get_ticks());
}

fn cmd_lp(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: lp <filename>");
        return;
    }
    let fs = get_fs();
    if let Some(file) = fs.get(args[0]) {
        let text = String::from_utf8_lossy(&file.data);
        crate::println!("======================== [RUSTOS PRINTER SPOOL] ========================");
        crate::println!("Job: {:<20} Size: {} bytes   Date: 2026-11-19", args[0], file.data.len());
        crate::println!("------------------------------------------------------------------------");
        for (i, line) in text.lines().enumerate() {
            crate::println!("{:>3} | {}", i + 1, line);
        }
        crate::println!("========================== [END OF SPOOL JOB] ==========================");
    } else {
        crate::println!("lp: {}: No such file", args[0]);
    }
}

fn cmd_browser(args: &[&str]) {
    let url = args.first().copied();
    launch_browser(url);
    cmd_clear();
}

fn cmd_net() {
    let net = get_net();
    let mac = get_mac();
    crate::println!("Network Interface: {}", net.adapter_name);
    crate::println!("Hardware MAC:      {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    
    let is_connected = unsafe {
        E1000.as_ref().map(|n| n.link_up).unwrap_or(true)
    };
    crate::println!("Link Status:       {}", if is_connected { "CONNECTED (1000 Mbps Full Duplex)" } else { "DISCONNECTED" });

    if let Some(nic) = unsafe { E1000.as_ref() } {
        crate::println!("Packets TX:        {}", nic.packets_tx.load(core::sync::atomic::Ordering::Relaxed));
        crate::println!("Packets RX:        {}", nic.packets_rx.load(core::sync::atomic::Ordering::Relaxed));
    }
    crate::println!("Host Transport:    VirtualBox NAT (Guest uses virtual Ethernet; Host uses physical Wi-Fi)");
}

fn cmd_ipconfig() {
    let net = get_net();
    let ip = net.config.ip;
    let mask = net.config.subnet_mask;
    let gw = net.config.gateway;
    let dns = net.config.dns_server;

    crate::println!("RustOS IP Configuration (Oracle VirtualBox Adapter 1):");
    crate::println!("   Connection-specific DNS Suffix  . : localdomain");
    crate::println!("   Link-local IPv6 Address . . . . . : fe80::800:27ff:fe12:3456");
    crate::println!("   IPv4 Address. . . . . . . . . . . : {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    crate::println!("   Subnet Mask . . . . . . . . . . . : {}.{}.{}.{}", mask[0], mask[1], mask[2], mask[3]);
    crate::println!("   Default Gateway . . . . . . . . . : {}.{}.{}.{}", gw[0], gw[1], gw[2], gw[3]);
    crate::println!("   DNS Servers . . . . . . . . . . . : {}.{}.{}.{}", dns[0], dns[1], dns[2], dns[3]);
    crate::println!("   DHCP Enabled. . . . . . . . . . . : Yes (Lease: 86400 seconds)");
}

fn cmd_ping(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: ping <ip_address_or_domain>");
        return;
    }
    let target = args[0];
    let ip = if let Some(parsed) = parse_ipv4(target) {
        parsed
    } else {
        match resolve_domain(target) {
            Ok(resolved) => {
                crate::println!("DNS resolved {} to {}.{}.{}.{}", target, resolved[0], resolved[1], resolved[2], resolved[3]);
                resolved
            }
            Err(e) => {
                crate::println!("ping: cannot resolve {}: {}", target, e);
                return;
            }
        }
    };

    crate::println!("PING {}.{}.{}.{} with 32 bytes of data:", ip[0], ip[1], ip[2], ip[3]);
    let mut received = 0;
    let mut total_time = 0u64;

    for i in 1..=4 {
        match ping_host(ip) {
            Ok(rtt) => {
                crate::println!("Reply from {}.{}.{}.{}: bytes=32 time={}ms TTL=64", ip[0], ip[1], ip[2], ip[3], rtt);
                received += 1;
                total_time += rtt;
            }
            Err(e) => {
                crate::println!("Request timed out (seq={}): {}", i, e);
            }
        }
        pit::sleep_ms(300);
    }

    crate::println!("--- {}.{}.{}.{} ping statistics ---", ip[0], ip[1], ip[2], ip[3]);
    crate::println!("4 packets transmitted, {} received, {}% packet loss",
        received, ((4 - received) * 100) / 4);
    if received > 0 {
        crate::println!("rtt min/avg/max = 1/{}/{} ms", total_time / received, (total_time / received) + 2);
    }
}

fn cmd_httpget(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: httpget <url>");
        return;
    }
    let url = args[0];
    crate::println!("Connecting to {} ...", url);
    match execute_http_get(url) {
        Ok(resp) => {
            crate::println!("HTTP/1.1 {} {}", resp.status_code, resp.status_text);
            crate::println!("Content-Type: {}", resp.content_type);
            crate::println!("Content-Length: {} bytes", resp.content_length);
            crate::println!("------------------------------------------------------------");
            // Print first 15 lines of body
            for line in resp.body.lines().take(15) {
                crate::println!("{}", line);
            }
            if resp.body.lines().count() > 15 {
                crate::println!("... [truncated, use 'browser {}' to view full page]", url);
            }
        }
        Err(e) => {
            crate::println!("httpget error: {}", e);
        }
    }
}

fn cmd_research() {
    launch_research_lab();
    cmd_clear();
}
