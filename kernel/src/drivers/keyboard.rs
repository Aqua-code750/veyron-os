//! PS/2 Keyboard Driver
//! Decodes Scan Code Set 1 with modifier tracking (Shift, Ctrl, Alt, CapsLock, Super key).

use crate::serial::inb;

const KEY_DATA_PORT: u16 = 0x60;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyCode {
    None,
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Space,
    Up,
    Down,
    Left,
    Right,
    Super, // Windows Key
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    CapsLock,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

#[derive(Debug, Copy, Clone)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub pressed: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

const QUEUE_SIZE: usize = 128;
static mut EVENT_QUEUE: [Option<KeyEvent>; QUEUE_SIZE] = [None; QUEUE_SIZE];
static mut QUEUE_HEAD: usize = 0;
static mut QUEUE_TAIL: usize = 0;

static mut SHIFT_DOWN: bool = false;
static mut CTRL_DOWN: bool = false;
static mut ALT_DOWN: bool = false;
static mut SUPER_DOWN: bool = false;
static mut CAPS_LOCK: bool = false;
static mut EXTENDED_PREFIX: bool = false;

pub fn init() {
    unsafe {
        // Unmask IRQ1 on PIC
        crate::arch::x86_64::pic::clear_mask(1);
    }
    crate::serial_println!("[KBD] PS/2 Keyboard driver initialized.");
}

pub fn on_keyboard_irq() {
    unsafe {
        let scancode = inb(KEY_DATA_PORT);
        if scancode == 0xE0 {
            EXTENDED_PREFIX = true;
            return;
        }

        let is_extended = EXTENDED_PREFIX;
        EXTENDED_PREFIX = false;

        let pressed = (scancode & 0x80) == 0;
        let code = scancode & 0x7F;

        let key = if is_extended {
            match code {
                0x48 => KeyCode::Up,
                0x50 => KeyCode::Down,
                0x4B => KeyCode::Left,
                0x4D => KeyCode::Right,
                0x5B => KeyCode::Super, // Left Super / Windows Key
                0x5C => KeyCode::Super, // Right Super
                0x1D => KeyCode::RightCtrl,
                0x38 => KeyCode::RightAlt,
                _ => KeyCode::None,
            }
        } else {
            match code {
                0x01 => KeyCode::Escape,
                0x0E => KeyCode::Backspace,
                0x0F => KeyCode::Tab,
                0x1C => KeyCode::Enter,
                0x39 => KeyCode::Space,
                0x2A => KeyCode::LeftShift,
                0x36 => KeyCode::RightShift,
                0x1D => KeyCode::LeftCtrl,
                0x38 => KeyCode::LeftAlt,
                0x3A => KeyCode::CapsLock,
                0x3B..=0x44 => match code {
                    0x3B => KeyCode::F1,
                    0x3C => KeyCode::F2,
                    0x3D => KeyCode::F3,
                    0x3E => KeyCode::F4,
                    0x3F => KeyCode::F5,
                    0x40 => KeyCode::F6,
                    0x41 => KeyCode::F7,
                    0x42 => KeyCode::F8,
                    0x43 => KeyCode::F9,
                    0x44 => KeyCode::F10,
                    _ => KeyCode::None,
                },
                0x57 => KeyCode::F11,
                0x58 => KeyCode::F12,
                _ => {
                    let ch = scancode_to_char(code, SHIFT_DOWN ^ CAPS_LOCK);
                    if let Some(c) = ch {
                        KeyCode::Char(c)
                    } else {
                        KeyCode::None
                    }
                }
            }
        };

        match key {
            KeyCode::LeftShift | KeyCode::RightShift => SHIFT_DOWN = pressed,
            KeyCode::LeftCtrl | KeyCode::RightCtrl => CTRL_DOWN = pressed,
            KeyCode::LeftAlt | KeyCode::RightAlt => ALT_DOWN = pressed,
            KeyCode::Super => SUPER_DOWN = pressed,
            KeyCode::CapsLock if pressed => CAPS_LOCK = !CAPS_LOCK,
            _ => {}
        }

        if key != KeyCode::None {
            let event = KeyEvent {
                key,
                pressed,
                shift: SHIFT_DOWN,
                ctrl: CTRL_DOWN,
                alt: ALT_DOWN,
                super_key: SUPER_DOWN,
            };
            push_event(event);
        }
    }
}

fn scancode_to_char(scancode: u8, shifted: bool) -> Option<char> {
    let unshifted = match scancode {
        0x02 => '1', 0x03 => '2', 0x04 => '3', 0x05 => '4', 0x06 => '5',
        0x07 => '6', 0x08 => '7', 0x09 => '8', 0x0A => '9', 0x0B => '0',
        0x0C => '-', 0x0D => '=',
        0x10 => 'q', 0x11 => 'w', 0x12 => 'e', 0x13 => 'r', 0x14 => 't',
        0x15 => 'y', 0x16 => 'u', 0x17 => 'i', 0x18 => 'o', 0x19 => 'p',
        0x1A => '[', 0x1B => ']',
        0x1E => 'a', 0x1F => 's', 0x20 => 'd', 0x21 => 'f', 0x22 => 'g',
        0x23 => 'h', 0x24 => 'j', 0x25 => 'k', 0x26 => 'l', 0x27 => ';',
        0x28 => '\'', 0x29 => '`',
        0x2B => '\\',
        0x2C => 'z', 0x2D => 'x', 0x2E => 'c', 0x2F => 'v', 0x30 => 'b',
        0x31 => 'n', 0x32 => 'm', 0x33 => ',', 0x34 => '.', 0x35 => '/',
        _ => return None,
    };

    if shifted {
        let ch = match unshifted {
            '1' => '!', '2' => '@', '3' => '#', '4' => '$', '5' => '%',
            '6' => '^', '7' => '&', '8' => '*', '9' => '(', '0' => ')',
            '-' => '_', '=' => '+',
            '[' => '{', ']' => '}',
            ';' => ':', '\'' => '"', '`' => '~',
            '\\' => '|',
            ',' => '<', '.' => '>', '/' => '?',
            c if c.is_ascii_lowercase() => c.to_ascii_uppercase(),
            c => c,
        };
        Some(ch)
    } else {
        Some(unshifted)
    }
}

fn push_event(event: KeyEvent) {
    unsafe {
        let next_head = (QUEUE_HEAD + 1) % QUEUE_SIZE;
        if next_head != QUEUE_TAIL {
            EVENT_QUEUE[QUEUE_HEAD] = Some(event);
            QUEUE_HEAD = next_head;
        }
    }
}

pub fn pop_event() -> Option<KeyEvent> {
    unsafe {
        if QUEUE_HEAD == QUEUE_TAIL {
            None
        } else {
            let event = EVENT_QUEUE[QUEUE_TAIL];
            QUEUE_TAIL = (QUEUE_TAIL + 1) % QUEUE_SIZE;
            event
        }
    }
}
