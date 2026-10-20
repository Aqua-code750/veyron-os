//! Window Structure and Window State Management

use super::icons::AppIcon;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AppType {
    None,
    Explorer,
    Notepad,
    Paint,
    Terminal,
    Settings,
    Browser,
    Calculator,
    TaskManager,
    Doom,
}

impl AppType {
    pub fn icon(&self) -> AppIcon {
        match self {
            AppType::Explorer => AppIcon::Explorer,
            AppType::Notepad => AppIcon::Notepad,
            AppType::Paint => AppIcon::Paint,
            AppType::Terminal => AppIcon::Terminal,
            AppType::Settings => AppIcon::Settings,
            AppType::Browser => AppIcon::Browser,
            AppType::Calculator => AppIcon::Calculator,
            AppType::TaskManager => AppIcon::TaskManager,
            AppType::Doom => AppIcon::Doom,
            AppType::None => AppIcon::Start,
        }
    }
}

pub const TITLEBAR_HEIGHT: usize = 32;

pub struct Window {
    pub id: usize,
    pub title: [u8; 32],
    pub title_len: usize,
    pub app_type: AppType,
    pub x: i32,
    pub y: i32,
    pub width: usize,
    pub height: usize,
    pub prev_x: i32,
    pub prev_y: i32,
    pub prev_w: usize,
    pub prev_h: usize,
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub is_active: bool,
}

impl Window {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            title: [0; 32],
            title_len: 0,
            app_type: AppType::None,
            x: 100,
            y: 100,
            width: 600,
            height: 400,
            prev_x: 100,
            prev_y: 100,
            prev_w: 600,
            prev_h: 400,
            is_minimized: false,
            is_maximized: false,
            is_active: false,
        }
    }

    pub fn title_as_str(&self) -> &str {
        core::str::from_utf8(&self.title[..self.title_len]).unwrap_or("")
    }

    pub fn set_title(&mut self, s: &str) {
        let b = s.as_bytes();
        let len = b.len().min(31);
        self.title[..len].copy_from_slice(&b[..len]);
        self.title_len = len;
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        if self.is_minimized { return false; }
        px >= self.x && px < (self.x + self.width as i32) && py >= self.y && py < (self.y + self.height as i32)
    }

    pub fn is_in_titlebar(&self, px: i32, py: i32) -> bool {
        self.contains(px, py) && py < (self.y + TITLEBAR_HEIGHT as i32)
    }

    pub fn is_in_close_btn(&self, px: i32, py: i32) -> bool {
        if !self.is_in_titlebar(px, py) { return false; }
        let right = self.x + self.width as i32;
        px >= (right - 44) && px < right
    }

    pub fn is_in_max_btn(&self, px: i32, py: i32) -> bool {
        if !self.is_in_titlebar(px, py) { return false; }
        let right = self.x + self.width as i32;
        px >= (right - 88) && px < (right - 44)
    }

    pub fn is_in_min_btn(&self, px: i32, py: i32) -> bool {
        if !self.is_in_titlebar(px, py) { return false; }
        let right = self.x + self.width as i32;
        px >= (right - 132) && px < (right - 88)
    }
}
