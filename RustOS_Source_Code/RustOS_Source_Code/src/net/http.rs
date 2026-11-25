//! Hypertext Transfer Protocol (HTTP/1.1) Client

use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub content_type: String,
    pub content_length: usize,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct ParsedUrl {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub is_https: bool,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, &'static str> {
        let trimmed = url.trim();
        let (is_https, rest) = if let Some(stripped) = trimmed.strip_prefix("http://") {
            (false, stripped)
        } else if let Some(stripped) = trimmed.strip_prefix("https://") {
            (true, stripped)
        } else {
            (false, trimmed)
        };

        let (host_port, path) = if let Some(slash_idx) = rest.find('/') {
            (&rest[..slash_idx], &rest[slash_idx..])
        } else {
            (rest, "/")
        };

        let (host, port) = if let Some(colon_idx) = host_port.find(':') {
            let h = &host_port[..colon_idx];
            let p_str = &host_port[colon_idx + 1..];
            let p = p_str.parse::<u16>().map_err(|_| "Invalid port number")?;
            (h, p)
        } else {
            let default_port = if is_https { 443 } else { 80 };
            (host_port, default_port)
        };

        if host.is_empty() {
            return Err("Empty hostname in URL");
        }

        Ok(Self {
            host: String::from(host),
            port,
            path: String::from(path),
            is_https,
        })
    }
}

pub struct HttpClient;

impl HttpClient {
    pub fn parse_response(raw: &[u8]) -> Option<HttpResponse> {
        let text = core::str::from_utf8(raw).unwrap_or("");
        let header_end = text.find("\r\n\r\n")?;
        let headers_part = &text[..header_end];
        let body_part = &text[header_end + 4..];

        let mut lines = headers_part.lines();
        let status_line = lines.next()?;
        let mut status_parts = status_line.splitn(3, ' ');
        let _proto = status_parts.next()?;
        let code_str = status_parts.next()?;
        let status_text = status_parts.next().unwrap_or("OK");
        let status_code = code_str.parse::<u16>().ok()?;

        let mut content_type = String::from("text/html");
        let mut content_length = body_part.len();

        for line in lines {
            if let Some(colon_idx) = line.find(':') {
                let name = line[..colon_idx].trim();
                let val = line[colon_idx + 1..].trim();
                if name.eq_ignore_ascii_case("content-type") {
                    content_type = val.to_string();
                } else if name.eq_ignore_ascii_case("content-length") {
                    if let Ok(len) = val.parse::<usize>() {
                        content_length = len;
                    }
                }
            }
        }

        Some(HttpResponse {
            status_code,
            status_text: status_text.to_string(),
            content_type,
            content_length,
            body: body_part.to_string(),
        })
    }
}
