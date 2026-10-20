//! Veyron OS Command Shell (Terminal)
//! Interactive command-line environment with file commands, history, and neofetch.

use crate::drivers::keyboard::{pop_event, KeyCode};
use crate::fs::vfs::{list_dir, read_file};
use crate::fs::fat32::FsEntry;
use crate::kprintln;

pub struct Terminal {
    pub cwd: [u8; 128],
    pub cwd_len: usize,
    pub line_buf: [u8; 256],
    pub line_len: usize,
}

impl Terminal {
    pub const fn new() -> Self {
        Self {
            cwd: [0; 128],
            cwd_len: 0,
            line_buf: [0; 256],
            line_len: 0,
        }
    }

    pub fn init(&mut self) {
        let initial_path = b"Users/Default/Desktop";
        self.cwd[..initial_path.len()].copy_from_slice(initial_path);
        self.cwd_len = initial_path.len();

        self.print_banner();
        self.print_prompt();
    }

    pub fn cwd_str(&self) -> &str {
        core::str::from_utf8(&self.cwd[..self.cwd_len]).unwrap_or("")
    }

    pub fn print_prompt(&self) {
        crate::kprint!("C:\\{}> ", self.cwd_str());
    }

    pub fn print_banner(&self) {
        kprintln!("Veyron OS [Version 0.1.0]");
        kprintln!("(c) 2026 Veyron Corporation. All rights reserved.\n");
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                if self.line_len < self.line_buf.len() - 1 {
                    self.line_buf[self.line_len] = c as u8;
                    self.line_len += 1;
                    crate::kprint!("{}", c);
                }
            }
            KeyCode::Backspace => {
                if self.line_len > 0 {
                    self.line_len -= 1;
                    // Move back, print space, move back
                    unsafe {
                        if crate::framebuffer::CONSOLE.cursor_x > 0 {
                            crate::framebuffer::CONSOLE.cursor_x -= 1;
                            let px = crate::framebuffer::CONSOLE.margin_x + crate::framebuffer::CONSOLE.cursor_x * crate::font::FONT_WIDTH;
                            let py = crate::framebuffer::CONSOLE.margin_y + crate::framebuffer::CONSOLE.cursor_y * crate::font::FONT_HEIGHT;
                            crate::framebuffer::FRAMEBUFFER.fill_rect(
                                px, py, crate::font::FONT_WIDTH, crate::font::FONT_HEIGHT, crate::framebuffer::CONSOLE.bg_color
                            );
                        }
                    }
                }
            }
            KeyCode::Enter => {
                kprintln!();
                self.execute_line();
                self.line_len = 0;
                self.print_prompt();
            }
            KeyCode::Tab => {
                // Auto-complete files/folders in CWD
                self.autocomplete();
            }
            _ => {}
        }
    }

    pub fn execute_line(&mut self) {
        let line = match core::str::from_utf8(&self.line_buf[..self.line_len]) {
            Ok(s) => s.trim(),
            Err(_) => return,
        };

        if line.is_empty() {
            return;
        }

        let mut parts = line.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let arg = parts.next().unwrap_or("");

        match cmd {
            "help" => {
                kprintln!("Veyron OS Terminal Shell - Supported Commands:");
                kprintln!("  help            - Display this list of commands");
                kprintln!("  ver             - Display operating system version");
                kprintln!("  cls             - Clear the screen buffer");
                kprintln!("  dir [path]      - List directory contents");
                kprintln!("  cd <path>       - Change directory");
                kprintln!("  type <file>     - Display contents of a text file");
                kprintln!("  echo <message>  - Display a message on console");
                kprintln!("  date            - Show current system date");
                kprintln!("  time            - Show current system time");
                kprintln!("  whoami          - Display current logged in user");
                kprintln!("  tree            - Show system directory tree");
                kprintln!("  neofetch        - Display system specifications & ASCII logo");
                kprintln!("  start <app>     - Launch GUI desktop application");
            }
            "ver" => {
                kprintln!("Veyron OS Kernel Version 0.1.0-uefi (x86_64, Scratch Core)");
            }
            "cls" => {
                unsafe {
                    crate::framebuffer::CONSOLE.init();
                }
            }
            "whoami" => {
                kprintln!("Veyron\\User (Administrator)");
            }
            "date" => {
                let t = crate::drivers::rtc::read_rtc();
                kprintln!("The current date is: {:04}-{:02}-{:02}", t.year, t.month, t.day);
            }
            "time" => {
                let t = crate::drivers::rtc::read_rtc();
                kprintln!("The current time is: {:02}:{:02}:{:02}", t.hours, t.minutes, t.seconds);
            }
            "dir" => {
                let target = if arg.is_empty() { self.cwd_str() } else { arg };
                let mut entries: [FsEntry; 64] = [const { FsEntry::empty() }; 64];
                let count = list_dir(target, &mut entries);

                kprintln!(" Directory of C:\\{}", target);
                kprintln!();

                let mut total_files = 0;
                let mut total_bytes = 0;
                for i in 0..count {
                    let entry = &entries[i];
                    if entry.is_dir {
                        kprintln!("  <DIR>          {}", entry.name_as_str());
                    } else {
                        kprintln!("       {:>8}  {}", entry.size, entry.name_as_str());
                        total_files += 1;
                        total_bytes += entry.size;
                    }
                }
                kprintln!();
                kprintln!("      {} File(s)    {} bytes", total_files, total_bytes);
            }
            "cd" => {
                if arg == ".." {
                    if let Some(idx) = self.cwd_str().rfind('/') {
                        self.cwd_len = idx;
                    } else {
                        self.cwd_len = 0;
                    }
                } else if !arg.is_empty() {
                    let mut test_path = [0u8; 128];
                    let mut len = 0;
                    if self.cwd_len > 0 {
                        test_path[..self.cwd_len].copy_from_slice(&self.cwd[..self.cwd_len]);
                        test_path[self.cwd_len] = b'/';
                        len = self.cwd_len + 1;
                    }
                    test_path[len..len + arg.len()].copy_from_slice(arg.as_bytes());
                    len += arg.len();

                    let test_str = core::str::from_utf8(&test_path[..len]).unwrap_or("");
                    let mut entries: [FsEntry; 1] = [FsEntry::empty()];
                    if list_dir(test_str, &mut entries) > 0 || arg == "/" {
                        self.cwd[..len].copy_from_slice(&test_path[..len]);
                        self.cwd_len = len;
                    } else {
                        kprintln!("The system cannot find the path specified.");
                    }
                }
            }
            "type" => {
                if arg.is_empty() {
                    kprintln!("Syntax: type <filename>");
                } else {
                    let mut full_path = [0u8; 128];
                    let mut len = 0;
                    if self.cwd_len > 0 {
                        full_path[..self.cwd_len].copy_from_slice(&self.cwd[..self.cwd_len]);
                        full_path[self.cwd_len] = b'/';
                        len = self.cwd_len + 1;
                    }
                    full_path[len..len + arg.len()].copy_from_slice(arg.as_bytes());
                    len += arg.len();

                    let path_str = core::str::from_utf8(&full_path[..len]).unwrap_or("");
                    let mut buf = [0u8; 1024];
                    let read_len = read_file(path_str, &mut buf);
                    if read_len > 0 {
                        if let Ok(text) = core::str::from_utf8(&buf[..read_len]) {
                            kprintln!("{}", text);
                        }
                    } else {
                        kprintln!("File not found or cannot be read.");
                    }
                }
            }
            "echo" => {
                let rest = line.strip_prefix("echo").unwrap_or("").trim();
                kprintln!("{}", rest);
            }
            "tree" => {
                kprintln!("C:\\");
                kprintln!("|-- EFI");
                kprintln!("|   `-- BOOT");
                kprintln!("|-- boot");
                kprintln!("|-- System");
                kprintln!("|-- Apps");
                kprintln!("`-- Users");
                kprintln!("    `-- Default");
                kprintln!("        |-- Desktop");
                kprintln!("        |-- Documents");
                kprintln!("        |-- Downloads");
                kprintln!("        `-- Pictures");
            }
            "neofetch" => {
                self.print_neofetch();
            }
            _ => {
                kprintln!("'{}' is not recognized as an internal command.", cmd);
            }
        }
    }

    pub fn autocomplete(&mut self) {
        let current_line = core::str::from_utf8(&self.line_buf[..self.line_len]).unwrap_or("");
        let prefix = current_line.split_whitespace().last().unwrap_or("");
        if prefix.is_empty() { return; }

        let mut entries: [FsEntry; 32] = [const { FsEntry::empty() }; 32];
        let count = list_dir(self.cwd_str(), &mut entries);

        for i in 0..count {
            let name = entries[i].name_as_str();
            if name.starts_with(prefix) && name.len() > prefix.len() {
                let completion = &name[prefix.len()..];
                for b in completion.bytes() {
                    if self.line_len < self.line_buf.len() - 1 {
                        self.line_buf[self.line_len] = b;
                        self.line_len += 1;
                        crate::kprint!("{}", b as char);
                    }
                }
                break;
            }
        }
    }

    pub fn print_neofetch(&self) {
        unsafe {
            let total_ram = crate::mm::pmm::PMM.total_memory() / (1024 * 1024);
            let used_ram = crate::mm::pmm::PMM.used_memory() / (1024 * 1024);
            let uptime = crate::drivers::timer::uptime_ms() / 1000;
            let fb_w = crate::framebuffer::FRAMEBUFFER.width;
            let fb_h = crate::framebuffer::FRAMEBUFFER.height;

            kprintln!("   __      __                        User@VeyronOS");
            kprintln!("   \\ \\    / /__ _   _ _ __ ___  _ __  -------------");
            kprintln!("    \\ \\  / / _ \\ | | | '__/ _ \\| '_ \\ OS: Veyron OS 11 (x86_64)");
            kprintln!("     \\ \\/ /  __/ |_| | | | (_) | | | | Host: QEMU / KVM Virtual Machine");
            kprintln!("      \\__/ \\___|\\__, |_|  \\___/|_| |_| Kernel: 0.1.0-scratch");
            kprintln!("                 |___/                Uptime: {} seconds", uptime);
            kprintln!("                                      Shell: Veyron CMD 1.0");
            kprintln!("                                      Resolution: {}x{} 32bpp", fb_w, fb_h);
            kprintln!("                                      Theme: Windows 11 Dark Mica");
            kprintln!("                                      Memory: {} MB / {} MB", used_ram, total_ram);
            kprintln!();
        }
    }
}

pub static mut TERMINAL: Terminal = Terminal::new();
