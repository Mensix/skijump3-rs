#!/usr/bin/env python3
"""Convert Ski Jump 3 PCX assets to PNG + TOML.

For each hill, produces:
  - hills/generated/HILL{idx}/terrain.toml   geometry metadata
  - hills/generated/HILL{idx}/front_rgba.png RGBA rendered terrain
  - hills/generated/HILL{idx}/back_rgba.png  RGBA rendered sky
  - hills/generated/HILL{idx}/front_mask.png grayscale indexed mask
  - hills/generated/HILL{idx}/back_mask.png  grayscale indexed mask

Also produces:
  - palette.bin  768-byte combined 6-bit palette (MAIN.PCX + standard UI)

Usage:  python3 tools/convert_assets.py [assets_dir]
"""

import os
import struct
import sys
import tomllib
from PIL import Image

HILL_PROFILE_LEN = 1300

STANDARD_UI_PALETTE = [
    [53, 17, 53],
    [63, 0, 0],
    [43, 12, 43],
    [63, 0, 0],
    [49, 45, 0],
    [34, 31, 0],
    [63, 0, 0],
    [56, 54, 54],
    [63, 63, 21],
    [54, 52, 10],
    [42, 42, 42],
    [42, 20, 10],
    [21, 21, 21],
    [57, 45, 38],
    [63, 0, 0],
    [63, 63, 32],
    [40, 40, 41],
    [48, 48, 49],
    [55, 55, 56],
    [63, 63, 63],
    [56, 13, 13],
    [13, 53, 13],
    [23, 23, 63],
    [63, 23, 23],
    [63, 63, 63],
    [44, 44, 44],
    [0, 0, 0],
    [18, 13, 34],
    [34, 13, 18],
    [20, 20, 20],
    [63, 57, 9],
    [9, 57, 63],
    [23, 16, 43],
    [43, 16, 23],
    [26, 26, 26],
    [52, 47, 0],
    [0, 47, 52],
    [46, 46, 63],
    [32, 32, 63],
    [63, 63, 63],
]


def scale_6bit(v):
    return (v * 255) // 63


def parse_pcx(path):
    """Return (width, height, pixels_bytearray, palette_6bit, rgba_pixels)."""
    with open(path, "rb") as f:
        data = f.read()
    if len(data) <= 128 + 768:
        raise ValueError(f"PCX file too small: {len(data)} bytes")
    width = struct.unpack_from("<H", data, 8)[0] + 1
    height = struct.unpack_from("<H", data, 10)[0] + 1
    total = width * height
    raw = bytearray(data[128:])
    pixels = bytearray()
    i = 0
    while len(pixels) < total and i < len(raw):
        b1 = raw[i]
        i += 1
        if b1 >= 192:
            if i >= len(raw):
                break
            count = b1 - 192
            b2 = raw[i]
            i += 1
            for _ in range(count):
                if len(pixels) < total:
                    pixels.append(b2)
        elif len(pixels) < total:
            pixels.append(b1)
    if len(pixels) != total:
        raise ValueError(f"PCX truncated: expected {total} pixels, decoded {len(pixels)}")
    palette_8bit = data[-768:]
    palette_6bit = bytearray(b >> 2 for b in palette_8bit)
    rgba = bytearray()
    for idx in pixels:
        if idx == 0:
            rgba.extend([0, 0, 0, 0])
        else:
            off = idx * 3
            rgba.extend([
                scale_6bit(palette_6bit[off]),
                scale_6bit(palette_6bit[off + 1]),
                scale_6bit(palette_6bit[off + 2]),
                255,
            ])
    return width, height, pixels, palette_6bit, rgba


def indexed_to_rgba(pixels, palette):
    """Convert indexed pixel array to RGBA bytes."""
    rgba = bytearray()
    for idx in pixels:
        if idx == 0:
            rgba.extend([0, 0, 0, 0])
        else:
            off = idx * 3
            rgba.extend([
                scale_6bit(palette[off]),
                scale_6bit(palette[off + 1]),
                scale_6bit(palette[off + 2]),
                255,
            ])
    return rgba


def line_lengths(pixels, width, height):
    lines = []
    for y in range(height):
        row = pixels[y * width:(y + 1) * width]
        found = 0
        for x in range(width - 1, -1, -1):
            if row[x] != 0:
                found = x + 1
                break
        lines.append(found)
    return lines


def profile_y(line_lengths, width, height):
    profile = [0] * HILL_PROFILE_LEN
    clip = min(width, HILL_PROFILE_LEN)
    for x in range(clip):
        found = height - 1
        for y, ll in enumerate(line_lengths):
            if ll > x:
                found = y
                break
        profile[x] = found
    last = profile[min(width - 1, HILL_PROFILE_LEN - 1)]
    for x in range(width, HILL_PROFILE_LEN):
        profile[x] = last
    return profile


def tip_x(profile_y, width):
    tip = 0
    former = 0
    for x in range(min(width, len(profile_y))):
        if profile_y[x] - former > 3:
            tip = x
        former = profile_y[x]
    return tip - 1


