//! Handcrafted Modern 24x24 Windows 11 Fluent Application and System Icons
//! Featuring multi-color gradients, depth layers, and clean vector-like geometry.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AppIcon {
    Start,
    Explorer,
    Terminal,
    Notepad,
    Paint,
    Settings,
    Browser,
    Calculator,
    TaskManager,
    Doom,
    Network,
    Volume,
}

pub const ICON_SIZE: usize = 24;

pub fn draw_icon(x: usize, y: usize, icon: AppIcon, put_pixel: &mut impl FnMut(usize, usize, u32)) {
    match icon {
        AppIcon::Start => {
            // Windows 11 iconic 4-quadrant Azure Blue logo with micro-rounded corners
            let c_top_left = 0x00A2ED;     // Sky blue
            let c_top_right = 0x0078D4;    // Classic Windows blue
            let c_bot_left = 0x0078D4;     // Medium blue
            let c_bot_right = 0x005A9E;    // Deep blue

            // Quadrant 1 (Top-Left): 2..11 x 2..11
            for r in 2..11 {
                for c in 2..11 {
                    if (r == 2 && c == 2) || (r == 2 && c == 10) || (r == 10 && c == 2) {
                        continue;
                    }
                    put_pixel(x + c, y + r, c_top_left);
                }
            }
            // Quadrant 2 (Top-Right): 13..22 x 2..11
            for r in 2..11 {
                for c in 13..22 {
                    if (r == 2 && c == 13) || (r == 2 && c == 21) || (r == 10 && c == 21) {
                        continue;
                    }
                    put_pixel(x + c, y + r, c_top_right);
                }
            }
            // Quadrant 3 (Bottom-Left): 2..11 x 13..22
            for r in 13..22 {
                for c in 2..11 {
                    if (r == 13 && c == 2) || (r == 21 && c == 2) || (r == 21 && c == 10) {
                        continue;
                    }
                    put_pixel(x + c, y + r, c_bot_left);
                }
            }
            // Quadrant 4 (Bottom-Right): 13..22 x 13..22
            for r in 13..22 {
                for c in 13..22 {
                    if (r == 13 && c == 21) || (r == 21 && c == 13) || (r == 21 && c == 21) {
                        continue;
                    }
                    put_pixel(x + c, y + r, c_bot_right);
                }
            }
        }

        AppIcon::Explorer => {
            // Modern Windows 11 File Explorer Folder
            let tab_yellow = 0xD97706;     // Amber dark
            let body_yellow = 0xFBBF24;    // Golden yellow
            let flap_blue = 0x0284C7;      // Fluent azure front flap
            let light_blue = 0x38BDF8;

            // Back folder tab
            for r in 3..7 {
                for c in 3..12 {
                    put_pixel(x + c, y + r, tab_yellow);
                }
            }
            // Back folder body
            for r in 6..19 {
                for c in 3..21 {
                    put_pixel(x + c, y + r, body_yellow);
                }
            }
            // White document preview sticking out
            for r in 7..13 {
                for c in 7..18 {
                    put_pixel(x + c, y + r, 0xF8FAFC);
                }
            }
            // Front azure flap with curved perspective
            for r in 11..21 {
                for c in 2..22 {
                    let col = if r < 14 { light_blue } else { flap_blue };
                    put_pixel(x + c, y + r, col);
                }
            }
        }

        AppIcon::Terminal => {
            // Modern Windows Terminal: dark acrylic surface with top bar and cyan prompt
            let bg_slate = 0x18181B;
            let bar_gray = 0x27272A;
            let prompt_cyan = 0x22D3EE;

            // Rounded frame
            for r in 3..21 {
                for c in 2..22 {
                    let is_top = r < 7;
                    put_pixel(x + c, y + r, if is_top { bar_gray } else { bg_slate });
                }
            }
            // Window controls: 3 dots (red, yellow, green)
            put_pixel(x + 5, y + 5, 0xEF4444);
            put_pixel(x + 8, y + 5, 0xF59E0B);
            put_pixel(x + 11, y + 5, 0x10B981);

            // Greater-than prompt `>`
            for i in 0..4 {
                put_pixel(x + 6 + i, y + 10 + i, prompt_cyan);
                put_pixel(x + 9 - i, y + 13 + i, prompt_cyan);
            }
            // Underline cursor `_`
            for c in 12..17 {
                put_pixel(x + c, y + 16, 0xF8FAFC);
            }
        }

        AppIcon::Notepad => {
            // Fluent Notepad: sky blue leatherette cover, white sheets, and bookmark
            let cover_blue = 0x0284C7;
            let sheet_white = 0xFFFFFF;
            let rule_gray = 0xCBD5E1;

            // Cover
            for r in 2..22 {
                for c in 4..20 {
                    put_pixel(x + c, y + r, cover_blue);
                }
            }
            // Inner paper pad
            for r in 4..20 {
                for c in 7..18 {
                    put_pixel(x + c, y + r, sheet_white);
                }
            }
            // Subtle horizontal rule lines
            for &r in &[8, 11, 14, 17] {
                for c in 9..16 {
                    put_pixel(x + c, y + r, rule_gray);
                }
            }
            // Spine accent on left
            for r in 2..22 {
                put_pixel(x + 4, y + r, 0x0369A1);
                put_pixel(x + 5, y + r, 0x0284C7);
            }
        }

        AppIcon::Paint => {
            // Fluent Artist Palette with colorful oil paint spots
            let palette_tan = 0xD97706;
            let palette_light = 0xF59E0B;

            // Elliptical palette body
            for r in 3..21 {
                for c in 3..21 {
                    let dr = (r as i32) - 12;
                    let dc = (c as i32) - 12;
                    let dist2 = dr * dr + dc * dc;
                    if dist2 <= 75 && !(dr > 2 && dc > 2 && dist2 <= 30) {
                        let col = if dr + dc < 0 { palette_light } else { palette_tan };
                        put_pixel(x + c, y + r, col);
                    }
                }
            }
            // Paint blotches (Red, Green, Blue, Purple)
            put_pixel(x + 7, y + 7, 0xEF4444);
            put_pixel(x + 8, y + 7, 0xEF4444);
            put_pixel(x + 13, y + 6, 0x10B981);
            put_pixel(x + 14, y + 6, 0x10B981);
            put_pixel(x + 7, y + 13, 0x3B82F6);
            put_pixel(x + 8, y + 13, 0x3B82F6);
            put_pixel(x + 11, y + 15, 0xA855F7);
        }

        AppIcon::Browser => {
            // Modern Web Browser: swirling cyan/blue sphere with white compass star
            for r in 3..21 {
                for c in 3..21 {
                    let dr = (r as i32) - 12;
                    let dc = (c as i32) - 12;
                    let dist2 = dr * dr + dc * dc;
                    if dist2 <= 72 {
                        let col = if dr + dc < -2 {
                            0x38BDF8 // Sky blue highlight
                        } else if dr + dc > 4 {
                            0x1E40AF // Deep indigo
                        } else {
                            0x0284C7 // Azure
                        };
                        put_pixel(x + c, y + r, col);
                    }
                }
            }
            // Diagonal white compass needle
            for i in -4..=4 {
                let px = (12 + i) as usize;
                let py = (12 - i) as usize;
                put_pixel(x + px, y + py, 0xFFFFFF);
                put_pixel(x + px + 1, y + py, 0xEF4444); // Red tip
            }
        }

        AppIcon::Calculator => {
            // Modern Fluent Calculator: dark slate shell, grid keys, and cyan equals button
            for r in 3..21 {
                for c in 4..20 {
                    put_pixel(x + c, y + r, 0x1E293B);
                }
            }
            // Display screen
            for r in 5..8 {
                for c in 6..18 {
                    put_pixel(x + c, y + r, 0x334155);
                }
            }
            // Number keypad buttons
            let key_color = 0x475569;
            for row in 0..3 {
                for col in 0..3 {
                    let kx = 6 + col * 4;
                    let ky = 10 + row * 3;
                    put_pixel(x + kx, y + ky, key_color);
                    put_pixel(x + kx + 1, y + ky, key_color);
                }
            }
            // Cyan '=' action button at bottom right
            put_pixel(x + 14, y + 17, 0x00A2ED);
            put_pixel(x + 15, y + 17, 0x00A2ED);
            put_pixel(x + 16, y + 17, 0x00A2ED);
            put_pixel(x + 17, y + 17, 0x00A2ED);
        }

        AppIcon::TaskManager => {
            // Modern Task Manager: frame with vibrant emerald green pulse waveform
            for r in 3..21 {
                for c in 3..21 {
                    put_pixel(x + c, y + r, 0x0F172A);
                }
            }
            // Subtle grid lines
            for r in 3..21 {
                put_pixel(x + 12, y + r, 0x1E293B);
            }
            for c in 3..21 {
                put_pixel(x + c, y + 12, 0x1E293B);
            }
            // ECG Heartbeat line in bright emerald
            let green = 0x22C55E;
            let wave: [(usize, usize); 14] = [
                (4, 12), (5, 12), (6, 12), (7, 10), (8, 15),
                (9, 6), (10, 18), (11, 12), (12, 12), (13, 11),
                (14, 13), (15, 12), (16, 12), (17, 12)
            ];
            for &(wx, wy) in wave.iter() {
                put_pixel(x + wx + 1, y + wy, green);
                put_pixel(x + wx + 1, y + wy + 1, 0x15803D); // Glow shadow
            }
        }

        AppIcon::Settings => {
            // Sleek Modern Gear
            let gear_col = 0x94A3B8;
            let gear_light = 0xE2E8F0;

            for r in 3..21 {
                for c in 3..21 {
                    let dr = (r as i32) - 12;
                    let dc = (c as i32) - 12;
                    let dist2 = dr * dr + dc * dc;

                    // Inner hole
                    if dist2 <= 9 {
                        continue;
                    }
                    // Outer circle & 8 gear teeth
                    let is_circle = dist2 <= 49;
                    let is_tooth = (dr.abs() <= 2 && dist2 <= 81)
                        || (dc.abs() <= 2 && dist2 <= 81)
                        || ((dr - dc).abs() <= 2 && dist2 <= 75)
                        || ((dr + dc).abs() <= 2 && dist2 <= 75);

                    if is_circle || is_tooth {
                        put_pixel(x + c, y + r, if dr < 0 { gear_light } else { gear_col });
                    }
                }
            }
        }

        AppIcon::Doom => {
            // DOOM Crimson Slayer Mark / Helmet
            for r in 3..21 {
                for c in 4..20 {
                    let dr = (r as i32) - 12;
                    let dc = (c as i32) - 12;
                    if dr * dr + dc * dc <= 64 {
                        put_pixel(x + c, y + r, 0xDC2626); // Crimson
                    }
                }
            }
            // Visor line
            for c in 7..17 {
                put_pixel(x + c, y + 11, 0x000000);
                put_pixel(x + c, y + 12, 0x38BDF8); // Cyan visor glow
            }
        }

        AppIcon::Network => {
            // Modern Wi-Fi Fan Arcs (3 curved concentric bands)
            let cyan = 0x38BDF8;
            // Dot at bottom
            put_pixel(x + 11, y + 18, cyan);
            put_pixel(x + 12, y + 18, cyan);
            // Mid arc
            for c in 8..16 {
                put_pixel(x + c, y + 14, cyan);
            }
            // Outer arc
            for c in 5..19 {
                put_pixel(x + c, y + 10, cyan);
            }
        }

        AppIcon::Volume => {
            // Modern Speaker + Sound Wave Arcs
            let white = 0xF8FAFC;
            // Speaker cone
            for r in 9..16 {
                put_pixel(x + 5, y + r, white);
                put_pixel(x + 6, y + r, white);
            }
            for i in 0..4 {
                let col = 7 + i;
                let top = 9 - i;
                let bot = 15 + i;
                for r in top..=bot {
                    put_pixel(x + col, y + r, white);
                }
            }
            // First wave
            put_pixel(x + 13, y + 10, white);
            put_pixel(x + 14, y + 11, white);
            put_pixel(x + 14, y + 12, white);
            put_pixel(x + 14, y + 13, white);
            put_pixel(x + 13, y + 14, white);
            // Second wave
            put_pixel(x + 16, y + 8, white);
            put_pixel(x + 18, y + 10, white);
            put_pixel(x + 18, y + 12, white);
            put_pixel(x + 18, y + 14, white);
            put_pixel(x + 16, y + 16, white);
        }
    }
}
