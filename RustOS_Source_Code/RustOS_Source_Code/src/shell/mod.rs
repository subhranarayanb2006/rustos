//! Interactive Command-Line Shell Loop for RustOS

pub mod commands;
pub mod editor;

use alloc::string::String;
use alloc::vec::Vec;
use crate::drivers::vga::{VGA_WRITER, Color};
use crate::drivers::keyboard::{pop_key, KeyEvent};
use crate::shell::commands::execute_command;

pub fn launch_shell() {
    unsafe {
        VGA_WRITER.clear_screen();
        VGA_WRITER.set_color(Color::LightCyan, Color::Black);
        crate::println!("============================================================");
        crate::println!("           RustOS Command-Line Interactive Shell            ");
        crate::println!("Type 'help' for command list. Press [Esc] to exit to Desktop.");
        crate::println!("============================================================");
    }

    let mut history: Vec<String> = Vec::new();
    let mut history_idx: usize = 0;

    loop {
        unsafe {
            VGA_WRITER.set_color(Color::LightGreen, Color::Black);
            crate::print!("rustos:/$ ");
            VGA_WRITER.set_color(Color::White, Color::Black);
        }

        let mut line = String::new();
        let mut exit_shell = false;

        loop {
            if let Some(key) = pop_key() {
                match key {
                    KeyEvent::Enter => {
                        crate::println!();
                        break;
                    }
                    KeyEvent::Esc => {
                        exit_shell = true;
                        break;
                    }
                    KeyEvent::Backspace => {
                        if !line.is_empty() {
                            line.pop();
                            unsafe {
                                VGA_WRITER.write_byte(0x08);
                            }
                        }
                    }
                    KeyEvent::Up => {
                        if !history.is_empty() && history_idx > 0 {
                            history_idx -= 1;
                            // Erase current line
                            for _ in 0..line.len() {
                                unsafe { VGA_WRITER.write_byte(0x08); }
                            }
                            line = history[history_idx].clone();
                            crate::print!("{}", line);
                        }
                    }
                    KeyEvent::Down => {
                        if !history.is_empty() && history_idx + 1 < history.len() {
                            history_idx += 1;
                            for _ in 0..line.len() {
                                unsafe { VGA_WRITER.write_byte(0x08); }
                            }
                            line = history[history_idx].clone();
                            crate::print!("{}", line);
                        }
                    }
                    KeyEvent::Char(c) => {
                        if c >= ' ' && c <= '~' && line.len() < 128 {
                            line.push(c);
                            unsafe {
                                VGA_WRITER.write_byte(c as u8);
                            }
                        }
                    }
                    _ => {}
                }
            }
            core::hint::spin_loop();
        }

        if exit_shell {
            break;
        }

        let trimmed = line.trim();
        if !trimmed.is_empty() {
            if history.is_empty() || history.last().unwrap() != trimmed {
                history.push(String::from(trimmed));
            }
            history_idx = history.len();

            if trimmed == "exit" {
                break;
            }

            execute_command(trimmed);
        }
    }
}
