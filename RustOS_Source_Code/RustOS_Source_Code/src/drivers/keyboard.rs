//! PS/2 Keyboard Driver (Scancode Set 1)

use crate::arch::x86::io::inb;

const KEYBOARD_PORT: u16 = 0x60;
const BUFFER_CAPACITY: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Esc,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

struct KeyboardState {
    shift: bool,
    caps_lock: bool,
    ctrl: bool,
    alt: bool,
    extended: bool,
    buffer: [KeyEvent; BUFFER_CAPACITY],
    head: usize,
    tail: usize,
}

static mut KEYBOARD: KeyboardState = KeyboardState {
    shift: false,
    caps_lock: false,
    ctrl: false,
    alt: false,
    extended: false,
    buffer: [KeyEvent::Enter; BUFFER_CAPACITY],
    head: 0,
    tail: 0,
};

pub unsafe fn init_keyboard() {
    use crate::arch::x86::io::outb;
    // 1. Disable second PS/2 port (mouse) to prevent mouse packets from clogging port 0x60
    outb(0x64, 0xA7);

    // 2. Enable first PS/2 port (keyboard)
    outb(0x64, 0xAE);

    // 3. Read Controller Configuration Byte
    outb(0x64, 0x20);
    let mut config = inb(KEYBOARD_PORT);
    config |= 0x01;   // Enable IRQ 1 (Keyboard)
    config &= !0x02;  // Disable IRQ 12 (Mouse)
    config &= !0x10;  // Enable keyboard clock (0 = enabled)
    config |= 0x20;   // Disable mouse clock (1 = disabled)
    outb(0x64, 0x60); // Write Command Byte
    outb(KEYBOARD_PORT, config);

    // 4. Flush any pending scancodes or mouse bytes
    while (inb(0x64) & 1) != 0 {
        let _ = inb(KEYBOARD_PORT);
    }
}

pub fn on_keyboard_interrupt() {
    unsafe {
        let status = inb(0x64);
        if (status & 1) != 0 {
            let scancode = inb(KEYBOARD_PORT);
            // Bit 5 is 0 for keyboard, 1 for mouse
            if (status & 0x20) == 0 {
                process_scancode(scancode);
            }
        }
    }
}

unsafe fn process_scancode(scancode: u8) {
    if scancode == 0xE0 {
        KEYBOARD.extended = true;
        return;
    }

    let is_release = (scancode & 0x80) != 0;
    let code = scancode & 0x7F;

    if KEYBOARD.extended {
        KEYBOARD.extended = false;
        if !is_release {
            match code {
                0x48 => push_event(KeyEvent::Up),
                0x50 => push_event(KeyEvent::Down),
                0x4B => push_event(KeyEvent::Left),
                0x4D => push_event(KeyEvent::Right),
                0x49 => push_event(KeyEvent::PageUp),
                0x51 => push_event(KeyEvent::PageDown),
                0x47 => push_event(KeyEvent::Home),
                0x4F => push_event(KeyEvent::End),
                0x53 => push_event(KeyEvent::Delete),
                _ => {}
            }
        }
        return;
    }

    match code {
        0x2A | 0x36 => {
            // Left or Right Shift
            KEYBOARD.shift = !is_release;
        }
        0x3A => {
            // Caps Lock
            if !is_release {
                KEYBOARD.caps_lock = !KEYBOARD.caps_lock;
            }
        }
        0x1D => {
            // Ctrl
            KEYBOARD.ctrl = !is_release;
        }
        0x38 => {
            // Alt
            KEYBOARD.alt = !is_release;
        }
        _ => {
            if !is_release {
                if let Some(event) = translate_scancode(code, KEYBOARD.shift, KEYBOARD.caps_lock) {
                    push_event(event);
                }
            }
        }
    }
}

fn push_event(event: KeyEvent) {
    unsafe {
        let next_head = (KEYBOARD.head + 1) % BUFFER_CAPACITY;
        if next_head != KEYBOARD.tail {
            KEYBOARD.buffer[KEYBOARD.head] = event;
            KEYBOARD.head = next_head;
        }
    }
}

pub fn pop_key() -> Option<KeyEvent> {
    unsafe {
        // Fallback polling: read port 0x60 only if data is available and from keyboard (bit 5 == 0)
        let status = inb(0x64);
        if (status & 1) != 0 {
            let scancode = inb(KEYBOARD_PORT);
            if (status & 0x20) == 0 {
                process_scancode(scancode);
            }
        }

        if KEYBOARD.head == KEYBOARD.tail {
            None
        } else {
            let event = KEYBOARD.buffer[KEYBOARD.tail];
            KEYBOARD.tail = (KEYBOARD.tail + 1) % BUFFER_CAPACITY;
            Some(event)
        }
    }
}

