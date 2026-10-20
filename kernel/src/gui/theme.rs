//! Windows 11 Design Theme & Styling for Veyron OS
//! Colors, rounded corners, translucent acrylic effects, and typography.

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn to_u32(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn blend(&self, bg: Color) -> Color {
        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;
        Color {
            r: (((self.r as u32 * alpha) + (bg.r as u32 * inv_alpha)) / 255) as u8,
            g: (((self.g as u32 * alpha) + (bg.g as u32 * inv_alpha)) / 255) as u8,
            b: (((self.b as u32 * alpha) + (bg.b as u32 * inv_alpha)) / 255) as u8,
            a: 255,
        }
    }
}

pub struct Theme {
    pub is_dark: bool,
    pub accent: Color,
    pub desktop_bg: Color,
    pub taskbar_bg: Color,
    pub window_bg: Color,
    pub window_titlebar: Color,
    pub window_border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub button_bg: Color,
    pub button_hover: Color,
    pub control_close_hover: Color,
}

impl Theme {
    pub const fn dark() -> Self {
        Self {
            is_dark: true,
            accent: Color::rgb(0x00, 0x78, 0xD4),              // Windows 11 Blue
            desktop_bg: Color::rgb(0x0F, 0x17, 0x2A),          // Deep Slate
            taskbar_bg: Color::rgba(0x1E, 0x29, 0x3B, 210),    // Translucent Slate
            window_bg: Color::rgb(0x20, 0x20, 0x20),           // Dark Window Surface
            window_titlebar: Color::rgb(0x2B, 0x2D, 0x30),     // Dark Titlebar
            window_border: Color::rgb(0x3F, 0x3F, 0x46),       // Subtle Border
            text_primary: Color::rgb(0xF8, 0xFA, 0xFC),        // Pure Light Text
            text_secondary: Color::rgb(0x94, 0xA3, 0xB8),      // Muted Secondary Text
            button_bg: Color::rgb(0x33, 0x33, 0x33),          // Dark Button
            button_hover: Color::rgb(0x44, 0x44, 0x44),       // Hover State
            control_close_hover: Color::rgb(0xE8, 0x11, 0x23),// Red Close Button
        }
    }
}

pub static mut CURRENT_THEME: Theme = Theme::dark();
