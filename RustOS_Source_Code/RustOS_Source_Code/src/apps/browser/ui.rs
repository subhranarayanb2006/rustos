//! Text-Mode Web Browser UI and Navigation Controller

use alloc::format;
use alloc::string::String;
use crate::drivers::vga::{VGA_WRITER, Color, VGA_WIDTH};
use crate::drivers::keyboard::{pop_key, KeyEvent};
use crate::net::execute_http_get;
use crate::apps::browser::html::{parse_html, RenderedDocument};

pub fn launch_browser(initial_url: Option<&str>) {
    let mut current_url = String::from(initial_url.unwrap_or("http://10.0.2.2/"));
    let mut scroll_offset = 0usize;
    let mut document = load_page(&current_url);

    loop {
        render_browser(&current_url, &document, scroll_offset);

        if let Some(key) = pop_key() {
            match key {
                KeyEvent::Esc => break,
                KeyEvent::Up => {
                    if scroll_offset > 0 { scroll_offset -= 1; }
                }
                KeyEvent::Down => {
                    if scroll_offset + 19 < document.lines.len() {
                        scroll_offset += 1;
                    }
                }
                KeyEvent::PageUp => {
                    scroll_offset = scroll_offset.saturating_sub(10);
                }
                KeyEvent::PageDown => {
                    if scroll_offset + 10 < document.lines.len() {
                        scroll_offset += 10;
                    }
                }
                KeyEvent::Char('r') | KeyEvent::Char('R') => {
                    document = load_page(&current_url);
                    scroll_offset = 0;
                }
                KeyEvent::Char('u') | KeyEvent::Char('U') | KeyEvent::Enter => {
                    // Edit URL
                    if let Some(new_url) = prompt_url_input(&current_url) {
                        current_url = new_url;
                        document = load_page(&current_url);
                        scroll_offset = 0;
                    }
                }
                KeyEvent::Char('l') | KeyEvent::Char('L') => {
                    // Follow link by number
                    if let Some(link_num) = prompt_link_number() {
                        if let Some(link) = document.links.iter().find(|l| l.index == link_num) {
                            current_url = resolve_relative_url(&current_url, &link.url);
                            document = load_page(&current_url);
                            scroll_offset = 0;
                        }
                    }
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

fn load_page(url: &str) -> RenderedDocument {
    show_status_indicator("Loading web resource...");

    match execute_http_get(url) {
        Ok(resp) => {
            let mut doc = parse_html(&resp.body);
            if doc.title.is_empty() || doc.title == "RustOS Web Document" {
                doc.title = format!("{} (HTTP {} {})", url, resp.status_code, resp.status_text);
            }
            doc
        }
        Err(err) => {
            let err_html = format!(
                "<html><head><title>Navigation Error</title></head><body>\n\
                <h1>Unable to Load Webpage</h1>\n\
                <p><b>Target URL:</b> {}</p>\n\
                <p><b>Diagnostic Error:</b> {}</p>\n\
                <hr>\n\
                <h2>Troubleshooting Guide:</h2>\n\
                <ul>\n\
                  <li>Verify that Oracle VirtualBox network adapter is attached to NAT or Bridged.</li>\n\
                  <li>Ensure the remote host server is reachable on port 80.</li>\n\
                  <li>HTTPS Note: TLS cryptographic certificates are not implemented in this text browser.</li>\n\
                </ul>\n\
                <p><a href=\"http://10.0.2.2/\">Return to Gateway Diagnostics</a></p>\n\
                </body></html>",
                url, err
            );
            parse_html(&err_html)
        }
    }
}

fn show_status_indicator(msg: &str) {
    unsafe {
        VGA_WRITER.fill_rect(20, 10, 40, 5, b' ', Color::Yellow, Color::Blue);
        VGA_WRITER.draw_box(20, 10, 40, 5, Color::White, Color::Blue, true);
        VGA_WRITER.print_at(22, 12, msg, Color::White, Color::Blue);
    }
}

fn render_browser(url: &str, doc: &RenderedDocument, scroll: usize) {
    unsafe {
        VGA_WRITER.clear_screen();

        // 1. Address Bar (Row 0)
        VGA_WRITER.fill_rect(0, 0, VGA_WIDTH, 1, b' ', Color::Black, Color::LightGray);
        VGA_WRITER.print_at(1, 0, "URL: ", Color::Black, Color::LightGray);
        let url_display = if url.len() > 68 { &url[..68] } else { url };
        VGA_WRITER.print_at(6, 0, url_display, Color::Blue, Color::LightGray);

        // 2. Title & Status Bar (Row 1)
        VGA_WRITER.fill_rect(0, 1, VGA_WIDTH, 1, b' ', Color::White, Color::DarkGray);
        let title_display = if doc.title.len() > 60 { &doc.title[..60] } else { &doc.title };
        VGA_WRITER.print_at(1, 1, title_display, Color::Yellow, Color::DarkGray);
        let link_count = format!("Links: {} | Line: {}", doc.links.len(), scroll + 1);
        VGA_WRITER.print_at(VGA_WIDTH - link_count.len() - 1, 1, &link_count, Color::LightCyan, Color::DarkGray);

        // 3. Viewport (Rows 2 to 22 = 21 lines)
        let viewport_height = 21;
        for i in 0..viewport_height {
            let line_idx = scroll + i;
            let screen_y = 2 + i;
            if line_idx < doc.lines.len() {
                let line = &doc.lines[line_idx];
                let text = if line.text.len() > 78 { &line.text[..78] } else { &line.text };
                VGA_WRITER.print_at(1, screen_y, text, line.fg, line.bg);
            }
        }

        // 4. Bottom Controls (Rows 23 & 24)
        VGA_WRITER.fill_rect(0, 23, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(1, 23, "[U/Enter] Edit URL  [L] Jump to Link  [R] Reload  [PgUp/PgDn] Scroll Fast", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(1, 24, "[Up/Down] Scroll Line  [Esc] Exit Browser", Color::LightCyan, Color::DarkGray);
    }
}

fn prompt_url_input(current: &str) -> Option<String> {
    unsafe {
        VGA_WRITER.fill_rect(5, 10, 70, 5, b' ', Color::White, Color::Blue);
        VGA_WRITER.draw_box(5, 10, 70, 5, Color::Yellow, Color::Blue, true);
        VGA_WRITER.print_at(7, 11, "Enter Web URL (http://...):", Color::White, Color::Blue);
    }

    let mut buf = String::from(current);
    loop {
        unsafe {
            VGA_WRITER.fill_rect(7, 13, 65, 1, b' ', Color::Black, Color::White);
            let display = if buf.len() > 63 { &buf[buf.len() - 63..] } else { &buf };
            VGA_WRITER.print_at(7, 13, display, Color::Black, Color::White);
        }

        if let Some(key) = pop_key() {
            match key {
                KeyEvent::Enter => return Some(buf),
                KeyEvent::Esc => return None,
                KeyEvent::Backspace => { buf.pop(); }
                KeyEvent::Char(c) => {
                    if buf.len() < 120 && c >= ' ' && c <= '~' {
                        buf.push(c);
                    }
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

fn prompt_link_number() -> Option<usize> {
    unsafe {
        VGA_WRITER.fill_rect(20, 10, 40, 5, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.draw_box(20, 10, 40, 5, Color::Yellow, Color::DarkGray, true);
        VGA_WRITER.print_at(22, 11, "Enter Link Number [1-99]:", Color::White, Color::DarkGray);
    }

    let mut buf = String::new();
    loop {
        unsafe {
            VGA_WRITER.fill_rect(25, 13, 30, 1, b' ', Color::Black, Color::White);
            VGA_WRITER.print_at(25, 13, &buf, Color::Black, Color::White);
        }

        if let Some(key) = pop_key() {
            match key {
                KeyEvent::Enter => {
                    return buf.parse::<usize>().ok();
                }
                KeyEvent::Esc => return None,
                KeyEvent::Backspace => { buf.pop(); }
                KeyEvent::Char(c) => {
                    if c.is_ascii_digit() && buf.len() < 3 {
                        buf.push(c);
                    }
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

fn resolve_relative_url(base: &str, target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        return String::from(target);
    }
    // Simple relative URL joiner
    if let Ok(parsed) = crate::net::http::ParsedUrl::parse(base) {
        if target.starts_with('/') {
            format!("http://{}:{}{}", parsed.host, parsed.port, target)
        } else {
            format!("http://{}:{}/{}", parsed.host, parsed.port, target)
        }
    } else {
        String::from(target)
    }
}
