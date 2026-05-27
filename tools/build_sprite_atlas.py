#!/usr/bin/env python3
"""Generate RGBA sprite atlas from indexed sprite definitions + PCX palette.

Usage: python3 tools/build_sprite_atlas.py
Output: game/assets/sprites/original_atlas.png
        game/assets/sprites/original_atlas.toml
"""

import os
import struct
import tomllib
import zlib

BASE = os.path.join(os.path.dirname(__file__), '..', 'game', 'assets')
PCX_PATH = os.path.join(BASE, 'MAIN.PCX')
SPRITES_TOML = os.path.join(BASE, 'sprites', 'original.toml')
OUT_PNG = os.path.join(BASE, 'sprites', 'original_atlas.png')
OUT_TOML = os.path.join(BASE, 'sprites', 'original_atlas.toml')
PADDING = 1


def read_pcx_palette(path: str) -> list[tuple[int, int, int]]:
    """Extract 256-color palette from PCX (last 768 bytes)."""
    with open(path, 'rb') as f:
        f.seek(-768, os.SEEK_END)
        data = f.read(768)
    palette: list[tuple[int, int, int]] = []
    for i in range(256):
        r6 = data[i * 3] >> 2
        g6 = data[i * 3 + 1] >> 2
        b6 = data[i * 3 + 2] >> 2
        palette.append(((r6 * 255) // 63, (g6 * 255) // 63, (b6 * 255) // 63))
    return palette


def decode_hex_pixels(s: str) -> list[int]:
    """Decode hex-encoded indexed pixel data (whitespace ignored)."""
    cleaned = s.replace(' ', '').replace('\n', '').replace('\r', '').replace('\t', '')
    return [int(cleaned[i:i + 2], 16) for i in range(0, len(cleaned), 2)]


def indexed_to_rgba(indexed: list[int], palette: list[tuple[int, int, int]]) -> bytes:
    """Convert indexed pixels to RGBA bytes. Index 0 = transparent."""
    out = bytearray()
    for idx in indexed:
        if idx == 0:
            out.extend([0, 0, 0, 0])
        else:
            r, g, b = palette[idx]
            out.extend([r, g, b, 255])
    return bytes(out)


def write_png(path: str, pixels: bytes, width: int, height: int):
    """Write RGBA pixels as valid PNG (no external deps)."""

    def chunk(typ: bytes, data: bytes) -> bytes:
        body = typ + data
        return struct.pack('>I', len(data)) + body + struct.pack('>I', zlib.crc32(body) & 0xffffffff)

    raw = bytearray()
    for y in range(height):
        row_start = y * width * 4
        raw.append(0)  # filter byte: None
        raw.extend(pixels[row_start:row_start + width * 4])
    compressed = zlib.compress(bytes(raw), 9)

    with open(path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n')
        ihdr = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)  # 8-bit RGBA
        f.write(chunk(b'IHDR', ihdr))
        f.write(chunk(b'IDAT', compressed))
        f.write(chunk(b'IEND', b''))

    print(f'  wrote {path}  ({width}x{height})')


def main():
    print('Reading PCX palette...')
    palette = read_pcx_palette(PCX_PATH)
    print(f'  loaded {len(palette)} palette entries')

    print('Reading sprite definitions...')
    with open(SPRITES_TOML, 'rb') as f:
        sprites_toml = tomllib.load(f)
    entries = sprites_toml['sprites']
    print(f'  found {len(entries)} sprites')

    # Decode each sprite into RGBA and compute atlas layout (horizontal strip)
    total_w = sum(e['width'] for e in entries) + PADDING * 2 * len(entries)
    max_h = max(e['height'] for e in entries) + PADDING * 2

    print(f'  atlas size: {total_w}x{max_h}')

    atlas_pixels = bytearray(total_w * max_h * 4)
    # Initialize to transparent black
    # (already zero-initialized)

    atlas_regions = []

    cx = PADDING
    for entry in entries:
        w = entry['width']
        h = entry['height']
        indexed = decode_hex_pixels(entry['pixels'])
        assert len(indexed) == w * h, f'sprite {entry["index"]}: expected {w * h} pixels, got {len(indexed)}'
        rgba = indexed_to_rgba(indexed, palette)

        # Copy into atlas
        for row in range(h):
            src_start = row * w * 4
            dst_start = ((PADDING + row) * total_w + cx) * 4
            atlas_pixels[dst_start:dst_start + w * 4] = rgba[src_start:src_start + w * 4]

        atlas_regions.append({
            'index': entry['index'],
            'x': cx,
            'y': PADDING,
            'width': w,
            'height': h,
            'center_x': entry['center_x'],
            'center_y': entry['center_y'],
        })
        cx += w + PADDING * 2

    print('Writing atlas PNG...')
    write_png(OUT_PNG, bytes(atlas_pixels), total_w, max_h)

    print('Writing atlas TOML...')
    with open(OUT_TOML, 'w', encoding='utf-8') as f:
        f.write('format_version = 2\n')
        f.write(f'image = "sprites/original_atlas.png"\n\n')
        for reg in atlas_regions:
            f.write('[[sprites]]\n')
            f.write(f'index = {reg["index"]}\n')
            f.write(f'x = {reg["x"]}\n')
            f.write(f'y = {reg["y"]}\n')
            f.write(f'width = {reg["width"]}\n')
            f.write(f'height = {reg["height"]}\n')
            f.write(f'center_x = {reg["center_x"]}\n')
            f.write(f'center_y = {reg["center_y"]}\n\n')
    print(f'  wrote {OUT_TOML}')

    print('Done.')


if __name__ == '__main__':
    main()
