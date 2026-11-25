//! Split-Pane File Manager Application for RAMFS

use alloc::format;
use alloc::string::String;
use crate::drivers::vga::{VGA_WRITER, Color, VGA_WIDTH};
use crate::drivers::keyboard::{pop_key, KeyEvent};
use crate::fs::ramfs::get_fs;
use crate::shell::editor::launch_vi;

pub fn launch_file_manager() {
    let mut selected_idx = 0usize;

    loop {
        let (files_count, selected_name) = render_fm_screen(selected_idx);

        if let Some(key) = pop_key() {
            match key {
                KeyEvent::Esc => break,
                KeyEvent::Up => {
                    if selected_idx > 0 { selected_idx -= 1; }
                }
                KeyEvent::Down => {
                    if selected_idx + 1 < files_count { selected_idx += 1; }
                }
                KeyEvent::Enter | KeyEvent::Char('e') | KeyEvent::Char('E') => {
                    if let Some(name) = &selected_name {
                        launch_vi(name);
                    }
                }
                KeyEvent::Char('d') | KeyEvent::Char('D') => {
                    if let Some(name) = &selected_name {
                        let fs = get_fs();
                        let _ = fs.remove(name);
                        if selected_idx > 0 && selected_idx >= fs.list().len() {
                            selected_idx = fs.list().len().saturating_sub(1);
                        }
                    }
                }
                KeyEvent::Char('n') | KeyEvent::Char('N') => {
                    let fs = get_fs();
                    let new_name = format!("file_{}.txt", fs.list().len() + 1);
                    let _ = fs.create(&new_name);
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

fn render_fm_screen(selected_idx: usize) -> (usize, Option<String>) {
    let fs = get_fs();
    let files = fs.list();
    let count = files.len();
    let selected_file = if selected_idx < count { Some(files[selected_idx].name.clone()) } else { None };

    unsafe {
        VGA_WRITER.clear_screen();

        // Header
        VGA_WRITER.fill_rect(0, 0, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 0, "RUSTOS FILE MANAGER (RAMFS)", Color::LightCyan, Color::DarkGray);
        VGA_WRITER.print_at(2, 1, "In-Memory Filesystem Navigator & Document Viewer", Color::LightGray, Color::DarkGray);

        // Left Pane: File List (Width 36)
        VGA_WRITER.draw_box(1, 3, 36, 19, Color::LightCyan, Color::Black, true);
        VGA_WRITER.print_at(3, 4, "FILES (RAMFS: MAX 32)", Color::Yellow, Color::Black);
        VGA_WRITER.print_at(3, 5, "NAME             SIZE   PERM", Color::White, Color::Black);
        VGA_WRITER.print_at(3, 6, "---------------- ---- --------", Color::DarkGray, Color::Black);

        for (i, f) in files.iter().enumerate().take(14) {
            let y = 7 + i;
            let is_sel = i == selected_idx;
            let (fg, bg) = if is_sel {
                (Color::Black, Color::LightCyan)
            } else {
                (Color::White, Color::Black)
            };

            let name_short = if f.name.len() > 15 { &f.name[..15] } else { &f.name };
            let line = format!("{:<15} {:>4}B {}", name_short, f.data.len(), if f.permissions.write { "rw" } else { "ro" });
            
            VGA_WRITER.fill_rect(3, y, 32, 1, b' ', fg, bg);
            VGA_WRITER.print_at(3, y, &line, fg, bg);
        }

        // Right Pane: Content Preview (Width 40)
        VGA_WRITER.draw_box(38, 3, 41, 19, Color::LightBlue, Color::Black, true);
        VGA_WRITER.print_at(40, 4, "DOCUMENT PREVIEW", Color::Yellow, Color::Black);

        if selected_idx < count {
            let f = &files[selected_idx];
            let dt = f.modified_at;
            VGA_WRITER.print_at(40, 5, &format!("File: {} ({} bytes)", f.name, f.data.len()), Color::LightGreen, Color::Black);
            VGA_WRITER.print_at(40, 6, &format!("Date: {:04}-{:02}-{:02} {:02}:{:02}:{:02}", dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second), Color::DarkGray, Color::Black);
            VGA_WRITER.print_at(40, 7, "-------------------------------------", Color::DarkGray, Color::Black);

            let preview_text = String::from_utf8_lossy(&f.data);
            for (i, line) in preview_text.lines().take(13).enumerate() {
                let y = 8 + i;
                let text = if line.len() > 37 { &line[..37] } else { line };
                VGA_WRITER.print_at(40, y, text, Color::LightGray, Color::Black);
            }
        } else {
            VGA_WRITER.print_at(40, 8, "No file selected.", Color::DarkGray, Color::Black);
        }

        // Bottom Bar
        VGA_WRITER.fill_rect(0, 23, VGA_WIDTH, 2, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(2, 23, "[↑/↓] Select File   [Enter/E] Edit in vi   [N] New File   [D] Delete File", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(2, 24, "[Esc] Return to Desktop", Color::LightCyan, Color::DarkGray);
    }

    (count, selected_file)
}
