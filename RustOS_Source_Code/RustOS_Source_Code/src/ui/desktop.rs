//! RustOS Friendly Windows-style Desktop Environment and Application Explorer

use alloc::format;
use alloc::string::String;
use crate::drivers::vga::{VGA_WRITER, Color, VGA_WIDTH, VGA_HEIGHT};
use crate::drivers::keyboard::{pop_key, KeyEvent};
use crate::drivers::rtc::Rtc;
use crate::drivers::pit;
use crate::net::{get_net, e1000::E1000};
use crate::shell::launch_shell;
use crate::apps::browser::ui::launch_browser;
use crate::apps::netsettings::launch_net_settings;
use crate::apps::filemanager::launch_file_manager;
use crate::apps::research::ui::launch_research_lab;
use crate::apps::sysinfo::launch_sysinfo;
use crate::apps::about::launch_about;

const MENU_ITEMS: [(&str, &str, &str); 7] = [
    ("Command Prompt",        "[Terminal]", "Interactive 32-bit Unix shell (ls, vi, ping, bc, cat)"),
    ("Web Browser",           "[Browser ]", "Built-in text web browser with HTML formatting & links"),
    ("Network Settings",      "[Network ]", "VirtualBox NAT/Bridge configuration & connection test"),
    ("My Documents (RAMFS)",  "[Explorer]", "Graphical RAM file manager with vi text document editor"),
    ("Deadlock Research Lab", "[Research]", "Banker's Algorithm & 6 empirical resource trials"),
    ("System & Memory Monitor","[Sys-Info]", "Live CPU info, 16 MB heap memory stats & kernel info"),
    ("About RustOS 1.0",      "[About-OS]", "Faculty project compendium, architecture & MIT license"),
];

const THEME_NAMES: [&str; 4] = [
    "XP Bliss Sky",
    "Win95 Teal",
    "Win2000 Navy",
    "Twilight Dark",
];

pub fn run_desktop() -> ! {
    let mut selected_index = 0usize;
    let mut theme_index = 0usize;
    let mut start_menu_open = false;
    let mut help_dialog_open = false;
    let mut need_redraw = true;
    let mut last_second = 255u8;

    loop {
        let dt = Rtc::read_datetime();
        if dt.second != last_second {
            last_second = dt.second;
            need_redraw = true;
        }

        if need_redraw {
            render_friendly_desktop(selected_index, theme_index, start_menu_open, help_dialog_open);
            need_redraw = false;
        }

        if let Some(key) = pop_key() {
            match key {
                KeyEvent::Up => {
                    if help_dialog_open {
                        // ignore or scroll
                    } else if start_menu_open {
                        if selected_index > 0 {
                            selected_index -= 1;
                        } else {
                            selected_index = MENU_ITEMS.len(); // 0..=7 (7 is shutdown)
                        }
                    } else {
                        if selected_index > 0 {
                            selected_index -= 1;
                        } else {
                            selected_index = MENU_ITEMS.len() - 1;
                        }
                    }
                    need_redraw = true;
                }
                KeyEvent::Down => {
                    if help_dialog_open {
                        // ignore or scroll
                    } else if start_menu_open {
                        if selected_index < MENU_ITEMS.len() {
                            selected_index += 1;
                        } else {
                            selected_index = 0;
                        }
                    } else {
                        if selected_index + 1 < MENU_ITEMS.len() {
                            selected_index += 1;
                        } else {
                            selected_index = 0;
                        }
                    }
                    need_redraw = true;
                }
                KeyEvent::Enter => {
                    if help_dialog_open {
                        help_dialog_open = false;
                        need_redraw = true;
                    } else if start_menu_open && selected_index == MENU_ITEMS.len() {
                        handle_shutdown();
                    } else {
                        let app = selected_index.min(MENU_ITEMS.len() - 1);
                        start_menu_open = false;
                        launch_application(app);
                        need_redraw = true;
                    }
                }
                KeyEvent::Char('1') => { start_menu_open = false; help_dialog_open = false; launch_application(0); need_redraw = true; }
                KeyEvent::Char('2') => { start_menu_open = false; help_dialog_open = false; launch_application(1); need_redraw = true; }
                KeyEvent::Char('3') => { start_menu_open = false; help_dialog_open = false; launch_application(2); need_redraw = true; }
                KeyEvent::Char('4') => { start_menu_open = false; help_dialog_open = false; launch_application(3); need_redraw = true; }
                KeyEvent::Char('5') => { start_menu_open = false; help_dialog_open = false; launch_application(4); need_redraw = true; }
                KeyEvent::Char('6') => { start_menu_open = false; help_dialog_open = false; launch_application(5); need_redraw = true; }
                KeyEvent::Char('7') => { start_menu_open = false; help_dialog_open = false; launch_application(6); need_redraw = true; }
                KeyEvent::Char('0') => {
                    handle_shutdown();
                }
                KeyEvent::Char('?') | KeyEvent::Char('h') | KeyEvent::Char('H') => {
                    help_dialog_open = !help_dialog_open;
                    start_menu_open = false;
                    need_redraw = true;
                }
                KeyEvent::Char('s') | KeyEvent::Char('S') | KeyEvent::Char(' ') => {
                    if help_dialog_open {
                        help_dialog_open = false;
                    } else {
                        start_menu_open = !start_menu_open;
                    }
                    need_redraw = true;
                }
                KeyEvent::Esc => {
                    if help_dialog_open {
                        help_dialog_open = false;
                        need_redraw = true;
                    } else if start_menu_open {
                        start_menu_open = false;
                        need_redraw = true;
                    }
                }
                KeyEvent::Char('b') | KeyEvent::Char('B') | KeyEvent::Char('w') | KeyEvent::Char('W') | KeyEvent::Tab => {
                    theme_index = (theme_index + 1) % THEME_NAMES.len();
                    need_redraw = true;
                }
                _ => {}
            }
        }
        pit::sleep_ms(30);
    }
}

