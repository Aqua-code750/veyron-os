//! Modern Windows 11 Fluent Compositor & Desktop Window Server
//! Features authentic Windows 11 "Bloom" wallpaper, rounded corners,
//! translucent acrylic styling, centered taskbar, high-DPI Fluent icons,
//! smooth caption controls, and native desktop applications.

use crate::framebuffer::FRAMEBUFFER;
use crate::font::{get_glyph, FONT_HEIGHT, FONT_WIDTH};
use crate::drivers::mouse::{get_state, MouseState};
use crate::drivers::keyboard::{pop_event, KeyCode};
use super::theme::{CURRENT_THEME, Color};
use super::window::{Window, AppType, TITLEBAR_HEIGHT};
use super::icons::{draw_icon, AppIcon, ICON_SIZE};

const MAX_WINDOWS: usize = 16;
const TASKBAR_HEIGHT: usize = 50;
const MAX_SEARCH_LEN: usize = 32;

pub struct Compositor {
    pub backbuffer: *mut u32,
    pub wallpaper_buffer: *mut u32,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub windows: [Window; MAX_WINDOWS],
    pub window_count: usize,
    pub active_window_idx: Option<usize>,
    pub dragging_idx: Option<usize>,
    pub drag_offset_x: i32,
    pub drag_offset_y: i32,
    pub start_menu_open: bool,
    pub search_buf: [u8; MAX_SEARCH_LEN],
    pub search_len: usize,
    pub prev_mouse_left: bool,
    pub anim_tick: u32,
    pub initialized: bool,
}

impl Compositor {
    pub const fn new() -> Self {
        const EMPTY_WIN: Window = Window::empty();
        Self {
            backbuffer: core::ptr::null_mut(),
            wallpaper_buffer: core::ptr::null_mut(),
            width: 0,
            height: 0,
            pitch: 0,
            windows: [EMPTY_WIN; MAX_WINDOWS],
            window_count: 0,
            active_window_idx: None,
            dragging_idx: None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            start_menu_open: false,
            search_buf: [0; MAX_SEARCH_LEN],
            search_len: 0,
            prev_mouse_left: false,
            anim_tick: 0,
            initialized: false,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            if !FRAMEBUFFER.is_valid() {
                return;
            }
            self.width = FRAMEBUFFER.width;
            self.height = FRAMEBUFFER.height;
            self.pitch = FRAMEBUFFER.pitch;

            // Allocate backbuffer and wallpaper buffer from kernel heap
            let bytes_needed = self.width * self.height * 4;
            let layout = core::alloc::Layout::from_size_align(bytes_needed, 4096).unwrap();

            self.backbuffer = crate::mm::heap::KERNEL_HEAP.allocate(layout) as *mut u32;
            self.wallpaper_buffer = crate::mm::heap::KERNEL_HEAP.allocate(layout) as *mut u32;

            if self.backbuffer.is_null() || self.wallpaper_buffer.is_null() {
                crate::serial_println!("[COMPOSITOR] Failed to allocate graphics buffers!");
                return;
            }

            self.initialized = true;
            crate::serial_println!(
                "[COMPOSITOR] Modern Windows 11 Compositor initialized: {}x{} (2x {} MB buffers).",
                self.width,
                self.height,
                bytes_needed / (1024 * 1024)
            );
        }

        // Generate authentic Windows 11 "Bloom" wallpaper once into pre-rendered buffer
        self.generate_bloom_wallpaper();

        // Open initial modern desktop windows
        self.spawn_window("Terminal - Command Prompt", AppType::Terminal, 90, 70, 680, 430);
        self.spawn_window("Notepad - Welcome to Veyron OS", AppType::Notepad, 420, 150, 620, 420);
    }

    pub fn spawn_window(&mut self, title: &str, app: AppType, x: i32, y: i32, w: usize, h: usize) -> Option<usize> {
        if self.window_count < MAX_WINDOWS {
            let idx = self.window_count;
            self.windows[idx] = Window::empty();
            self.windows[idx].id = idx + 1;
            self.windows[idx].set_title(title);
            self.windows[idx].app_type = app;
            self.windows[idx].x = x;
            self.windows[idx].y = y;
            self.windows[idx].width = w;
            self.windows[idx].height = h;
            self.windows[idx].prev_x = x;
            self.windows[idx].prev_y = y;
            self.windows[idx].prev_w = w;
            self.windows[idx].prev_h = h;
            self.windows[idx].is_active = true;

            for i in 0..self.window_count {
                self.windows[i].is_active = false;
            }

            self.window_count += 1;
            self.active_window_idx = Some(idx);
            return Some(idx);
        }
        None
    }

