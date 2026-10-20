//! Framebuffer and Graphics Console for Veyron OS
//! Manages linear 32bpp framebuffer, text rendering, and console scrolling.

use crate::font::{get_glyph, FONT_HEIGHT, FONT_WIDTH};
use core::fmt;

pub struct Framebuffer {
    pub ptr: *mut u32,
    pub width: usize,
    pub height: usize,
    pub pitch: usize, // in pixels (pitch_bytes / 4)
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

impl Framebuffer {
    pub const fn empty() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
            width: 0,
            height: 0,
            pitch: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.ptr.is_null() && self.width > 0 && self.height > 0
    }

    #[inline]
    pub fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            unsafe {
                *self.ptr.add(y * self.pitch + x) = color;
            }
        }
    }

    pub fn clear(&self, color: u32) {
        self.fill_rect(0, 0, self.width, self.height, color);
    }

    pub fn fill_rect(&self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for row in y..y_end {
            let offset = row * self.pitch;
            for col in x..x_end {
                unsafe {
                    *self.ptr.add(offset + col) = color;
                }
            }
        }
    }

    pub fn draw_char(&self, x: usize, y: usize, c: char, fg: u32, bg: u32) {
        let glyph = get_glyph(c);
        for row in 0..FONT_HEIGHT {
            let py = y + row;
            if py >= self.height {
                break;
            }
            let byte = glyph[row];
            for col in 0..FONT_WIDTH {
                let px = x + col;
                if px >= self.width {
                    break;
                }
                let color = if (byte & (0x80 >> col)) != 0 {
                    fg
                } else {
                    bg
                };
                if color != 0xFF00_0000 {
                    // non-transparent
                    self.put_pixel(px, py, color);
                }
            }
        }
    }

    pub fn draw_string(&self, mut x: usize, y: usize, s: &str, fg: u32, bg: u32) {
        for c in s.chars() {
            if c == '\n' {
                break;
            }
            self.draw_char(x, y, c, fg, bg);
            x += FONT_WIDTH;
        }
    }

    pub fn copy_rect(&self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize, w: usize, h: usize) {
        for row in 0..h {
            let sy = src_y + row;
            let dy = dst_y + row;
            if sy < self.height && dy < self.height {
                for col in 0..w {
                    let sx = src_x + col;
                    let dx = dst_x + col;
                    if sx < self.width && dx < self.width {
                        unsafe {
                            let val = *self.ptr.add(sy * self.pitch + sx);
                            *self.ptr.add(dy * self.pitch + dx) = val;
                        }
                    }
                }
            }
        }
    }
}

pub static mut FRAMEBUFFER: Framebuffer = Framebuffer::empty();

pub struct Console {
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub fg_color: u32,
    pub bg_color: u32,
    pub margin_x: usize,
    pub margin_y: usize,
}

impl Console {
    pub const fn new() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            fg_color: 0x00F8FAFC, // Soft white
            bg_color: 0x000F172A, // Slate 900
            margin_x: 24,
            margin_y: 24,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            if FRAMEBUFFER.is_valid() {
                FRAMEBUFFER.clear(self.bg_color);
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn write_char(&mut self, c: char) {
        unsafe {
            if !FRAMEBUFFER.is_valid() {
                return;
            }

            let max_cols = (FRAMEBUFFER.width - self.margin_x * 2) / FONT_WIDTH;
            let max_rows = (FRAMEBUFFER.height - self.margin_y * 2) / FONT_HEIGHT;

            match c {
                '\n' => {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }
                '\r' => {
                    self.cursor_x = 0;
                }
                '\t' => {
                    self.cursor_x = (self.cursor_x + 4) & !3;
                    if self.cursor_x >= max_cols {
                        self.cursor_x = 0;
                        self.cursor_y += 1;
                    }
                }
                _ => {
                    if self.cursor_x >= max_cols {
                        self.cursor_x = 0;
                        self.cursor_y += 1;
                    }
                    if self.cursor_y >= max_rows {
                        self.scroll();
                    }

                    let px = self.margin_x + self.cursor_x * FONT_WIDTH;
                    let py = self.margin_y + self.cursor_y * FONT_HEIGHT;
                    FRAMEBUFFER.draw_char(px, py, c, self.fg_color, self.bg_color);
                    self.cursor_x += 1;
                }
            }

            if self.cursor_y >= max_rows {
                self.scroll();
            }
        }
    }

    pub fn scroll(&mut self) {
        unsafe {
            if !FRAMEBUFFER.is_valid() {
                return;
            }
            let line_height = FONT_HEIGHT;
            let area_w = FRAMEBUFFER.width - self.margin_x * 2;
            let area_h = (FRAMEBUFFER.height - self.margin_y * 2) / FONT_HEIGHT * FONT_HEIGHT;
            
            // Move up by 1 line
            FRAMEBUFFER.copy_rect(
                self.margin_x,
                self.margin_y + line_height,
                self.margin_x,
                self.margin_y,
                area_w,
                area_h - line_height,
            );

            // Clear bottom line
            FRAMEBUFFER.fill_rect(
                self.margin_x,
                self.margin_y + area_h - line_height,
                area_w,
                line_height,
                self.bg_color,
            );

            self.cursor_y = (FRAMEBUFFER.height - self.margin_y * 2) / FONT_HEIGHT - 1;
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Console::write_str(self, s);
        Ok(())
    }
}

pub static mut CONSOLE: Console = Console::new();

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::serial_print!($($arg)*);
        #[allow(unused_unsafe)]
        unsafe {
            let _ = write!($crate::framebuffer::CONSOLE, $($arg)*);
        }
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => {{
        $crate::kprint!($($arg)*);
        $crate::kprint!("\n");
    }};
}