fn launch_application(index: usize) {
    match index {
        0 => launch_shell(),
        1 => launch_browser(None),
        2 => launch_net_settings(),
        3 => launch_file_manager(),
        4 => launch_research_lab(),
        5 => launch_sysinfo(),
        6 => launch_about(),
        _ => {}
    }
}

fn handle_shutdown() -> ! {
    unsafe {
        VGA_WRITER.clear_screen();
        // Friendly, authentic Windows shutdown screen
        let msg1 = "Microsoft Windows is now shut down.";
        let msg2 = "It is now safe to turn off your computer.";
        let msg3 = "[ RustOS 1.0 - Thank you for testing! ]";

        VGA_WRITER.fill_rect(0, 0, VGA_WIDTH, VGA_HEIGHT, b' ', Color::White, Color::Black);
        VGA_WRITER.print_at((VGA_WIDTH - msg1.len()) / 2, 9, msg1, Color::LightRed, Color::Black);
        VGA_WRITER.print_at((VGA_WIDTH - msg2.len()) / 2, 11, msg2, Color::Yellow, Color::Black);
        VGA_WRITER.print_at((VGA_WIDTH - msg3.len()) / 2, 14, msg3, Color::LightCyan, Color::Black);

        // Power off via ACPI / VirtualBox I/O ports
        crate::arch::x86::io::outw(0x604, 0x2000);
        crate::arch::x86::io::outw(0x4004, 0x3400);
        crate::arch::x86::io::outw(0xB004, 0x2000);
        core::arch::asm!("cli");
        loop {
            core::arch::asm!("hlt");
        }
    }
}