    #[inline]
    pub fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height && !self.backbuffer.is_null() {
            unsafe {
                *self.backbuffer.add(y * self.pitch + x) = color;
            }
        }
    }

    #[inline]
    pub fn blend_pixel(&self, x: usize, y: usize, color: u32, alpha: u8) {
        if x < self.width && y < self.height && !self.backbuffer.is_null() {
            unsafe {
                let dst_ptr = self.backbuffer.add(y * self.pitch + x);
                let bg = *dst_ptr;

                let r_bg = (bg >> 16) & 0xFF;
                let g_bg = (bg >> 8) & 0xFF;
                let b_bg = bg & 0xFF;

                let r_fg = (color >> 16) & 0xFF;
                let g_fg = (color >> 8) & 0xFF;
                let b_fg = color & 0xFF;

                let a = alpha as u32;
                let inv = 255 - a;

                let r = ((r_fg * a + r_bg * inv) / 255) as u32;
                let g = ((g_fg * a + g_bg * inv) / 255) as u32;
                let b = ((b_fg * a + b_bg * inv) / 255) as u32;

                *dst_ptr = (r << 16) | (g << 8) | b;
            }
        }
    }

    pub fn fill_rect(&self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for row in y..y_end {
            let off = row * self.pitch;
            for col in x..x_end {
                unsafe {
                    *self.backbuffer.add(off + col) = color;
                }
            }
        }
    }

    pub fn fill_rounded_rect(&self, x: usize, y: usize, w: usize, h: usize, r: usize, color: u32) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        let r_i32 = r as i32;
        let r2 = r_i32 * r_i32;

        for row in y..y_end {
            let dy = row as i32;
            let off = row * self.pitch;

            for col in x..x_end {
                let dx = col as i32;

                // Check 4 rounded corners
                let mut in_corner = false;
                let mut dist_sq = 0;

                // Top-left
                if dx < (x as i32 + r_i32) && dy < (y as i32 + r_i32) {
                    let cx = x as i32 + r_i32;
                    let cy = y as i32 + r_i32;
                    dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
                    in_corner = true;
                }
                // Top-right
                else if dx >= (x_end as i32 - r_i32) && dy < (y as i32 + r_i32) {
                    let cx = x_end as i32 - r_i32 - 1;
                    let cy = y as i32 + r_i32;
                    dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
                    in_corner = true;
                }
                // Bottom-left
                else if dx < (x as i32 + r_i32) && dy >= (y_end as i32 - r_i32) {
                    let cx = x as i32 + r_i32;
                    let cy = y_end as i32 - r_i32 - 1;
                    dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
                    in_corner = true;
                }
                // Bottom-right
                else if dx >= (x_end as i32 - r_i32) && dy >= (y_end as i32 - r_i32) {
                    let cx = x_end as i32 - r_i32 - 1;
                    let cy = y_end as i32 - r_i32 - 1;
                    dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
                    in_corner = true;
                }

                if in_corner {
                    if dist_sq <= r2 {
                        unsafe { *self.backbuffer.add(off + col) = color; }
                    }
                } else {
                    unsafe { *self.backbuffer.add(off + col) = color; }
                }
            }
        }
    }

    pub fn draw_rounded_border(&self, x: usize, y: usize, w: usize, h: usize, r: usize, color: u32) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        let r_i32 = r as i32;
        let r2 = r_i32 * r_i32;
        let inner_r2 = (r_i32 - 1) * (r_i32 - 1);

        for row in y..y_end {
            let dy = row as i32;
            let off = row * self.pitch;

            for col in x..x_end {
                let dx = col as i32;

                let is_edge = dx == x as i32 || dx == (x_end as i32 - 1) || dy == y as i32 || dy == (y_end as i32 - 1);

                if is_edge {
                    // Check corners
                    let mut in_corner = false;
                    let mut dist_sq = 0;

                    if dx < (x as i32 + r_i32) && dy < (y as i32 + r_i32) {
                        let cx = x as i32 + r_i32;
                        let cy = y as i32 + r_i32;
                        dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
                        in_corner = true;
                    } else if dx >= (x_end as i32 - r_i32) && dy < (y as i32 + r_i32) {
                        let cx = x_end as i32 - r_i32 - 1;
                        let cy = y as i32 + r_i32;
                        dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
                        in_corner = true;
                    } else if dx < (x as i32 + r_i32) && dy >= (y_end as i32 - r_i32) {
                        let cx = x as i32 + r_i32;
                        let cy = y_end as i32 - r_i32 - 1;
                        dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
                        in_corner = true;
                    } else if dx >= (x_end as i32 - r_i32) && dy >= (y_end as i32 - r_i32) {
                        let cx = x_end as i32 - r_i32 - 1;
                        let cy = y_end as i32 - r_i32 - 1;
                        dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
                        in_corner = true;
                    }

                    if in_corner {
                        if dist_sq <= r2 && dist_sq >= inner_r2 {
                            unsafe { *self.backbuffer.add(off + col) = color; }
                        }
                    } else {
                        unsafe { *self.backbuffer.add(off + col) = color; }
                    }
                }
            }
        }
    }

    pub fn draw_string(&self, mut x: usize, y: usize, s: &str, fg: u32) {
        for c in s.chars() {
            if c == '\n' { break; }
            let glyph = get_glyph(c);
            for row in 0..FONT_HEIGHT {
                let py = y + row;
                if py >= self.height { break; }
                let byte = glyph[row];
                for col in 0..FONT_WIDTH {
                    let px = x + col;
                    if px >= self.width { break; }
                    if (byte & (0x80 >> col)) != 0 {
                        self.put_pixel(px, py, fg);
                    }
                }
            }
            x += FONT_WIDTH;
        }
    }

