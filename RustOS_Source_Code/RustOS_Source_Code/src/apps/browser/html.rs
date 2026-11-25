//! HTML Tokenizer and Text Layout Engine for RustOS Web Browser

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::drivers::vga::Color;

#[derive(Debug, Clone)]
pub struct FormattedLine {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
    pub link_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Hyperlink {
    pub index: usize,
    pub url: String,
    pub text: String,
}

pub struct RenderedDocument {
    pub title: String,
    pub lines: Vec<FormattedLine>,
    pub links: Vec<Hyperlink>,
}

pub fn parse_html(html: &str) -> RenderedDocument {
    let mut title = String::from("RustOS Web Document");
    let mut lines = Vec::new();
    let mut links = Vec::new();

    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut in_title = false;
    let mut current_tag = String::new();
    let mut text_buf = String::new();
    let mut title_buf = String::new();

    let mut current_href = String::new();
    let mut in_a = false;
    let mut in_heading = 0usize; // 1..6

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '<' {
            // Flush text buffer before tag
            if !text_buf.trim().is_empty() {
                let trimmed = text_buf.trim();
                let fg = if in_heading > 0 {
                    Color::Yellow
                } else if in_a {
                    Color::LightCyan
                } else {
                    Color::White
                };

                let link_idx = if in_a {
                    let idx = links.len() + 1;
                    links.push(Hyperlink {
                        index: idx,
                        url: current_href.clone(),
                        text: String::from(trimmed),
                    });
                    Some(idx)
                } else {
                    None
                };

                let line_str = if let Some(idx) = link_idx {
                    format!("[{}] {}", idx, trimmed)
                } else {
                    String::from(trimmed)
                };

                lines.push(FormattedLine {
                    text: line_str,
                    fg,
                    bg: Color::Black,
                    link_index: link_idx,
                });

                text_buf.clear();
            }

            in_tag = true;
            current_tag.clear();
            i += 1;
            continue;
        }

        if c == '>' {
            in_tag = false;
            let tag_lower = current_tag.to_lowercase();
            let tag_name = tag_lower.split_whitespace().next().unwrap_or("");

            match tag_name {
                "script" => in_script = true,
                "/script" => in_script = false,
                "style" => in_style = true,
                "/style" => in_style = false,
                "title" => in_title = true,
                "/title" => {
                    in_title = false;
                    if !title_buf.trim().is_empty() {
                        title = title_buf.trim().into();
                    }
                }
                "h1" => in_heading = 1,
                "/h1" => { in_heading = 0; lines.push(FormattedLine { text: String::new(), fg: Color::White, bg: Color::Black, link_index: None }); }
                "h2" => in_heading = 2,
                "/h2" => { in_heading = 0; lines.push(FormattedLine { text: String::new(), fg: Color::White, bg: Color::Black, link_index: None }); }
                "h3" => in_heading = 3,
                "/h3" => { in_heading = 0; }
                "p" | "/p" | "div" | "/div" | "br" => {
                    lines.push(FormattedLine { text: String::new(), fg: Color::White, bg: Color::Black, link_index: None });
                }
                "li" => {
                    text_buf.push_str(" * ");
                }
                "hr" => {
                    lines.push(FormattedLine {
                        text: String::from("----------------------------------------------------------------------------"),
                        fg: Color::DarkGray,
                        bg: Color::Black,
                        link_index: None,
                    });
                }
                _ => {
                    if tag_name.starts_with("a") {
                        in_a = true;
                        // Extract href="..."
                        if let Some(href_idx) = tag_lower.find("href=") {
                            let after = &current_tag[href_idx + 5..];
                            let quote = after.chars().next().unwrap_or('"');
                            if quote == '"' || quote == '\'' {
                                if let Some(end_quote) = after[1..].find(quote) {
                                    current_href = String::from(&after[1..1 + end_quote]);
                                }
                            }
                        }
                    } else if tag_name == "/a" {
                        in_a = false;
                        current_href.clear();
                    }
                }
            }

            i += 1;
            continue;
        }

        if in_tag {
            current_tag.push(c);
        } else if in_script || in_style {
            // Ignore script/style content
        } else if in_title {
            title_buf.push(c);
        } else {
            if c == '\n' || c == '\r' {
                text_buf.push(' ');
            } else {
                text_buf.push(c);
            }
        }

        i += 1;
    }

    // Flush any leftover text
    if !text_buf.trim().is_empty() {
        lines.push(FormattedLine {
            text: String::from(text_buf.trim()),
            fg: Color::White,
            bg: Color::Black,
            link_index: None,
        });
    }

    RenderedDocument { title, lines, links }
}