fn draw_friendly_wallpaper(theme: usize) {
    unsafe {
        match theme {
            0 => {
                // Theme 0: Windows XP Bliss Landscape (Sky + Sunshine + Fluffy Clouds + Emerald Meadow)
                for y in 0..14 {
                    for x in 0..VGA_WIDTH {
                        // Fluffy clouds & golden sun
                        let is_sun = x >= 70 && x <= 75 && y >= 1 && y <= 3;
                        let is_cloud = (x >= 40 && x <= 62 && y >= 3 && y <= 6)
                            || (x >= 8 && x <= 22 && y >= 2 && y <= 4)
                            || (x >= 28 && x <= 36 && y >= 7 && y <= 8);

                        let (ch, fg, bg) = if is_sun {
                            (15u8, Color::Yellow, Color::LightCyan) // ☼
                        } else if is_cloud {
                            (0xB0, Color::White, Color::LightCyan)
                        } else {
                            (b' ', Color::White, Color::LightCyan)
                        };
                        VGA_WRITER.fill_rect(x, y, 1, 1, ch, fg, bg);
                    }
                }
                for y in 14..23 {
                    for x in 0..VGA_WIDTH {
                        // Rolling green meadow with floral accents
                        let is_flower = (x * 13 + y * 7) % 37 == 0;
                        let (ch, fg) = if is_flower {
                            (15u8, Color::Yellow) // flower ☼
                        } else if (x + y) % 2 == 0 {
                            (0xB1, Color::DarkGray)
                        } else {
                            (0xB0, Color::LightGreen)
                        };
                        VGA_WRITER.fill_rect(x, y, 1, 1, ch, fg, Color::Green);
                    }
                }
            }
            1 => {
                // Theme 1: Windows 95/98 Classic Teal (#008080)
                for y in 0..23 {
                    for x in 0..VGA_WIDTH {
                        let ch = if (x + y) % 2 == 0 { 0xB0 } else { 0xB1 };
                        VGA_WRITER.fill_rect(x, y, 1, 1, ch, Color::DarkGray, Color::Cyan);
                    }
                }
            }
            2 => {
                // Theme 2: Windows 2000 Professional Navy
                for y in 0..23 {
                    for x in 0..VGA_WIDTH {
                        let ch = if (x + y * 2) % 6 == 0 { 0xB0 } else { b' ' };
                        VGA_WRITER.fill_rect(x, y, 1, 1, ch, Color::LightCyan, Color::Blue);
                    }
                }
            }
            _ => {
                // Theme 3: Twilight Dream (Soft dusk gradient with stars)
                for y in 0..23 {
                    for x in 0..VGA_WIDTH {
                        let is_star = ((x as u32 * 7919) ^ (y as u32 * 6271)) % 100 < 3;
                        let (ch, fg, bg) = if is_star {
                            (b'*', Color::Yellow, Color::Black)
                        } else if (x + y) % 5 == 0 {
                            (0xB0, Color::Blue, Color::Black)
                        } else {
                            (b' ', Color::Black, Color::Black)
                        };
                        VGA_WRITER.fill_rect(x, y, 1, 1, ch, fg, bg);
                    }
                }
            }
        }

        // Friendly Desktop Shortcut Icons on the left (Columns 1 to 14)
        draw_desktop_icon(1, 1,  "[#] My PC",        "Computer");
        draw_desktop_icon(1, 4,  "[@] Internet",     "Browser");
        draw_desktop_icon(1, 7,  "[%] My Documents", "RAM Files");
        draw_desktop_icon(1, 10, "[&] Network",      "LAN Setup");
        draw_desktop_icon(1, 13, "[$] Research Lab", "Banker's");
        draw_desktop_icon(1, 16, "[!] Terminal",     "Command");
        draw_desktop_icon(1, 19, "[?] Quick Help",   "Press [?]");
    }
}

fn draw_desktop_icon(x: usize, y: usize, icon: &str, label: &str) {
    unsafe {
        VGA_WRITER.print_at(x, y, icon, Color::Yellow, Color::Black);
        VGA_WRITER.print_at(x, y + 1, label, Color::White, Color::Black);
    }
}