pub fn read_char() -> char {
    loop {
        if let Some(event) = pop_key() {
            match event {
                KeyEvent::Char(c) => return c,
                KeyEvent::Enter => return '\n',
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

fn translate_scancode(code: u8, shift: bool, caps: bool) -> Option<KeyEvent> {
    match code {
        0x01 => Some(KeyEvent::Esc),
        0x0E => Some(KeyEvent::Backspace),
        0x0F => Some(KeyEvent::Tab),
        0x1C => Some(KeyEvent::Enter),
        0x3B => Some(KeyEvent::F1),
        0x3C => Some(KeyEvent::F2),
        0x3D => Some(KeyEvent::F3),
        0x3E => Some(KeyEvent::F4),
        0x3F => Some(KeyEvent::F5),
        0x40 => Some(KeyEvent::F6),
        0x41 => Some(KeyEvent::F7),
        0x42 => Some(KeyEvent::F8),
        0x43 => Some(KeyEvent::F9),
        0x44 => Some(KeyEvent::F10),
        0x57 => Some(KeyEvent::F11),
        0x58 => Some(KeyEvent::F12),
        _ => {
            let ch = scancode_to_char(code, shift, caps)?;
            Some(KeyEvent::Char(ch))
        }
    }
}

fn scancode_to_char(code: u8, shift: bool, caps: bool) -> Option<char> {
    let letter_upper = shift ^ caps;
    match code {
        0x02 => Some(if shift { '!' } else { '1' }),
        0x03 => Some(if shift { '@' } else { '2' }),
        0x04 => Some(if shift { '#' } else { '3' }),
        0x05 => Some(if shift { '$' } else { '4' }),
        0x06 => Some(if shift { '%' } else { '5' }),
        0x07 => Some(if shift { '^' } else { '6' }),
        0x08 => Some(if shift { '&' } else { '7' }),
        0x09 => Some(if shift { '*' } else { '8' }),
        0x0A => Some(if shift { '(' } else { '9' }),
        0x0B => Some(if shift { ')' } else { '0' }),
        0x0C => Some(if shift { '_' } else { '-' }),
        0x0D => Some(if shift { '+' } else { '=' }),
        0x10 => Some(if letter_upper { 'Q' } else { 'q' }),
        0x11 => Some(if letter_upper { 'W' } else { 'w' }),
        0x12 => Some(if letter_upper { 'E' } else { 'e' }),
        0x13 => Some(if letter_upper { 'R' } else { 'r' }),
        0x14 => Some(if letter_upper { 'T' } else { 't' }),
        0x15 => Some(if letter_upper { 'Y' } else { 'y' }),
        0x16 => Some(if letter_upper { 'U' } else { 'u' }),
        0x17 => Some(if letter_upper { 'I' } else { 'i' }),
        0x18 => Some(if letter_upper { 'O' } else { 'o' }),
        0x19 => Some(if letter_upper { 'P' } else { 'p' }),
        0x1A => Some(if shift { '{' } else { '[' }),
        0x1B => Some(if shift { '}' } else { ']' }),
        0x1E => Some(if letter_upper { 'A' } else { 'a' }),
        0x1F => Some(if letter_upper { 'S' } else { 's' }),
        0x20 => Some(if letter_upper { 'D' } else { 'd' }),
        0x21 => Some(if letter_upper { 'F' } else { 'f' }),
        0x22 => Some(if letter_upper { 'G' } else { 'g' }),
        0x23 => Some(if letter_upper { 'H' } else { 'h' }),
        0x24 => Some(if letter_upper { 'J' } else { 'j' }),
        0x25 => Some(if letter_upper { 'K' } else { 'k' }),
        0x26 => Some(if letter_upper { 'L' } else { 'l' }),
        0x27 => Some(if shift { ':' } else { ';' }),
        0x28 => Some(if shift { '"' } else { '\'' }),
        0x29 => Some(if shift { '~' } else { '`' }),
        0x2B => Some(if shift { '|' } else { '\\' }),
        0x2C => Some(if letter_upper { 'Z' } else { 'z' }),
        0x2D => Some(if letter_upper { 'X' } else { 'x' }),
        0x2E => Some(if letter_upper { 'C' } else { 'c' }),
        0x2F => Some(if letter_upper { 'V' } else { 'v' }),
        0x30 => Some(if letter_upper { 'B' } else { 'b' }),
        0x31 => Some(if letter_upper { 'N' } else { 'n' }),
        0x32 => Some(if letter_upper { 'M' } else { 'm' }),
        0x33 => Some(if shift { '<' } else { ',' }),
        0x34 => Some(if shift { '>' } else { '.' }),
        0x35 => Some(if shift { '?' } else { '/' }),
        0x39 => Some(' '), // Space bar
        _ => None,
    }
}
