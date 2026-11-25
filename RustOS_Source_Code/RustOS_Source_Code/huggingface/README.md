---
title: RustOS - 32-bit x86 Operating System
emoji: 🖥️
colorFrom: blue
colorTo: indigo
sdk: static
pinned: false
---

# RustOS: Live in the Browser

RustOS is an educational 32-bit x86 operating system implemented in Rust (`#![no_std]`).

This Hugging Face Space runs **RustOS live in your browser** using client-side x86 WebAssembly emulation ([v86](https://github.com/copy/v86)).

### Features
* **Windows-Style Graphical Desktop**: Start Menu, Explorer window, taskbar with digital clock, and 4 wallpapers.
* **Unix-Compatible Shell**: 33 commands (`ls`, `cat`, `touch`, `rm`, `vi`, `bc`, `ping`, `ipconfig`, etc.).
* **Deadlock Detection Research Lab**: Evaluates the Banker's algorithm across 6 resource allocation topologies with VGA bar charts.
* **HTML Web Browser**: Text-mode web browser with HTML rendering.
* **100% In-Browser**: Boots directly from `rustos.iso` via WebAssembly without requiring any server-side VM.
