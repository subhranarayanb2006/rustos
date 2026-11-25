# RustOS Built-in Text-Mode Web Browser

## 1. Overview

RustOS features a built-in text-mode web browser designed for educational exploration of networking and web protocols on bare-metal x86 systems.

The browser can be launched from:
1. **Desktop Application Menu**: Select option 2 (`Web Browser`) and press `Enter`.
2. **Command Shell**: Type `browser` or `browser http://<url>`.

---

## 2. Browser Interface & Layout

The browser UI utilizes the full 80x25 VGA character grid divided into functional visual zones:

```
+-------------------------------------------------------------------------------+
| URL: [ http://10.0.2.2:8080/index.html                           ] [STATUS: OK]|
+-------------------------------------------------------------------------------+
| PAGE TITLE: Welcome to the RustOS Web Portal                                   |
+-------------------------------------------------------------------------------+
|                                                                               |
|  # RustOS Bare-Metal Web Experience                                           |
|                                                                               |
|  Welcome to the experimental text-mode web client! This browser renders       |
|  HTML documents received directly across the virtual Intel 82540EM NIC.       |
|                                                                               |
|  Core Capabilities:                                                           |
|    * Heading rendering (H1, H2, H3)                                           |
|    * Paragraph word wrapping and formatting                                   |
|    * Unordered and ordered lists                                              |
|    * Selectable hyperlinked navigation                                        |
|                                                                               |
|  Useful Links:                                                                |
|    [1] http://10.0.2.2:8080/docs.html                                         |
|    [2] http://10.0.2.2:8080/research.html                                     |
|                                                                               |
|                                                                               |
+-------------------------------------------------------------------------------+
| Controls: [Enter] Open URL  [Up/Down] Scroll  [R] Reload  [Esc] Exit Browser  |
+-------------------------------------------------------------------------------+
```

---

## 3. Features & Supported HTML Elements

### A. HTTP Protocol Support
- **Protocol**: HTTP/1.1 over TCP.
- **Port**: Default port 80 (or custom port specified in URL, e.g. `http://10.0.2.2:8080`).
- **Headers Sent**: `User-Agent: RustOS-Browser/1.0`, `Host: <host>`, `Connection: close`, `Accept: text/html`.
- **Response Parsing**: Parses `HTTP/1.1 <status_code> <reason>`, parses headers (`Content-Type`, `Content-Length`), and extracts the payload body.

### B. HTML Parsing & Tokenization
The browser implements a lightweight, fault-tolerant HTML tokenizer:
- `<h1>`, `<h2>`, `<h3>`: Formatted as bold, highlighted header banners.
- `<p>`: Paragraph blocks formatted with clean margins and line spacing.
- `<ul>`, `<ol>`, `<li>`: Bullet points (`* `) and indented list items.
- `<a href="...">`: Hyperlinks indexed with numerical tags `[1]`, `[2]` for keyboard navigation.
- `<b>`, `<strong>`, `<i>`, `<em>`: Formatted with distinct foreground colors.
- HTML Entity Decoding: `&amp;` -> `&`, `&lt;` -> `<`, `&gt;` -> `>`, `&quot;` -> `"`, `&nbsp;` -> space.

### C. HTTPS Limitation Notice
> [!NOTE]
> Modern HTTPS requires TLS 1.3 encryption, cryptographic ciphers, and X.509 certificate authority bundle validation. RustOS intentionally does not implement partial or broken TLS. Attempting to open `https://` URLs displays a clean diagnostic explaining the limitation and suggesting HTTP alternatives, maintaining cryptographic security principles.

---

## 4. Keyboard Navigation & Controls

| Key | Action |
|---|---|
| `Enter` | Activate address bar or follow the currently selected numbered hyperlink |
| `Up Arrow` | Scroll page up by one line |
| `Down Arrow` | Scroll page down by one line |
| `Page Up` | Scroll page up by 15 lines |
| `Page Down` | Scroll page down by 15 lines |
| `Tab` | Cycle through hyperlinked items on page |
| `R` or `r` | Reload the current URL |
| `U` or `u` | Focus the URL entry bar to type a new address |
| `Esc` | Exit the browser and return to the Desktop |

---

## 5. Hosting Local Test Sites on the Windows Host

To test the browser against real web pages:
1. On the Windows host, start a simple Python HTTP server in any directory:
   ```powershell
   python -m http.server 8080
   ```
2. In RustOS, launch the browser and navigate to:
   ```
   http://10.0.2.2:8080
   ```
   *(In VirtualBox NAT mode, `10.0.2.2` routes directly to `localhost` on the Windows host).*
