//! Simple vi-Style Full-Screen Text Editor for RustOS RAMFS

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::drivers::vga::{VGA_WRITER, Color, VGA_WIDTH};
use crate::drivers::keyboard::{pop_key, KeyEvent};
use crate::fs::ramfs::get_fs;

#[derive(PartialEq, Eq)]
enum EditorMode {
    Command,
    Insert,
    ExCommand,
}

pub fn launch_vi(filename: &str) {
    let fs = get_fs();
    let initial_content = if let Some(file) = fs.get(filename) {
        String::from_utf8_lossy(&file.data).into_owned()
    } else {
        String::new()
    };

    let mut lines: Vec<String> = if initial_content.is_empty() {
        alloc::vec![String::new()]
    } else {
        initial_content.lines().map(String::from).collect()
    };
    if lines.is_empty() {
        lines.push(String::new());
    }

    let mut cursor_x = 0usize;
    let mut cursor_y = 0usize;
    let scroll_y = 0usize;
    let mut mode = EditorMode::Command;
    let mut status_msg = format!("\"{}\" {} lines", filename, lines.len());
    let mut ex_cmd = String::new();

    loop {
        render_editor(filename, &lines, cursor_x, cursor_y, scroll_y, &mode, &status_msg, &ex_cmd);

        if let Some(key) = pop_key() {
            match mode {
                EditorMode::Command => match key {
                    KeyEvent::Char('i') => {
                        mode = EditorMode::Insert;
                        status_msg = String::from("-- INSERT --");
                    }
                    KeyEvent::Char(':') => {
                        mode = EditorMode::ExCommand;
                        ex_cmd.clear();
                    }
                    KeyEvent::Char('x') => {
                        // Delete char under cursor
                        if cursor_y < lines.len() && cursor_x < lines[cursor_y].len() {
                            lines[cursor_y].remove(cursor_x);
                        }
                    }
                    KeyEvent::Char('h') | KeyEvent::Left => {
                        if cursor_x > 0 { cursor_x -= 1; }
                    }
                    KeyEvent::Char('l') | KeyEvent::Right => {
                        if cursor_y < lines.len() && cursor_x < lines[cursor_y].len() {
                            cursor_x += 1;
                        }
                    }
                    KeyEvent::Char('k') | KeyEvent::Up => {
                        if cursor_y > 0 { cursor_y -= 1; }
                        cursor_x = cursor_x.min(lines[cursor_y].len());
                    }
                    KeyEvent::Char('j') | KeyEvent::Down => {
                        if cursor_y + 1 < lines.len() { cursor_y += 1; }
                        cursor_x = cursor_x.min(lines[cursor_y].len());
                    }
                    KeyEvent::Char('o') => {
                        lines.insert(cursor_y + 1, String::new());
                        cursor_y += 1;
                        cursor_x = 0;
                        mode = EditorMode::Insert;
                        status_msg = String::from("-- INSERT --");
                    }
                    _ => {}
                },
                EditorMode::Insert => match key {
                    KeyEvent::Esc => {
                        mode = EditorMode::Command;
                        status_msg = String::from("-- COMMAND --");
                        if cursor_x > 0 && cursor_x >= lines[cursor_y].len() {
                            cursor_x = lines[cursor_y].len().saturating_sub(1);
                        }
                    }
                    KeyEvent::Enter => {
                        let remainder = if cursor_x < lines[cursor_y].len() {
                            lines[cursor_y].split_off(cursor_x)
                        } else {
                            String::new()
                        };
                        lines.insert(cursor_y + 1, remainder);
                        cursor_y += 1;
                        cursor_x = 0;
                    }
                    KeyEvent::Backspace => {
                        if cursor_x > 0 {
                            lines[cursor_y].remove(cursor_x - 1);
                            cursor_x -= 1;
                        } else if cursor_y > 0 {
                            let curr = lines.remove(cursor_y);
                            cursor_y -= 1;
                            cursor_x = lines[cursor_y].len();
                            lines[cursor_y].push_str(&curr);
                        }
                    }
                    KeyEvent::Left => { if cursor_x > 0 { cursor_x -= 1; } }
                    KeyEvent::Right => { if cursor_x < lines[cursor_y].len() { cursor_x += 1; } }
                    KeyEvent::Up => { if cursor_y > 0 { cursor_y -= 1; } }
                    KeyEvent::Down => { if cursor_y + 1 < lines.len() { cursor_y += 1; } }
                    KeyEvent::Char(c) => {
                        if c >= ' ' && c <= '~' {
                            lines[cursor_y].insert(cursor_x, c);
                            cursor_x += 1;
                        }
                    }
                    _ => {}
                },
                EditorMode::ExCommand => match key {
                    KeyEvent::Esc => {
                        mode = EditorMode::Command;
                        status_msg = String::from("-- COMMAND --");
                    }
                    KeyEvent::Enter => {
                        let cmd = ex_cmd.trim();
                        if cmd == "q" {
                            break;
                        } else if cmd == "q!" {
                            break;
                        } else if cmd == "w" || cmd == "wq" {
                            let mut full_text = String::new();
                            for (i, line) in lines.iter().enumerate() {
                                full_text.push_str(line);
                                if i + 1 < lines.len() { full_text.push('\n'); }
                            }
                            let fs = get_fs();
                            let res = fs.write_file(filename, full_text.as_bytes());
                            if let Err(e) = res {
                                status_msg = format!("Error saving: {}", e);
                            } else {
                                status_msg = format!("\"{}\" written ({} bytes)", filename, full_text.len());
                            }
                            if cmd == "wq" {
                                break;
                            }
                            mode = EditorMode::Command;
                        } else {
                            status_msg = format!("Unknown ex command: {}", cmd);
                            mode = EditorMode::Command;
                        }
                    }
                    KeyEvent::Backspace => {
                        if !ex_cmd.is_empty() {
                            ex_cmd.pop();
                        } else {
                            mode = EditorMode::Command;
                        }
                    }
                    KeyEvent::Char(c) => {
                        ex_cmd.push(c);
                    }
                    _ => {}
                },
            }
        }
        core::hint::spin_loop();
    }
}

