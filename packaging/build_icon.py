#!/usr/bin/env python3
"""Generate the Port Monitor app icon set.

Design: a deep navy gradient rounded square with three horizontal "port" rows.
Each row shows a small status LED (green/yellow/red) on the left and a port
number "label" on the right — communicating "monitoring listening ports"
without leaning on the same SF Symbol used in the menu bar.

Run from the repo root:
    python3 packaging/build_icon.py

Writes:
    packaging/port-monitor.iconset/icon_*.png
    packaging/port-monitor.icns
"""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ICONSET = ROOT / "packaging" / "port-monitor.iconset"
ICNS = ROOT / "packaging" / "port-monitor.icns"

# Standard icon sizes Apple expects in an .icns (size, scale).
SIZES = [
    (16, 1), (16, 2),
    (32, 1), (32, 2),
    (64, 1), (64, 2),
    (128, 1), (128, 2),
    (256, 1), (256, 2),
    (512, 1), (512, 2),
]

BG_TOP = (37, 99, 235)     # blue-600
BG_BOT = (29, 78, 216)     # blue-700
CARD = (255, 255, 255, 230)
DOT_LIVE = (74, 222, 128)   # green-400
DOT_IDLE = (250, 204, 21)   # amber-400
DOT_DEAD = (248, 113, 113)  # red-400
TEXT = (15, 23, 42)         # slate-900


def lerp(a: int, b: int, t: float) -> int:
    return int(a + (b - a) * t)


def gradient_bg(size: int) -> Image.Image:
    """A rounded-square gradient background."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    px = img.load()
    radius = int(size * 0.22)  # macOS Big Sur+ radius
    for y in range(size):
        t = y / max(1, size - 1)
        r = lerp(BG_TOP[0], BG_BOT[0], t)
        g = lerp(BG_TOP[1], BG_BOT[1], t)
        b = lerp(BG_TOP[2], BG_BOT[2], t)
        for x in range(size):
            # rounded mask: skip pixels outside the rounded rect
            dx = min(x, size - 1 - x)
            dy = min(y, size - 1 - y)
            if dx < radius and dy < radius:
                d = ((radius - dx) ** 2 + (radius - dy) ** 2) ** 0.5
                if d > radius:
                    continue
            px[x, y] = (r, g, b, 255)
    return img


def draw_card(canvas: Image.Image, x: int, y: int, w: int, h: int, radius: int) -> None:
    """White rounded card on the gradient."""
    draw = ImageDraw.Draw(canvas, "RGBA")
    draw.rounded_rectangle([x, y, x + w, y + h], radius=radius, fill=CARD)


def draw_dot(draw: ImageDraw.Image, cx: int, cy: int, r: int, color: tuple[int, int, int]) -> None:
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=color)


def draw_row(canvas: Image.Image, top: int, height: int, color: tuple[int, int, int]) -> None:
    """One row in the port list: LED on the left, a 'label' bar on the right."""
    draw = ImageDraw.Draw(canvas, "RGBA")
    # LED
    r = height * 0.18
    draw_dot(draw, int(height * 0.55), top + height // 2, int(r), color)
    # Label bar (simulates text)
    bar_left = int(height * 1.1)
    bar_w = int(height * 2.6)
    bar_h = int(height * 0.22)
    bar_y = top + (height - bar_h) // 2
    draw.rounded_rectangle([bar_left, bar_y, bar_left + bar_w, bar_y + bar_h],
                           radius=bar_h // 2, fill=TEXT)


def render(size: int) -> Image.Image:
    pad = int(size * 0.18)
    inner = size - 2 * pad
    img = gradient_bg(size)
    # Card
    card_pad = int(inner * 0.04)
    card_x = pad + card_pad
    card_y = pad + card_pad
    card_w = inner - 2 * card_pad
    card_h = inner - 2 * card_pad
    card_radius = int(size * 0.10)
    draw_card(img, card_x, card_y, card_w, card_h, card_radius)

    # Three rows of ports
    row_count = 3
    row_gap = int(card_h * 0.08)
    row_h = (card_h - row_gap * (row_count + 1)) // row_count
    row_top = card_y + row_gap
    colors = [DOT_LIVE, DOT_LIVE, DOT_IDLE]
    for i, color in enumerate(colors):
        draw_row(img, row_top + i * (row_h + row_gap), row_h, color)
    return img


def main() -> int:
    ICONSET.mkdir(parents=True, exist_ok=True)

    # Render at 1024 first (the source-of-truth), then downscale to all sizes.
    base = render(1024)
    for s, scale in SIZES:
        actual = s * scale
        out = base.resize((actual, actual), Image.LANCZOS)
        if scale == 1:
            name = f"icon_{s}x{s}.png"
        else:
            name = f"icon_{s}x{s}@2x.png"
        out.save(ICONSET / name)
    print(f"Wrote icons to {ICONSET}")

    # Build .icns from the iconset.
    subprocess.check_call(["iconutil", "-c", "icns", str(ICONSET), "-o", str(ICNS)])
    print(f"Wrote {ICNS}")
    return 0


if __name__ == "__main__":
    sys.exit(main())