def draw_distance_markers(pixels, width, profile_y, tip, kr, pk):
    tip_idx = max(tip, 0)
    drawable = min(width, len(profile_y))
    for x in range(tip_idx, drawable - 10):
        x2 = x - tip
        y2 = profile_y[x] - profile_y[tip_idx]
        hp = int(((x2 * x2 + y2 * y2) ** 0.5) * pk * 0.5 + 0.5) * 5
        if (2 * kr * 10) // 3 <= hp <= kr * 12:
            color = 238 if hp < kr * 10 else 239
            for dy in range(3):
                y = profile_y[x] + dy + 1
                if y >= 0 and y * width + x < len(pixels):
                    pixels[y * width + x] = color


def mirror_pixels(pixels, width, height):
    for y in range(height):
        row = pixels[y * width:(y + 1) * width]
        row.reverse()
        pixels[y * width:(y + 1) * width] = row


def mirror_rgba(rgba, width, height):
    stride = width * 4
    for y in range(height):
        row = bytearray(rgba[y * stride:(y + 1) * stride])
        flipped = bytearray()
        for x in range(width - 1, -1, -1):
            flipped.extend(row[x * 4:(x + 1) * 4])
        rgba[y * stride:(y + 1) * stride] = flipped


def convert_palette(assets_dir):
    main_pcx = os.path.join(assets_dir, "MAIN.PCX")
    with open(main_pcx, "rb") as f:
        data = f.read()
    combined = bytearray(b >> 2 for b in data[-768:])
    base = 216
    for i, rgb in enumerate(STANDARD_UI_PALETTE):
        off = (base + i) * 3
        combined[off] = rgb[0]
        combined[off + 1] = rgb[1]
        combined[off + 2] = rgb[2]
    # Write as flat TOML array
    entries = ", ".join(str(b) for b in combined)
    toml = f"format_version = 1\ndata = [{entries}]\n"
    pal_path = os.path.join(assets_dir, "palette.toml")
    with open(pal_path, "w") as f:
        f.write(toml)
    print(f"  palette.toml ({len(combined)} entries)")


def convert_hills(assets_dir):
    hills_toml = os.path.join(assets_dir, "hills", "original.toml")
    with open(hills_toml, "rb") as f:
        catalog = tomllib.load(f)

    out_base = os.path.join(assets_dir, "hills", "generated")
    os.makedirs(out_base, exist_ok=True)

    for idx, hill in enumerate(catalog["hills"]):
        front_idx = hill["front_index"]
        back_idx = hill["back_index"]
        kr = hill["kr"]
        pk = hill["pk_hundred"] / 100.0
        back_mirror = hill.get("back_mirror", False)

        front_pcx = os.path.join(assets_dir, f"FRONT{front_idx}.PCX")
        back_pcx = os.path.join(assets_dir, f"BACK{back_idx}.PCX")

        print(f"[{idx}/{len(catalog['hills'])}] {hill['name']} ...", end=" ")
        sys.stdout.flush()

        fw, fh, front_pixels, front_pal, front_rgba_orig = parse_pcx(front_pcx)
        bw, bh, back_pixels, back_pal, back_rgba = parse_pcx(back_pcx)

        # Compute geometry from original (pre-marker) front pixels
        ll = line_lengths(front_pixels, fw, fh)
        py = profile_y(ll, fw, fh)
        tip = tip_x(py, fw)

        # Draw distance markers on a copy of front pixels
        marked = bytearray(front_pixels)
        draw_distance_markers(marked, fw, py, tip, kr, pk)

        # RGBA from marked pixels
        marked_rgba = indexed_to_rgba(marked, front_pal)

        # Handle back mirror
        if back_mirror:
            mirror_pixels(back_pixels, bw, bh)
            mirror_rgba(back_rgba, bw, bh)

        # Write output files
        hill_dir = os.path.join(out_base, f"HILL{idx}")
        os.makedirs(hill_dir, exist_ok=True)

        # RGBA PNGs
        front_rgba_img = Image.frombytes("RGBA", (fw, fh), bytes(marked_rgba))
        front_rgba_img.save(os.path.join(hill_dir, "front_rgba.png"))

        back_rgba_img = Image.frombytes("RGBA", (bw, bh), bytes(back_rgba))
        back_rgba_img.save(os.path.join(hill_dir, "back_rgba.png"))

        # Grayscale masks (single channel, 0-255)
        front_mask_img = Image.frombytes("L", (fw, fh), bytes(marked))
        front_mask_img.save(os.path.join(hill_dir, "front_mask.png"))

        back_mask_img = Image.frombytes("L", (bw, bh), bytes(back_pixels))
        back_mask_img.save(os.path.join(hill_dir, "back_mask.png"))

        # Metadata TOML
        toml_lines = [
            'format_version = 1',
            f'width = {fw}',
            f'height = {fh}',
            f'back_width = {bw}',
            f'back_height = {bh}',
            f'tip_x = {tip}',
            f'line_lengths = [{", ".join(str(x) for x in ll)}]',
            f'profile_y = [{", ".join(str(x) for x in py)}]',
        ]
        with open(os.path.join(hill_dir, "terrain.toml"), "w") as f:
            f.write("\n".join(toml_lines) + "\n")

        print("done")


def main():
    assets_dir = sys.argv[1] if len(sys.argv) > 1 else "game/assets"
    assets_dir = os.path.abspath(assets_dir)

    print("Converting palette...")
    convert_palette(assets_dir)
    print()
    print("Converting hills...")
    convert_hills(assets_dir)
    print("\nDone.")


if __name__ == "__main__":
    main()
