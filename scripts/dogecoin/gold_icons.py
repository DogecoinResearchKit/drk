#!/usr/bin/env python3
"""Recolor the showcases' PNG icons from Bitview orange to Dogecoin gold (#C2A633).

Writes `crates/bitview_website/dogecoin/assets/favicon/**`, the overlay the dogecoin build lays
over the relabeled showcases (SVGs are recolored by `brk_relabel` instead), and the MCP server's
`logo.dogecoin.png`. Re-run after an upstream merge changes an icon. Needs Pillow:

    python3 -m venv /tmp/imgvenv && /tmp/imgvenv/bin/pip install pillow
    /tmp/imgvenv/bin/python scripts/dogecoin/gold_icons.py
"""

import colorsys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "crates/bitview_website/showcases/assets/favicon"
TARGET = ROOT / "crates/bitview_website/dogecoin/assets/favicon"
MCP_LOGO = ROOT / "crates/bitview_mcp/assets/logo.png"

# Bitview orange (#e65b00) sits near 24 degrees; Dogecoin gold (#c2a633) at 48, a little darker.
ORANGE_HUES = (5 / 360, 42 / 360)
HUE_SHIFT = 24 / 360
VALUE_SCALE = 0.85
MIN_SATURATION = 0.25


def gold(pixel):
    r, g, b, a = pixel
    h, s, v = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
    if a == 0 or s < MIN_SATURATION or not ORANGE_HUES[0] <= h <= ORANGE_HUES[1]:
        return pixel
    r, g, b = colorsys.hsv_to_rgb(h + HUE_SHIFT, s, v * VALUE_SCALE)
    return round(r * 255), round(g * 255), round(b * 255), a


def recolor(source, target):
    image = Image.open(source).convert("RGBA")
    image.putdata([gold(p) for p in image.get_flattened_data()])
    target.parent.mkdir(parents=True, exist_ok=True)
    image.save(target, optimize=True)
    print(target.relative_to(ROOT))


def main():
    for source in sorted(SOURCE.rglob("*.png")):
        recolor(source, TARGET / source.relative_to(SOURCE))
    recolor(MCP_LOGO, MCP_LOGO.with_name("logo.dogecoin.png"))


if __name__ == "__main__":
    main()
