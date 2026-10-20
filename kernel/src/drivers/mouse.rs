//! PS/2 Mouse Driver
//! Tracks absolute mouse coordinates, button states (Left, Right, Middle), and movement deltas.

use crate::serial::{inb, outb};

const PS2_DATA_PORT: u16    = 0x60;
const PS2_STATUS_PORT: u16  = 0x64;
const PS2_COMMAND_PORT: u16 = 0x64;

#[derive(Debug, Copy, Clone)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

static mut MOUSE: MouseState = MouseState {
    x: 400,
    y: 300,
    left_button: false,
    right_button: false,
    middle_button: false,
};

static mut PACKET: [u8; 3] = [0; 3];
static mut PACKET_INDEX: usize = 0;
static mut SCREEN_WIDTH: i32 = 1280;
static mut SCREEN_HEIGHT: i32 = 800;

pub fn init(width: usize, height: usize) {
    unsafe {
        SCREEN_WIDTH = width as i32;
        SCREEN_HEIGHT = height as i32;
        MOUSE.x = SCREEN_WIDTH / 2;
        MOUSE.y = SCREEN_HEIGHT / 2;

        // 1. Enable auxiliary mouse device
        mouse_wait_write();
        outb(PS2_COMMAND_PORT, 0xA8);

        // 2. Enable mouse interrupts in controller config
        mouse_wait_write();
        outb(PS2_COMMAND_PORT, 0x20); // Read Command Byte
        mouse_wait_read();
        let status = inb(PS2_DATA_PORT) | 2; // Enable IRQ12
        mouse_wait_write();
        outb(PS2_COMMAND_PORT, 0x60); // Write Command Byte
        mouse_wait_write();
        outb(PS2_DATA_PORT, status);

        // 3. Set mouse to default settings
        mouse_write(0xF6);
        let _ = mouse_read(); // ACK (0xFA)

        // 4. Enable data streaming
        mouse_write(0xF4);
        let _ = mouse_read(); // ACK (0xFA)

        // 5. Unmask IRQ12 (and IRQ2 cascade line) on PIC
        crate::arch::x86_64::pic::clear_mask(2);  // Cascade IRQ
        crate::arch::x86_64::pic::clear_mask(12); // Mouse IRQ

        crate::serial_println!("[MOUSE] PS/2 Mouse initialized (screen bounds: {}x{}).", width, height);
    }
}

pub fn on_mouse_irq() {
    unsafe {
        let status = inb(PS2_STATUS_PORT);
        if (status & 0x01) != 0 && (status & 0x20) != 0 {
            let data = inb(PS2_DATA_PORT);
            PACKET[PACKET_INDEX] = data;
            PACKET_INDEX += 1;

            if PACKET_INDEX == 3 {
                PACKET_INDEX = 0;

                let flags = PACKET[0];
                let mut dx = PACKET[1] as i16;
                let mut dy = PACKET[2] as i16;

                // Sign extension for 9-bit relative coordinates
                if (flags & 0x10) != 0 {
                    dx |= !0xFF;
                }
                if (flags & 0x20) != 0 {
                    dy |= !0xFF;
                }

                // Check for overflow bits (discard packet if overflowed)
                if (flags & 0xC0) == 0 {
                    MOUSE.x = (MOUSE.x + dx as i32).clamp(0, SCREEN_WIDTH - 1);
                    MOUSE.y = (MOUSE.y - dy as i32).clamp(0, SCREEN_HEIGHT - 1); // dy is inverted on PS/2

                    MOUSE.left_button = (flags & 0x01) != 0;
                    MOUSE.right_button = (flags & 0x02) != 0;
                    MOUSE.middle_button = (flags & 0x04) != 0;
                }
            }
        }
    }
}

pub fn get_state() -> MouseState {
    unsafe { MOUSE }
}

unsafe fn mouse_wait_write() {
    let mut timeout = 100_000;
    while (inb(PS2_STATUS_PORT) & 0x02) != 0 && timeout > 0 {
        timeout -= 1;
    }
}

unsafe fn mouse_wait_read() {
    let mut timeout = 100_000;
    while (inb(PS2_STATUS_PORT) & 0x01) == 0 && timeout > 0 {
        timeout -= 1;
    }
}

unsafe fn mouse_write(cmd: u8) {
    mouse_wait_write();
    outb(PS2_COMMAND_PORT, 0xD4); // Tell controller to route byte to mouse
    mouse_wait_write();
    outb(PS2_DATA_PORT, cmd);
}

unsafe fn mouse_read() -> u8 {
    mouse_wait_read();
    inb(PS2_DATA_PORT)
}