#[inline]
fn isqrt(val: u32) -> u32 {
    if val <= 1 { return val; }
    let mut x0 = val / 2;
    let mut x1 = (x0 + val / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + val / x0) / 2;
    }
    x0
}
    fn generate_bloom_wallpaper(&mut self) {
        if self.wallpaper_buffer.is_null() { return; }

        let w = self.width;
        let h = self.height;
        let cx = (w / 2) as i32;
        let cy = ((h / 2) - 30) as i32;

        unsafe {
            for y in 0..h {
                let dy = y as i32 - cy;
                let row_off = y * self.pitch;

                for x in 0..w {
                    let dx = x as i32 - cx;

                    // 1. Deep Midnight Sapphire Base Gradient
                    let dist_center = ((dx * dx + dy * dy * 2).max(0) as u32).isqrt() as i32;
                    let base_r = (10 + (dist_center / 50)).min(20) as u32;
                    let base_g = (16 + (dist_center / 35)).min(28) as u32;
                    let base_b = (36 + (dist_center / 20)).min(55) as u32;

                    let mut r = base_r;
                    let mut g = base_g;
                    let mut b = base_b;

                    // 2. Central Cyan Glow Aura
                    let glow_dist = (dx * dx + dy * dy) as i32;
                    if glow_dist < 120_000 {
                        let factor = (120_000 - glow_dist) / 1000;
                        r = (r + (factor as u32 * 0 / 120)).min(255);
                        g = (g + (factor as u32 * 75 / 120)).min(255);
                        b = (b + (factor as u32 * 160 / 120)).min(255);
                    }

                    // 3. Mathematical Ribbons of the Windows 11 "Bloom" Silk Petals
                    // Petal 1: Central looping fold (Cyan & Azure silk)
                    let p1_x = (dx * 10 - dy * 6) / 8;
                    let p1_y = (dx * 6 + dy * 10) / 8;
                    let p1_rad = p1_x * p1_x + p1_y * p1_y * 2;
                    if p1_rad < 35_000 && p1_rad > 3_000 {
                        let silk_glow = ((35_000 - p1_rad) / 250) as u32;
                        r = (r + silk_glow / 6).min(255);
                        g = (g + silk_glow * 2 / 3).min(255);
                        b = (b + silk_glow).min(255);
                    }

                    // Petal 2: Left curving ribbon (Electric Blue & Indigo)
                    let p2_x = dx + 70;
                    let p2_y = dy + 20;
                    let p2_curve = p2_x * p2_x / 2 + p2_y * p2_y * 3;
                    if p2_curve < 45_000 && p2_curve > 5_000 {
                        let silk_blue = ((45_000 - p2_curve) / 300) as u32;
                        r = (r + silk_blue / 4).min(255);
                        g = (g + silk_blue / 2).min(255);
                        b = (b + silk_blue).min(255);
                    }

                    // Petal 3: Right upward wave (Lavender & Royal Blue)
                    let p3_x = dx - 65;
                    let p3_y = dy - 15;
                    let p3_curve = p3_x * p3_x * 2 + p3_y * p3_y;
                    if p3_curve < 40_000 && p3_curve > 4_000 {
                        let silk_lav = ((40_000 - p3_curve) / 320) as u32;
                        r = (r + silk_lav / 2).min(255);
                        g = (g + silk_lav / 3).min(255);
                        b = (b + silk_lav).min(255);
                    }

                    // Petal 4: Top-center crisp specular crest highlight
                    let crest_dist = dx * dx + (dy + 50) * (dy + 50) * 3;
                    if crest_dist < 10_000 {
                        let spec = ((10_000 - crest_dist) / 100) as u32;
                        r = (r + spec / 2).min(255);
                        g = (g + spec * 3 / 4).min(255);
                        b = (b + spec).min(255);
                    }

                    let pixel = (r << 16) | (g << 8) | b;
                    *self.wallpaper_buffer.add(row_off + x) = pixel;
                }
            }
        }
    }

    pub fn render_frame(&mut self) {
        if !self.initialized || self.backbuffer.is_null() {
            return;
        }

        self.anim_tick = self.anim_tick.wrapping_add(1);
        let mouse = get_state();

        // 1. Blit pre-rendered Windows 11 Bloom wallpaper (instantaneous copy)
        self.draw_wallpaper();

        // 2. Draw Modern Desktop Icons (with Fluent 24x24 graphics)
        self.draw_desktop_icons();

        // 3. Draw Windows in z-order with rounded corners & Mica titlebars
        for i in 0..self.window_count {
            if !self.windows[i].is_minimized {
                self.draw_window(&self.windows[i], &mouse);
            }
        }

        // 4. Draw Centered Windows 11 Taskbar with active app indicator & system tray
        self.draw_taskbar(&mouse);

        // 5. Draw Start Menu if open
        if self.start_menu_open {
            self.draw_start_menu(&mouse);
        }

        // 6. Draw Modern High-DPI Mouse Cursor
        self.draw_cursor(mouse.x as usize, mouse.y as usize);

        // 7. Swap to Hardware GOP Framebuffer
        self.flip();

        // 8. Handle Input & Interactions
        self.handle_input(&mouse);
    }

    fn draw_wallpaper(&self) {
        if !self.wallpaper_buffer.is_null() && !self.backbuffer.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.wallpaper_buffer,
                    self.backbuffer,
                    self.width * self.height,
                );
            }
        }
    }

    fn draw_desktop_icons(&self) {
        let icons = [
            ("This PC", AppIcon::Explorer),
            ("Terminal", AppIcon::Terminal),
            ("Notepad", AppIcon::Notepad),
            ("Paint", AppIcon::Paint),
            ("Browser", AppIcon::Browser),
            ("Calculator", AppIcon::Calculator),
            ("Settings", AppIcon::Settings),
            ("Doom", AppIcon::Doom),
        ];

        let start_x = 28;
        let mut start_y = 28;

        for (label, icon) in icons.iter() {
            // Draw modern 24x24 icon
            let mut put_fn = |ix, iy, c| {
                self.put_pixel(start_x + 16 + ix, start_y + iy, c);
            };
            draw_icon(0, 0, *icon, &mut put_fn);

            // Draw label with subtle drop shadow for readability on wallpaper
            self.draw_string(start_x + 6, start_y + 30, label, 0x050811);
            self.draw_string(start_x + 5, start_y + 29, label, 0xF8FAFC);

            start_y += 76;
        }
    }

    fn draw_window(&self, win: &Window, mouse: &MouseState) {
        let wx = win.x.max(0) as usize;
        let wy = win.y.max(0) as usize;
        let ww = win.width;
        let wh = win.height;
        let corner_r = 10;

        // 1. Multi-layered Soft Drop Shadow (expanding rounded aura)
        for s in (1..=3).rev() {
            let offset = s * 3;
            let alpha = (40 / s) as u8;
            self.fill_rounded_rect(wx + offset, wy + offset, ww, wh, corner_r + 2, 0x020408);
        }

        // 2. Window Frame Surface with Rounded Corners
        let frame_bg = if win.is_active { 0x1E2430 } else { 0x161B24 };
        self.fill_rounded_rect(wx, wy, ww, wh, corner_r, frame_bg);

        // 3. Subtle 1px Mica Border
        let border_col = if win.is_active { 0x0078D4 } else { 0x334155 };
        self.draw_rounded_border(wx, wy, ww, wh, corner_r, border_col);

        // 4. Modern Titlebar
        let titlebar_h = TITLEBAR_HEIGHT + 4; // 36px titlebar
        // Titlebar App Icon (24x24 scaled down to 18px or nicely aligned)
        let mut put_fn = |ix, iy, c| {
            if ix < 20 && iy < 20 {
                self.put_pixel(wx + 10 + ix, wy + 8 + iy, c);
            }
        };
        draw_icon(0, 0, win.app_type.icon(), &mut put_fn);

        // Titlebar Text
        let title_color = if win.is_active { 0xF8FAFC } else { 0x94A3B8 };
        self.draw_string(wx + 38, wy + 11, win.title_as_str(), title_color);

        // Modern Caption Buttons: Minimize (—), Maximize (▢), Close (✕)
        let right = wx + ww;
        let mx = mouse.x as usize;
        let my = mouse.y as usize;

        // Minimize Button (46px zone)
        let min_hover = mx >= (right - 138) && mx < (right - 92) && my >= wy && my < (wy + titlebar_h);
        if min_hover {
            self.fill_rect(right - 138, wy + 2, 46, titlebar_h - 2, 0x2A354A);
        }
        // Draw 10px centered line
        self.fill_rect(right - 118, wy + 18, 10, 1, 0x94A3B8);

        // Maximize Button (46px zone)
        let max_hover = mx >= (right - 92) && mx < (right - 46) && my >= wy && my < (wy + titlebar_h);
        if max_hover {
            self.fill_rect(right - 92, wy + 2, 46, titlebar_h - 2, 0x2A354A);
        }
        // Draw 10x10 square outline
        self.draw_hollow_rect(right - 74, wy + 13, 10, 10, 0x94A3B8);

        // Close Button (46px zone with Windows 11 Crimson hover)
        let close_hover = mx >= (right - 46) && mx < right && my >= wy && my < (wy + titlebar_h);
        let close_bg = if close_hover { 0xE81123 } else { frame_bg };
        if close_hover {
            self.fill_rect(right - 46, wy + 2, 44, titlebar_h - 2, close_bg);
        }
        // Draw centered 'X' cross
        let cross_col = if close_hover { 0xFFFFFF } else { 0x94A3B8 };
        let cx = right - 24;
        let cy = wy + 18;
        for i in 0..5 {
            self.put_pixel(cx - 4 + i * 2, cy - 4 + i * 2, cross_col);
            self.put_pixel(cx + 4 - i * 2, cy - 4 + i * 2, cross_col);
        }

        // 5. Window Client Area (Dark Obsidian / Zinc 950)
        let client_y = wy + titlebar_h;
        let client_h = wh.saturating_sub(titlebar_h + 4);
        let client_bg = 0x12151D;
        self.fill_rect(wx + 2, client_y, ww - 4, client_h, client_bg);

        // 6. Render Native Application Contents
        self.render_app_content(win, wx + 2, client_y, ww - 4, client_h);
    }

    fn draw_hollow_rect(&self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        self.fill_rect(x, y, w, 1, color);
        self.fill_rect(x, y + h - 1, w, 1, color);
        self.fill_rect(x, y, 1, h, color);
        self.fill_rect(x + w - 1, y, 1, h, color);
    }

    fn render_app_content(&self, win: &Window, cx: usize, cy: usize, cw: usize, ch: usize) {
        match win.app_type {
            AppType::Terminal => {
                // Windows Terminal style: top tab bar + acrylic dark console
                self.fill_rect(cx, cy, cw, 26, 0x18181F);
                self.draw_string(cx + 12, cy + 6, "Command Prompt  x     +", 0x94A3B8);
                self.fill_rect(cx, cy + 26, cw, 1, 0x27272A);

                let start_y = cy + 34;
                self.draw_string(cx + 12, start_y, "Veyron OS [Version 11.0.2026.1]", 0xF8FAFC);
                self.draw_string(cx + 12, start_y + 18, "(c) 2026 Veyron Project. All rights reserved.", 0x64748B);

                self.draw_string(cx + 12, start_y + 44, "C:\\Users\\Default>", 0x06B6D4);
                self.draw_string(cx + 160, start_y + 44, "neofetch", 0xF8FAFC);

                // Neofetch card
                let neo_y = start_y + 68;
                let logo_lines = [
                    "    __     __",
                    "    \\ \\   / /__ _   _ _ __ ___  _ __",
                    "     \\ \\ / / _ \\ | | | '__/ _ \\| '_ \\",
                    "      \\ V /  __/ |_| | | | (_) | | | |",
                    "       \\_/ \\___|\\__, |_|  \\___/|_| |_|",
                    "                |___/",
                ];
                for (li, line) in logo_lines.iter().enumerate() {
                    self.draw_string(cx + 16, neo_y + li * 16, line, 0x00A2ED);
                }

                let info_lines = [
                    ("OS:", "Veyron OS 11 Pro x86_64"),
                    ("Host:", "QEMU Virtual Machine (Q35)"),
                    ("Kernel:", "0.1.0-scratch (Rust no_std)"),
                    ("Uptime:", "12 mins"),
                    ("Shell:", "Veyron CLI 1.0"),
                    ("Resolution:", "1280x800 60Hz"),
                    ("Memory:", "48 MB / 2048 MB"),
                ];
                for (ii, (k, v)) in info_lines.iter().enumerate() {
                    self.draw_string(cx + 340, neo_y + ii * 16, k, 0x38BDF8);
                    self.draw_string(cx + 430, neo_y + ii * 16, v, 0xF8FAFC);
                }

                // Blinking terminal cursor
                self.draw_string(cx + 12, neo_y + 120, "C:\\Users\\Default> ", 0x06B6D4);
                if (self.anim_tick / 30) % 2 == 0 {
                    self.fill_rect(cx + 160, neo_y + 120, 8, 16, 0x22D3EE);
                }
            }

            AppType::Notepad => {
                // Fluent Notepad Ribbon: File  Edit  View
                self.fill_rect(cx, cy, cw, 28, 0x1E1E24);
                self.draw_string(cx + 14, cy + 7, "File    Edit    View", 0x94A3B8);
                self.fill_rect(cx, cy + 28, cw, 1, 0x27272A);

                // Text body
                let ty = cy + 38;
                self.draw_string(cx + 20, ty, "Welcome to Veyron OS 11!", 0xF8FAFC);
                self.draw_string(cx + 20, ty + 24, "A modern desktop operating system engineered from scratch in Rust.", 0x94A3B8);
                self.draw_string(cx + 20, ty + 48, "Features:", 0x38BDF8);
                self.draw_string(cx + 36, ty + 70, "- UEFI x86_64 Limine Bootloader & Higher-Half Kernel", 0xF8FAFC);
                self.draw_string(cx + 36, ty + 92, "- High-speed AHCI SATA Controller & Persistent FAT32 VFS", 0xF8FAFC);
                self.draw_string(cx + 36, ty + 114, "- Preemptive Multitasking Scheduler & User Ring 3 Mode", 0xF8FAFC);
                self.draw_string(cx + 36, ty + 136, "- Native TCP/IP Network Stack (DHCP, DNS, HTTP/1.1)", 0xF8FAFC);
                self.draw_string(cx + 36, ty + 158, "- Windows 11 Fluent Compositor with Acrylic & Bloom graphics", 0xF8FAFC);

                // Status bar at bottom
                let sby = cy + ch - 24;
                self.fill_rect(cx, sby, cw, 24, 0x18181E);
                self.fill_rect(cx, sby, cw, 1, 0x27272A);
                self.draw_string(cx + 16, sby + 5, "Ln 1, Col 1     100%     Windows (CRLF)     UTF-8", 0x64748B);
            }

            AppType::Explorer => {
                // Command bar: + New | Cut | Copy | Delete
                self.fill_rect(cx, cy, cw, 34, 0x1E2430);
                self.fill_rounded_rect(cx + 10, cy + 5, 75, 24, 4, 0x0078D4);
                self.draw_string(cx + 18, cy + 9, "+ New", 0xF8FAFC);
                self.draw_string(cx + 100, cy + 9, "Cut    Copy    Rename    Delete", 0x94A3B8);
                self.fill_rect(cx, cy + 34, cw, 1, 0x2A354A);

                // Path bar
                self.fill_rect(cx, cy + 35, cw, 28, 0x181E28);
                self.draw_string(cx + 16, cy + 41, "This PC  >  Local Disk (C:)  >  Users  >  Default", 0xF8FAFC);
                self.fill_rect(cx, cy + 63, cw, 1, 0x2A354A);

                // Sidebar
                let sb_w = 160;
                self.fill_rect(cx, cy + 64, sb_w, ch - 64, 0x141A24);
                self.draw_string(cx + 12, cy + 76, "Quick access", 0x00A2ED);
                self.draw_string(cx + 16, cy + 100, "* Desktop", 0xF8FAFC);
                self.draw_string(cx + 16, cy + 124, "* Documents", 0xF8FAFC);
                self.draw_string(cx + 16, cy + 148, "* Downloads", 0xF8FAFC);
                self.draw_string(cx + 16, cy + 172, "* Pictures", 0xF8FAFC);
                self.draw_string(cx + 12, cy + 204, "This PC", 0x00A2ED);
                self.draw_string(cx + 16, cy + 228, "Local Disk (C:)", 0xF8FAFC);

                // File items in main view
                let mx = cx + sb_w + 20;
                let files = [
                    ("Desktop", "File folder", "--"),
                    ("Documents", "File folder", "--"),
                    ("Downloads", "File folder", "--"),
                    ("Pictures", "File folder", "--"),
                    ("Welcome.txt", "Text Document", "142 bytes"),
                ];

                self.draw_string(mx, cy + 76, "Name                     Type                Size", 0x64748B);
                self.fill_rect(mx, cy + 94, cw - sb_w - 40, 1, 0x27272A);

                for (i, (fname, ftype, fsize)) in files.iter().enumerate() {
                    let fy = cy + 104 + i * 26;
                    let color = if fname.ends_with(".txt") { 0x38BDF8 } else { 0xFBBF24 };
                    self.draw_string(mx, fy, fname, color);
                    self.draw_string(mx + 180, fy, ftype, 0x94A3B8);
                    self.draw_string(mx + 320, fy, fsize, 0x94A3B8);
                }
            }

            AppType::Calculator => {
                // Modern Fluent Calculator
                self.fill_rect(cx + 16, cy + 16, cw - 32, 54, 0x1E2430);
                self.draw_string(cx + cw - 90, cy + 34, "2,026", 0xF8FAFC);

                let keys = [
                    ["CE", "C", "Del", "/"],
                    ["7", "8", "9", "*"],
                    ["4", "5", "6", "-"],
                    ["1", "2", "3", "+"],
                    ["+/-", "0", ".", "="],
                ];
                let mut by = cy + 86;
                for row in keys.iter() {
                    let mut bx = cx + 16;
                    for &k in row.iter() {
                        let btn_col = if k == "=" { 0x0078D4 } else { 0x2A354A };
                        self.fill_rounded_rect(bx, by, 54, 38, 4, btn_col);
                        self.draw_string(bx + 20, by + 12, k, 0xF8FAFC);
                        bx += 62;
                    }
                    by += 46;
                }
            }

            AppType::TaskManager => {
                // Tabs
                self.fill_rect(cx, cy, cw, 28, 0x1E2430);
                self.draw_string(cx + 16, cy + 7, "Processes       Performance       App history", 0xF8FAFC);
                self.fill_rect(cx, cy + 28, cw, 1, 0x2A354A);

                // Real-time animated CPU graph
                self.draw_string(cx + 16, cy + 38, "CPU Utilization (Dual-Core x86_64)", 0x22C55E);
                let graph_x = cx + 16;
                let graph_y = cy + 56;
                let graph_w = cw - 32;
                let graph_h = 70;
                self.fill_rect(graph_x, graph_y, graph_w, graph_h, 0x0F172A);
                self.draw_hollow_rect(graph_x, graph_y, graph_w, graph_h, 0x1E293B);

                // Draw live undulating wave
                for x in 0..graph_w {
                    let phase = ((x as u32 + self.anim_tick * 4) % 64) as i32;
                    let tri = if phase < 32 { phase - 16 } else { 48 - phase };
                    let h_offset = ((tri * tri) / 10 + 15) as usize;
                    let wy = (graph_y + graph_h).saturating_sub(h_offset.min(graph_h - 2));
                    self.put_pixel(graph_x + x, wy, 0x22C55E);
                    self.blend_pixel(graph_x + x, wy + 1, 0x15803D, 180);
                }

                // Process list table
                let ty = graph_y + graph_h + 16;
                self.draw_string(cx + 16, ty, "Name                   PID      Status      Memory     CPU", 0x64748B);
                self.fill_rect(cx + 16, ty + 16, cw - 32, 1, 0x27272A);

                let procs = [
                    ("veyron_compositor", "1", "Running", "14.2 MB", "2.1%"),
                    ("terminal.exe",      "2", "Running", "4.8 MB",  "0.1%"),
                    ("notepad.exe",       "3", "Running", "5.1 MB",  "0.0%"),
                    ("explorer.exe",      "4", "Running", "8.9 MB",  "0.2%"),
                    ("system_idle",       "0", "Ready",   "0.1 MB",  "97.6%"),
                ];

                for (i, p) in procs.iter().enumerate() {
                    let py = ty + 24 + i * 22;
                    let name_col = if i == 0 { 0x38BDF8 } else { 0xF8FAFC };
                    self.draw_string(cx + 16, py, p.0, name_col);
                    self.draw_string(cx + 180, py, p.1, 0x94A3B8);
                    self.draw_string(cx + 240, py, p.2, 0x22C55E);
                    self.draw_string(cx + 320, py, p.3, 0x94A3B8);
                    self.draw_string(cx + 400, py, p.4, 0x94A3B8);
                }
            }

            AppType::Paint => {
                // Modern Paint ribbon
                self.fill_rect(cx, cy, cw, 36, 0x1E2430);
                self.draw_string(cx + 14, cy + 10, "Tools: [Brush] [Pencil] [Eraser] [Shapes] | Colors: [● Red] [● Green] [● Blue]", 0xF8FAFC);
                // Canvas surface
                self.fill_rect(cx + 10, cy + 46, cw - 20, ch - 56, 0xFFFFFF);
                self.draw_string(cx + 24, cy + 64, "Veyron Paint Studio - High Resolution Canvas", 0x0078D4);
            }

            AppType::Settings => {
                // Windows 11 style Settings card interface
                let sb_w = 160;
                self.fill_rect(cx, cy, sb_w, ch, 0x141A24);
                self.draw_string(cx + 14, cy + 20, "Settings", 0xF8FAFC);
                self.draw_string(cx + 14, cy + 54, "* System", 0x00A2ED);
                self.draw_string(cx + 14, cy + 80, "* Personalization", 0x94A3B8);
                self.draw_string(cx + 14, cy + 106, "* Network", 0x94A3B8);
                self.draw_string(cx + 14, cy + 132, "* Accounts", 0x94A3B8);
                self.draw_string(cx + 14, cy + 158, "* About", 0x94A3B8);

                // Right Pane Cards
                let rx = cx + sb_w + 24;
                self.draw_string(rx, cy + 20, "System  >  About", 0xF8FAFC);

                // Card 1: Device specifications
                self.fill_rounded_rect(rx, cy + 50, cw - sb_w - 48, 120, 6, 0x1E2430);
                self.draw_string(rx + 16, cy + 64, "Device Name:     Veyron-PC", 0xF8FAFC);
                self.draw_string(rx + 16, cy + 84, "Processor:       x86_64 Dual-Core SMP", 0x94A3B8);
                self.draw_string(rx + 16, cy + 104, "Installed RAM:   2.00 GB", 0x94A3B8);
                self.draw_string(rx + 16, cy + 124, "System Type:     64-bit UEFI Native OS", 0x94A3B8);
                self.draw_string(rx + 16, cy + 144, "Graphics:        virtio-vga / GOP Framebuffer", 0x94A3B8);

                // Reset button
                self.fill_rounded_rect(rx, cy + 190, 160, 34, 4, 0xDC2626);
                self.draw_string(rx + 24, cy + 200, "Reset Veyron OS", 0xF8FAFC);
            }

            AppType::Browser => {
                // Tab bar
                self.fill_rect(cx, cy, cw, 30, 0x161B24);
                self.fill_rounded_rect(cx + 10, cy + 4, 160, 26, 4, 0x222B38);
                self.draw_string(cx + 18, cy + 9, "veyron://welcome  x", 0xF8FAFC);
                // Address bar
                self.fill_rect(cx, cy + 30, cw, 38, 0x1E2430);
                self.fill_rounded_rect(cx + 50, cy + 34, cw - 80, 28, 4, 0x12151D);
                self.draw_string(cx + 14, cy + 40, "<  >  @", 0x94A3B8);
                self.draw_string(cx + 64, cy + 40, "https://veyron.os/welcome", 0xF8FAFC);

                // Browser content
                self.fill_rect(cx, cy + 68, cw, ch - 68, 0x0A0F1D);
                self.draw_string(cx + 30, cy + 90, "Veyron Web Browser", 0x38BDF8);
                self.draw_string(cx + 30, cy + 120, "Powered by Veyron's native TCP/IP stack & BearSSL.", 0xF8FAFC);
                self.draw_string(cx + 30, cy + 144, "Render Engine: HTML5 / CSS3 Subset Renderer", 0x94A3B8);
            }

            AppType::Doom => {
                // DOOM Screen
                self.fill_rect(cx, cy, cw, ch, 0x000000);
                self.draw_string(cx + cw / 2 - 90, cy + ch / 2 - 20, "DOOM (wasm3 engine)", 0xDC2626);
                self.draw_string(cx + cw / 2 - 100, cy + ch / 2 + 10, "Press Space to Play WAD", 0xF8FAFC);
            }

            AppType::None => {}
        }
    }

    fn draw_taskbar(&self, mouse: &MouseState) {
        let ty = self.height - TASKBAR_HEIGHT;
        let tw = self.width;

        // Translucent Acrylic Dark Slate Taskbar (Mica style)
        self.fill_rect(0, ty, tw, TASKBAR_HEIGHT, 0x141A26);
        // 1px subtle top highlight line
        self.fill_rect(0, ty, tw, 1, 0x2D374D);

        // Centered App Dock
        let center_x = tw / 2;
        let apps = [
            AppIcon::Start,
            AppIcon::Explorer,
            AppIcon::Terminal,
            AppIcon::Notepad,
            AppIcon::Paint,
            AppIcon::Browser,
            AppIcon::Calculator,
            AppIcon::Settings,
        ];

        let icon_spacing = 46;
        let total_dock_w = apps.len() * icon_spacing;
        let start_dock_x = center_x - (total_dock_w / 2);

        for (i, icon) in apps.iter().enumerate() {
            let bx = start_dock_x + i * icon_spacing;

            // Hover rounded pill
            let is_hover = mouse.y >= ty as i32 && mouse.x >= bx as i32 && mouse.x < (bx + 40) as i32;
            if is_hover {
                self.fill_rounded_rect(bx, ty + 4, 40, 42, 6, 0x2A354A);
            }

            // Draw modern 24x24 Fluent icon centered
            let mut put_fn = |ix, iy, c| {
                self.put_pixel(bx + 8 + ix, ty + 10 + iy, c);
            };
            draw_icon(0, 0, *icon, &mut put_fn);

            // Active cyan pill indicator under running apps
            if i > 0 && i <= self.window_count {
                self.fill_rounded_rect(bx + 12, ty + 44, 16, 3, 2, 0x00A2ED);
            }
        }

        // Right-aligned System Tray
        let tray_x = tw - 160;
        self.fill_rounded_rect(tray_x - 10, ty + 8, 150, 34, 6, 0x1B2332);

        // Network icon
        let mut put_net = |ix, iy, c| { self.put_pixel(tray_x + ix, ty + 13 + iy, c); };
        draw_icon(0, 0, AppIcon::Network, &mut put_net);

        // Volume icon
        let mut put_vol = |ix, iy, c| { self.put_pixel(tray_x + 28 + ix, ty + 13 + iy, c); };
        draw_icon(0, 0, AppIcon::Volume, &mut put_vol);

        // Digital Clock
        let time = crate::drivers::rtc::read_rtc();
        let h12 = if time.hours == 0 { 12 } else if time.hours > 12 { time.hours - 12 } else { time.hours };
        let ampm = if time.hours >= 12 { "PM" } else { "AM" };
        let mut time_str = [b'0'; 8];
        time_str[0] = b'0' + (h12 / 10) as u8;
        time_str[1] = b'0' + (h12 % 10) as u8;
        time_str[2] = b':';
        time_str[3] = b'0' + (time.minutes / 10) as u8;
        time_str[4] = b'0' + (time.minutes % 10) as u8;
        time_str[5] = b' ';
        time_str[6] = ampm.as_bytes()[0];
        time_str[7] = ampm.as_bytes()[1];

        if let Ok(s) = core::str::from_utf8(&time_str) {
            self.draw_string(tray_x + 60, ty + 17, s, 0xF8FAFC);
        }

        // Extreme Right: Windows 11 "Show Desktop" line
        self.fill_rect(tw - 2, ty + 10, 1, 30, 0x334155);
    }

    fn draw_start_menu(&self, mouse: &MouseState) {
        let mw = 540;
        let mh = 500;
        let mx = (self.width - mw) / 2;
        let my = self.height - TASKBAR_HEIGHT - mh - 12;

        // Shadow & 12px Rounded Glass Frame
        self.fill_rounded_rect(mx + 4, my + 4, mw, mh, 12, 0x050811);
        self.fill_rounded_rect(mx, my, mw, mh, 12, 0x161D2A);
        self.draw_rounded_border(mx, my, mw, mh, 12, 0x2E3A52);

        // Search Bar (Pill shape)
        self.fill_rounded_rect(mx + 32, my + 24, mw - 64, 38, 8, 0x0E131E);
        self.draw_rounded_border(mx + 32, my + 24, mw - 64, 38, 8, 0x334155);
        self.draw_string(mx + 48, my + 35, "Search apps, settings, and files...", 0x64748B);

        // Pinned Header
        self.draw_string(mx + 36, my + 82, "Pinned", 0xF8FAFC);
        self.draw_string(mx + mw - 110, my + 82, "All apps >", 0x00A2ED);

        // Pinned Apps Grid (4 columns x 2 rows)
        let pinned = [
            ("Explorer", AppIcon::Explorer),
            ("Terminal", AppIcon::Terminal),
            ("Notepad", AppIcon::Notepad),
            ("Paint", AppIcon::Paint),
            ("Browser", AppIcon::Browser),
            ("Calculator", AppIcon::Calculator),
            ("Settings", AppIcon::Settings),
            ("Task Mgr", AppIcon::TaskManager),
        ];

        for (i, (label, icon)) in pinned.iter().enumerate() {
            let col = i % 4;
            let row = i / 4;
            let ax = mx + 42 + col * 118;
            let ay = my + 112 + row * 74;

            // Hover card
            let hov = mouse.x >= ax as i32 && mouse.x < (ax + 105) as i32
                && mouse.y >= ay as i32 && mouse.y < (ay + 68) as i32;
            if hov {
                self.fill_rounded_rect(ax, ay, 105, 68, 6, 0x222B3D);
            }

            let mut put_fn = |ix, iy, c| { self.put_pixel(ax + 40 + ix, ay + 10 + iy, c); };
            draw_icon(0, 0, *icon, &mut put_fn);
            self.draw_string(ax + 24, ay + 42, label, 0xF8FAFC);
        }

        // Recommended Section
        self.draw_string(mx + 36, my + 280, "Recommended", 0xF8FAFC);
        self.draw_string(mx + mw - 90, my + 280, "More >", 0x00A2ED);

        let rec_items = [
            ("Welcome.txt", "Yesterday at 4:20 PM"),
            ("Terminal - Session Logs", "2h ago"),
            ("Kernel Config (limine.conf)", "1d ago"),
        ];

        for (ri, (rtitle, rsub)) in rec_items.iter().enumerate() {
            let ry = my + 308 + ri * 36;
            self.draw_string(mx + 48, ry, rtitle, 0xF8FAFC);
            self.draw_string(mx + 48, ry + 16, rsub, 0x64748B);
        }

        // Bottom User Profile & Power Bar
        let p_y = my + mh - 56;
        self.fill_rounded_rect(mx, p_y, mw, 56, 10, 0x101520);
        self.fill_rect(mx, p_y, mw, 1, 0x232D3F);

        // Circular User Avatar
        self.fill_rounded_rect(mx + 28, p_y + 12, 32, 32, 16, 0x0078D4);
        self.draw_string(mx + 70, p_y + 20, "Veyron Administrator", 0xF8FAFC);

        // Power Button Icon on Right
        self.draw_string(mx + mw - 70, p_y + 20, "Power", 0xEF4444);
    }

    fn draw_cursor(&self, cx: usize, cy: usize) {
        // High-DPI Windows 11 style arrow cursor with crisp dark outline
        let cursor_shape: [&str; 19] = [
            "X                  ",
            "XX                 ",
            "X.X                ",
            "X..X               ",
            "X...X              ",
            "X....X             ",
            "X.....X            ",
            "X......X           ",
            "X.......X          ",
            "X........X         ",
            "X.....XXXXXX       ",
            "X..X..X            ",
            "X.X X..X           ",
            "XX   X..X          ",
            "X     X..X         ",
            "       X..X        ",
            "        X..X       ",
            "         XX        ",
            "                   ",
        ];

        for (r, row) in cursor_shape.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                let px = cx + c;
                let py = cy + r;
                if ch == 'X' {
                    self.put_pixel(px, py, 0x0A0F1D); // Dark border
                } else if ch == '.' {
                    self.put_pixel(px, py, 0xFFFFFF); // Bright white interior
                }
            }
        }
    }

    fn flip(&self) {
        unsafe {
            if FRAMEBUFFER.is_valid() && !self.backbuffer.is_null() {
                core::ptr::copy_nonoverlapping(
                    self.backbuffer,
                    FRAMEBUFFER.ptr,
                    self.width * self.height,
                );
            }
        }
    }

    fn handle_input(&mut self, mouse: &MouseState) {
        // Keyboard Shortcuts
        while let Some(event) = pop_event() {
            if event.pressed {
                match event.key {
                    KeyCode::Super => {
                        self.start_menu_open = !self.start_menu_open;
                    }
                    KeyCode::Escape => {
                        self.start_menu_open = false;
                    }
                    _ => {}
                }
            }
        }

        // Mouse Clicks
        let clicked = mouse.left_button && !self.prev_mouse_left;
        let released = !mouse.left_button && self.prev_mouse_left;
        self.prev_mouse_left = mouse.left_button;

        if clicked {
            let ty = (self.height - TASKBAR_HEIGHT) as i32;
            let center_x = (self.width / 2) as i32;
            let start_dock_x = center_x - (8 * 46 / 2);

            // Check Taskbar Start Button click
            if mouse.y >= ty && mouse.x >= start_dock_x && mouse.x < (start_dock_x + 46) {
                self.start_menu_open = !self.start_menu_open;
                return;
            }

            // Check Window Dragging / Clicking (top-to-bottom of z-order)
            let mut clicked_win: Option<usize> = None;
            for i in (0..self.window_count).rev() {
                let win = &self.windows[i];
                if win.contains(mouse.x, mouse.y) {
                    clicked_win = Some(i);
                    break;
                }
            }

            if let Some(idx) = clicked_win {
                self.active_window_idx = Some(idx);
                for i in 0..self.window_count {
                    self.windows[i].is_active = (i == idx);
                }

                let win = &mut self.windows[idx];
                if win.is_in_close_btn(mouse.x, mouse.y) {
                    win.is_minimized = true;
                } else if win.is_in_max_btn(mouse.x, mouse.y) {
                    if win.is_maximized {
                        win.x = win.prev_x;
                        win.y = win.prev_y;
                        win.width = win.prev_w;
                        win.height = win.prev_h;
                        win.is_maximized = false;
                    } else {
                        win.prev_x = win.x;
                        win.prev_y = win.y;
                        win.prev_w = win.width;
                        win.prev_h = win.height;
                        win.x = 0;
                        win.y = 0;
                        win.width = self.width;
                        win.height = self.height - TASKBAR_HEIGHT;
                        win.is_maximized = true;
                    }
                } else if win.is_in_titlebar(mouse.x, mouse.y) {
                    self.dragging_idx = Some(idx);
                    self.drag_offset_x = mouse.x - win.x;
                    self.drag_offset_y = mouse.y - win.y;
                }
            }
        }

        if mouse.left_button {
            if let Some(idx) = self.dragging_idx {
                let win = &mut self.windows[idx];
                win.x = mouse.x - self.drag_offset_x;
                win.y = mouse.y - self.drag_offset_y;
            }
        }

        if released {
            self.dragging_idx = None;
        }
    }
}

pub static mut COMPOSITOR: Compositor = Compositor::new();
