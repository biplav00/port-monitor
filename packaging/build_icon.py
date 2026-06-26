#!/usr/bin/env python3
"""Generate the Port Monitor app icon set.

Design: a white rounded square with a stylized radar / port-scan motif —
a central node with three concentric arcs radiating outward. Reads as
"scanning for listening ports" but is distinct from Apple's
`dot.radiowaves.up.forward` SF Symbol already used in the menu bar:
- arcs fan to the upper-right (consistent with the broadcast direction
  in `dot.radiowaves.up.forward`),
- the center is a solid dot rather than a hollow ring,
- the outermost arc is broken into two segments to evoke "ports"
  rather than continuous signal.

Run from the repo root:
    python3 packaging/build_icon.py

Writes:
    packaging/port-monitor.iconset/icon_*.png
    packaging/port-monitor.icns
"""
from __future__ import annotations

import math
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

# Full-white, monochrome icon (per design brief).
INK = (15, 23, 42)  # slate-900 — high contrast on white


def _rounded_rect_mask(size: int, radius: int) -> Image.Image:
    """White rounded square (the 'tile')."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([0, 0, size - 1, size - 1], radius=radius, fill=(255, 255, 255, 255))
    return img


def _draw_arc(canvas: Image.Image, cx: int, cy: int, r: float,
              start_deg: float, end_deg: float, width: int) -> None:
    """Draw an arc segment as a thick stroke (open, not a pie)."""
    draw = ImageDraw.Draw(canvas, "RGBA")
    bbox = [cx - r, cy - r, cx + r, cy + r]
    draw.arc(bbox, start=start_deg, end=end_deg, fill=INK, width=width)


def _draw_dot(canvas: Image.Image, cx: int, cy: int, r: float) -> None:
    draw = ImageDraw.Draw(canvas, "RGBA")
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=INK)


def _draw_port_segment(canvas: Image.Image, cx: int, cy: int, r: float,
                       gap_deg: float, width: int) -> None:
    """A nearly-full ring with a small gap (suggests a 'port' / opening)."""
    draw = ImageDraw.Draw(canvas, "RGBA")
    bbox = [cx - r, cy - r, cx + r, cy + r]
    # Draw most of the ring, leave a small gap at the top-right.
    draw.arc(bbox, start=gap_deg, end=360 - gap_deg, fill=INK, width=width)


def render(size: int) -> Image.Image:
    canvas = _rounded_rect_mask(size, radius=int(size * 0.22))

    # Anchor in the lower-left quadrant so the arcs fan up-right,
    # echoing `dot.radiowaves.up.forward` without copying it.
    cx = int(size * 0.34)
    cy = int(size * 0.66)
    max_r = size * 0.40
    width = max(2, int(size * 0.075))

    # Solid dot at the origin (the 'listening port').
    _draw_dot(canvas, cx, cy, r=size * 0.06)

    # Three concentric arcs fanning up-right. The outermost is split
    # into two short segments to evoke 'ports' rather than a continuous
    # signal — that's the visual distinction from the wifi symbol.
    r1 = max_r * 0.45
    _draw_arc(canvas, cx, cy, r1, start_deg=-70, end_deg=-20, width=width)

    r2 = max_r * 0.78
    _draw_arc(canvas, cx, cy, r2, start_deg=-65, end_deg=-25, width=width)

    # Outermost: two short segments with a small gap.
    r3 = max_r * 1.10
    _draw_arc(canvas, cx, cy, r3, start_deg=-60, end_deg=-38, width=width)
    _draw_arc(canvas, cx, cy, r3, start_deg=-28, end_deg=-12, width=width)

    return canvas


def main() -> int:
    ICONSET.mkdir(parents=True, exist_ok=True)

    base = render(1024)
    for s, scale in SIZES:
        actual = s * scale
        out = base.resize((actual, actual), Image.LANCZOS)
        name = f"icon_{s}x{s}.png" if scale == 1 else f"icon_{s}x{s}@2x.png"
        out.save(ICONSET / name)
    print(f"Wrote icons to {ICONSET}")

    subprocess.check_call(["iconutil", "-c", "icns", str(ICONSET), "-o", str(ICNS)])
    print(f"Wrote {ICNS}")
    return 0


if __name__ == "__main__":
    sys.exit(main())