fn render_friendly_desktop(selected: usize, theme: usize, start_open: bool, help_open: bool) {
    let dt = Rtc::read_datetime();
    let net = get_net();
    let is_connected = unsafe { E1000.as_ref().map(|n| n.link_up).unwrap_or(true) };

    // Friendly greeting based on current hardware RTC time
    let greeting = if dt.hour >= 5 && dt.hour < 12 {
        "Good morning, Subhranshu! Welcome back."
    } else if dt.hour >= 12 && dt.hour < 17 {
        "Good afternoon, Subhranshu! Welcome back."
    } else if dt.hour >= 17 && dt.hour < 22 {
        "Good evening, Subhranshu! Welcome back."
    } else {
        "Working late, Subhranshu? Welcome back."
    };

    unsafe {
        VGA_WRITER.disable_cursor();

        // 1. Draw Wallpaper & Desktop Icons
        draw_friendly_wallpaper(theme);

        // 2. Main "RustOS Applications Explorer" Window
        let win_x = 16;
        let win_y = 1;
        let win_w = 61;
        let win_h = 21;

        // Window 3D Drop Shadow (Right and Bottom)
        VGA_WRITER.fill_rect(win_x + win_w, win_y + 1, 2, win_h, 0xB2, Color::Black, Color::DarkGray);
        VGA_WRITER.fill_rect(win_x + 2, win_y + win_h, win_w, 1, 0xB2, Color::Black, Color::DarkGray);

        // Window Frame Background (Classic Light Gray)
        VGA_WRITER.fill_rect(win_x, win_y, win_w, win_h, b' ', Color::Black, Color::LightGray);
        VGA_WRITER.draw_box(win_x, win_y, win_w, win_h, Color::White, Color::LightGray, false);

        // Window Title Bar (Row 1, Royal Blue with White Title and Control Buttons)
        VGA_WRITER.fill_rect(win_x + 1, win_y, win_w - 2, 1, b' ', Color::White, Color::Blue);
        VGA_WRITER.print_at(win_x + 2, win_y, " RustOS Explorer - Welcome Center ", Color::White, Color::Blue);
        VGA_WRITER.print_at(win_x + win_w - 11, win_y, "[_] [□] [X]", Color::Yellow, Color::Blue);

        // Friendly Menu Bar (Row 2, Light Gray with Mnemonics)
        VGA_WRITER.fill_rect(win_x + 1, win_y + 1, win_w - 2, 1, b' ', Color::Black, Color::LightGray);
        VGA_WRITER.print_at(win_x + 2, win_y + 1, "File(F)   Edit(E)   View(V)   Tools(T)   Help(H)", Color::Black, Color::LightGray);

        // Friendly Personalized Greeting Bar (Row 3)
        VGA_WRITER.fill_rect(win_x + 1, win_y + 2, win_w - 2, 1, b' ', Color::DarkGray, Color::LightGray);
        VGA_WRITER.print_at(win_x + 2, win_y + 2, "Address:", Color::Black, Color::LightGray);
        VGA_WRITER.fill_rect(win_x + 11, win_y + 2, win_w - 14, 1, b' ', Color::Black, Color::White);
        let greeting_trimmed = if greeting.len() > win_w - 16 { &greeting[..win_w - 16] } else { greeting };
        VGA_WRITER.print_at(win_x + 12, win_y + 2, greeting_trimmed, Color::Blue, Color::White);
        VGA_WRITER.print_at(win_x + win_w - 5, win_y + 2, "[▼]", Color::Black, Color::LightGray);

        // Inner White File Explorer Viewport (Rows win_y + 3 to win_y + win_h - 4)
        let vp_x = win_x + 2;
        let vp_y = win_y + 3;
        let vp_w = win_w - 4;
        let vp_h = 13;

        VGA_WRITER.fill_rect(vp_x, vp_y, vp_w, vp_h, b' ', Color::Black, Color::White);
        VGA_WRITER.draw_box(vp_x, vp_y, vp_w, vp_h, Color::DarkGray, Color::White, false);

        // Viewport Header Label
        VGA_WRITER.print_at(vp_x + 2, vp_y, " [ All Applications & Tools ] ", Color::DarkGray, Color::White);

        // Applications List Inside Viewport
        for (i, (title, tag, _desc)) in MENU_ITEMS.iter().enumerate() {
            let item_y = vp_y + 2 + i;
            let is_sel = (i == selected) && !start_open && !help_open;

            if is_sel {
                // Cheerful Royal Blue highlight with Gold pointer and badge
                VGA_WRITER.fill_rect(vp_x + 1, item_y, vp_w - 2, 1, b' ', Color::White, Color::Blue);
                VGA_WRITER.print_at(vp_x + 2, item_y, "►", Color::Yellow, Color::Blue);
                VGA_WRITER.print_at(vp_x + 4, item_y, tag, Color::Yellow, Color::Blue);
                let item_text = format!("{}. {}", i + 1, title);
                VGA_WRITER.print_at(vp_x + 16, item_y, &item_text, Color::White, Color::Blue);
                VGA_WRITER.print_at(vp_x + vp_w - 8, item_y, "[Open]", Color::LightCyan, Color::Blue);
            } else {
                VGA_WRITER.fill_rect(vp_x + 1, item_y, vp_w - 2, 1, b' ', Color::Black, Color::White);
                VGA_WRITER.print_at(vp_x + 4, item_y, tag, Color::Blue, Color::White);
                let item_text = format!("{}. {}", i + 1, title);
                VGA_WRITER.print_at(vp_x + 16, item_y, &item_text, Color::Black, Color::White);
            }
        }

        // Viewport Tip at the bottom of the list
        VGA_WRITER.print_at(vp_x + 3, vp_y + vp_h - 2, "Tip: Use [↑/↓] to select, [Enter] to run, or press 1-7", Color::DarkGray, Color::White);

        // Active Application Description Footer (Row win_y + win_h - 3)
        let desc = if selected < MENU_ITEMS.len() { MENU_ITEMS[selected].2 } else { "Power off the virtual machine safely" };
        VGA_WRITER.fill_rect(win_x + 1, win_y + win_h - 3, win_w - 2, 1, b' ', Color::Black, Color::LightGray);
        VGA_WRITER.print_at(win_x + 2, win_y + win_h - 3, "Info: ", Color::Black, Color::LightGray);
        let desc_trimmed = if desc.len() > win_w - 10 { &desc[..win_w - 10] } else { desc };
        VGA_WRITER.print_at(win_x + 8, win_y + win_h - 3, desc_trimmed, Color::Blue, Color::LightGray);

        // Window System Capacity (Row win_y + win_h - 2)
        VGA_WRITER.fill_rect(win_x + 1, win_y + win_h - 2, win_w - 2, 1, b' ', Color::DarkGray, Color::LightGray);
        VGA_WRITER.print_at(win_x + 2, win_y + win_h - 2, "7 apps ready to run", Color::DarkGray, Color::LightGray);
        VGA_WRITER.print_at(win_x + win_w - 21, win_y + win_h - 2, "16 MB Heap Available", Color::DarkGray, Color::LightGray);

        // 3. Helpful Keyboard Shortcut Bar (Row 23, Right above Taskbar)
        VGA_WRITER.fill_rect(0, 23, VGA_WIDTH, 1, b' ', Color::White, Color::DarkGray);
        VGA_WRITER.print_at(1, 23, "[↑/↓] Select", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(16, 23, "[Enter] Open", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(31, 23, "[1-7] Direct", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(46, 23, "[S] Start Menu", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(62, 23, "[Tab] Theme", Color::Yellow, Color::DarkGray);
        VGA_WRITER.print_at(74, 23, "[?] Help", Color::Yellow, Color::DarkGray);

        // 4. Start Menu Popup (if open)
        if start_open {
            let sm_x = 0;
            let sm_y = 8;
            let sm_w = 32;
            let sm_h = 15;

            // Start Menu Drop Shadow
            VGA_WRITER.fill_rect(sm_x + sm_w, sm_y + 1, 2, sm_h, 0xB2, Color::Black, Color::DarkGray);

            // Start Menu Body
            VGA_WRITER.fill_rect(sm_x, sm_y, sm_w, sm_h, b' ', Color::Black, Color::LightGray);
            VGA_WRITER.draw_box(sm_x, sm_y, sm_w, sm_h, Color::White, Color::LightGray, false);

            // Left Vertical Blue "RustOS 95" Banner
            VGA_WRITER.fill_rect(sm_x + 1, sm_y + 1, 3, sm_h - 2, b' ', Color::White, Color::Blue);
            let banner = ['R', 'u', 's', 't', 'O', 'S', ' ', '9', '5'];
            for (bi, bc) in banner.iter().enumerate() {
                if bi < sm_h - 3 {
                    VGA_WRITER.print_at(sm_x + 2, sm_y + 2 + bi, core::str::from_utf8_unchecked(&[*bc as u8]), Color::Yellow, Color::Blue);
                }
            }

            // User Header in Start Menu
            VGA_WRITER.fill_rect(sm_x + 5, sm_y + 1, sm_w - 6, 2, b' ', Color::Black, Color::LightGray);
            VGA_WRITER.print_at(sm_x + 5, sm_y + 1, "[☺] Subhranshu", Color::Blue, Color::LightGray);
            VGA_WRITER.print_at(sm_x + 9, sm_y + 2, "System User", Color::DarkGray, Color::LightGray);
            VGA_WRITER.fill_rect(sm_x + 5, sm_y + 3, sm_w - 6, 1, 196u8, Color::DarkGray, Color::LightGray);

            // Start Menu Application Items
            for (i, (title, _, _)) in MENU_ITEMS.iter().enumerate() {
                let sy = sm_y + 4 + i;
                let is_sel = selected == i;
                let short_title = if title.len() > 18 { &title[..18] } else { title };

                if is_sel {
                    VGA_WRITER.fill_rect(sm_x + 5, sy, sm_w - 6, 1, b' ', Color::White, Color::Blue);
                    let label = format!("{}. {}", i + 1, short_title);
                    VGA_WRITER.print_at(sm_x + 6, sy, &label, Color::White, Color::Blue);
                } else {
                    VGA_WRITER.fill_rect(sm_x + 5, sy, sm_w - 6, 1, b' ', Color::Black, Color::LightGray);
                    let label = format!("{}. {}", i + 1, short_title);
                    VGA_WRITER.print_at(sm_x + 6, sy, &label, Color::Black, Color::LightGray);
                }
            }

            // Divider Line
            let div_y = sm_y + 4 + MENU_ITEMS.len();
            VGA_WRITER.fill_rect(sm_x + 5, div_y, sm_w - 6, 1, 196u8, Color::DarkGray, Color::LightGray);

            // Shut Down Option
            let shut_y = div_y + 1;
            let is_shut_sel = selected == MENU_ITEMS.len();
            if is_shut_sel {
                VGA_WRITER.fill_rect(sm_x + 5, shut_y, sm_w - 6, 1, b' ', Color::White, Color::Blue);
                VGA_WRITER.print_at(sm_x + 6, shut_y, "0. Shut Down...", Color::Yellow, Color::Blue);
            } else {
                VGA_WRITER.fill_rect(sm_x + 5, shut_y, sm_w - 6, 1, b' ', Color::Black, Color::LightGray);
                VGA_WRITER.print_at(sm_x + 6, shut_y, "0. Shut Down...", Color::Black, Color::LightGray);
            }
        }

        // 5. Friendly "Quick Help & Tips" Pop-up Dialog (if open)
        if help_open {
            let hx = 16;
            let hy = 4;
            let hw = 50;
            let hh = 15;

            // Dialog Drop Shadow
            VGA_WRITER.fill_rect(hx + hw, hy + 1, 2, hh, 0xB2, Color::Black, Color::DarkGray);
            VGA_WRITER.fill_rect(hx + 2, hy + hh, hw, 1, 0xB2, Color::Black, Color::DarkGray);

            // Dialog Frame
            VGA_WRITER.fill_rect(hx, hy, hw, hh, b' ', Color::Black, Color::LightGray);
            VGA_WRITER.draw_box(hx, hy, hw, hh, Color::White, Color::LightGray, false);

            // Dialog Title
            VGA_WRITER.fill_rect(hx + 1, hy, hw - 2, 1, b' ', Color::White, Color::Blue);
            VGA_WRITER.print_at(hx + 2, hy, " ✿ Welcome to RustOS - Quick Guide ✿ ", Color::White, Color::Blue);
            VGA_WRITER.print_at(hx + hw - 5, hy, "[X]", Color::Yellow, Color::Blue);

            // Friendly Help Text Content
            VGA_WRITER.print_at(hx + 3, hy + 2, "Welcome to RustOS! Here are some friendly tips:", Color::Black, Color::LightGray);
            VGA_WRITER.print_at(hx + 3, hy + 4, "• Terminal (1): 33 commands (ls, vi, bc, ping)", Color::Blue, Color::LightGray);
            VGA_WRITER.print_at(hx + 3, hy + 5, "• Browser (2) : Built-in text web browser", Color::Blue, Color::LightGray);
            VGA_WRITER.print_at(hx + 3, hy + 6, "• Files (4)   : Create & edit notes in RAMFS", Color::Blue, Color::LightGray);
            VGA_WRITER.print_at(hx + 3, hy + 7, "• Research (5): Deadlock detection experiments", Color::Blue, Color::LightGray);
            VGA_WRITER.print_at(hx + 3, hy + 8, "• Wallpapers  : Press [Tab] to cycle 4 themes", Color::Black, Color::LightGray);
            VGA_WRITER.print_at(hx + 3, hy + 9, "• Start Menu  : Press [S] or [Space] anytime", Color::Black, Color::LightGray);
            VGA_WRITER.print_at(hx + 3, hy + 10, "• Turn Off    : Press [0] to safely halt system", Color::Black, Color::LightGray);

            // Friendly Dismiss Button
            VGA_WRITER.fill_rect(hx + 14, hy + 12, hw - 28, 1, b' ', Color::White, Color::Blue);
            VGA_WRITER.print_at(hx + 16, hy + 12, "[ OK - Press Enter ]", Color::Yellow, Color::Blue);
        }

        // 6. Classic Windows Taskbar (Row 24, Bottom of Screen)
        VGA_WRITER.fill_rect(0, 24, VGA_WIDTH, 1, b' ', Color::Black, Color::LightGray);

        // Start Button (Cols 0 to 11)
        if start_open {
            VGA_WRITER.fill_rect(0, 24, 11, 1, b' ', Color::White, Color::DarkGray);
            VGA_WRITER.print_at(1, 24, "[▼ Start]", Color::Yellow, Color::DarkGray);
        } else {
            VGA_WRITER.fill_rect(0, 24, 11, 1, b' ', Color::Black, Color::LightGray);
            VGA_WRITER.print_at(1, 24, "[■ Start]", Color::Black, Color::LightGray);
        }

        // Active Taskbar Tab: "Applications Explorer" (Cols 13 to 38)
        VGA_WRITER.fill_rect(13, 24, 24, 1, b' ', Color::Black, Color::White);
        VGA_WRITER.print_at(14, 24, "■ Welcome Center", Color::Blue, Color::White);

        // Friendly Help Pill on Taskbar (Cols 38 to 47)
        VGA_WRITER.fill_rect(39, 24, 9, 1, b' ', Color::Black, Color::LightGray);
        VGA_WRITER.print_at(40, 24, "[? Help]", Color::Blue, Color::LightGray);

        // Theme indicator pill in System Tray
        let theme_str = format!("[{}]", THEME_NAMES[theme]);
        VGA_WRITER.print_at(50, 24, &theme_str, Color::DarkGray, Color::LightGray);

        // Network Indicator in System Tray
        let net_str = if is_connected {
            String::from("[LAN:ONLINE]")
        } else {
            String::from("[LAN:OFFLINE]")
        };
        let net_color = if is_connected { Color::Green } else { Color::Red };
        VGA_WRITER.print_at(64, 24, &net_str, net_color, Color::LightGray);

        // System Tray Sunken Clock (Cols 71 to 79)
        let clock_str = format!("{:02}:{:02}", dt.hour, dt.minute);
        VGA_WRITER.print_at(74, 24, &clock_str, Color::Black, Color::LightGray);
    }
}