fn render_editor(
    _filename: &str,
    lines: &[String],
    cursor_x: usize,
    cursor_y: usize,
    scroll_y: usize,
    mode: &EditorMode,
    status_msg: &str,
    ex_cmd: &str
) {
    unsafe {
        VGA_WRITER.clear_screen();

        let view_lines = 23;
        for i in 0..view_lines {
            let line_idx = scroll_y + i;
            let screen_y = i;
            if line_idx < lines.len() {
                let line_str = &lines[line_idx];
                let display = if line_str.len() > 78 { &line_str[..78] } else { line_str };
                VGA_WRITER.print_at(0, screen_y, display, Color::LightGray, Color::Black);
            } else {
                VGA_WRITER.print_at(0, screen_y, "~", Color::DarkGray, Color::Black);
            }
        }

        // Status Line (Row 23)
        VGA_WRITER.fill_rect(0, 23, VGA_WIDTH, 1, b' ', Color::White, Color::DarkGray);
        if *mode == EditorMode::ExCommand {
            VGA_WRITER.print_at(0, 23, &format!(":{}", ex_cmd), Color::Yellow, Color::DarkGray);
        } else {
            VGA_WRITER.print_at(0, 23, status_msg, Color::Yellow, Color::DarkGray);
            let pos_str = format!("Ln {}, Col {}", cursor_y + 1, cursor_x + 1);
            VGA_WRITER.print_at(VGA_WIDTH - pos_str.len() - 1, 23, &pos_str, Color::White, Color::DarkGray);
        }

        // Position hardware cursor
        let cur_screen_y = cursor_y.saturating_sub(scroll_y).min(view_lines - 1);
        let cur_screen_x = cursor_x.min(VGA_WIDTH - 1);
        VGA_WRITER.set_cursor(cur_screen_x, cur_screen_y);
    }
}
