#!/usr/bin/env python3
"""
Generates Fancy 'V' Icons for Veyron OS Chrome Extension
Renders anti-aliased, multi-layered glowing V emblem with gradients and bevels.
"""

import os
from PIL import Image, ImageDraw, ImageFilter

def create_fancy_v_icon(size):
    # Render at 4x resolution for smooth anti-aliasing
    scale = 4
    dim = size * scale
    img = Image.new("RGBA", (dim, dim), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    cx = dim / 2
    cy = dim / 2
    r_outer = dim * 0.46

    # 1. Dark circular backing with subtle radial aura
    for dr in range(int(r_outer), 0, -2):
        factor = dr / r_outer
        # Dark navy/slate base: #0F172A to #1E293B
        red = int(12 + (1 - factor) * 15)
        green = int(18 + (1 - factor) * 30)
        blue = int(35 + (1 - factor) * 60)
        alpha = int(240 * (1 if factor < 0.95 else (1 - factor) * 20))
        draw.ellipse([cx - dr, cy - dr, cx + dr, cy + dr], fill=(red, green, blue, alpha))

    # 2. Outer glowing cyan ring border
    ring_thick = max(2, int(dim * 0.03))
    for i in range(ring_thick):
        rad = r_outer - i
        draw.ellipse([cx - rad, cy - rad, cx + rad, cy + rad], outline=(0, 220, 255, 230 - i * 40))

    # 3. Draw the Fancy 'V' geometry:
    # Wings coordinates:
    # Top Left: (cx - dim*0.28, cy - dim*0.24)
    # Top Outer Left Flange: (cx - dim*0.35, cy - dim*0.18)
    # Bottom Center V Tip: (cx, cy + dim*0.30)
    # Inner V Crest: (cx, cy - dim*0.04)
    # Top Right: (cx + dim*0.28, cy - dim*0.24)
    # Top Outer Right Flange: (cx + dim*0.35, cy - dim*0.18)

    p_tip = (cx, cy + dim * 0.30)
    p_inner_v = (cx, cy - dim * 0.02)
    p_tl_outer = (cx - dim * 0.32, cy - dim * 0.22)
    p_tl_inner = (cx - dim * 0.14, cy - dim * 0.25)
    p_tr_outer = (cx + dim * 0.32, cy - dim * 0.22)
    p_tr_inner = (cx + dim * 0.14, cy - dim * 0.25)

    # Left Wing (Cyan to Deep Azure)
    poly_left = [
        p_tip,
        (cx - dim * 0.08, cy + dim * 0.12),
        p_tl_outer,
        (cx - dim * 0.22, cy - dim * 0.26),
        p_tl_inner,
        p_inner_v
    ]
    # Draw left wing with multi-layer shading
    draw.polygon(poly_left, fill=(0, 162, 237, 255))
    # Left wing highlight edge
    draw.line([p_tl_outer, (cx - dim * 0.22, cy - dim * 0.26), p_tl_inner, p_inner_v, p_tip], fill=(120, 240, 255, 255), width=max(2, int(dim*0.02)))

    # Right Wing (Electric Violet to Cyan-Blue)
    poly_right = [
        p_tip,
        p_inner_v,
        p_tr_inner,
        (cx + dim * 0.22, cy - dim * 0.26),
        p_tr_outer,
        (cx + dim * 0.08, cy + dim * 0.12),
    ]
    draw.polygon(poly_right, fill=(30, 100, 235, 255))
    draw.line([p_tr_outer, (cx + dim * 0.22, cy - dim * 0.26), p_tr_inner, p_inner_v, p_tip], fill=(160, 220, 255, 255), width=max(2, int(dim*0.02)))

    # Central Bevel Ridge & Glowing Diamond Core
    p_mid_ridge = (cx, cy + dim * 0.10)
    draw.line([p_inner_v, p_tip], fill=(255, 255, 255, 230), width=max(2, int(dim*0.025)))

    # Small top chevron accent
    chev_h = dim * 0.06
    draw.polygon([
        (cx, cy - dim * 0.12),
        (cx - dim * 0.08, cy - dim * 0.18),
        (cx, cy - dim * 0.15),
        (cx + dim * 0.08, cy - dim * 0.18),
    ], fill=(0, 240, 255, 220))

    # Downscale using LANCZOS for crystal sharp anti-aliasing
    final_img = img.resize((size, size), Image.Resampling.LANCZOS)
    return final_img

def main():
    os.makedirs("icons", exist_ok=True)
    sizes = [16, 32, 48, 128, 256]
    for s in sizes:
        icon = create_fancy_v_icon(s)
        path = os.path.join("icons", f"icon{s}.png")
        icon.save(path, "PNG")
        print(f"Generated {path} ({s}x{s})")

if __name__ == "__main__":
    main()
