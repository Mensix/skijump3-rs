#!/usr/bin/env python3
"""Convert indexed sprites + palette + material variants to PNG files.

Output: game/assets/sprites/png/{idx}.png         (default sprite)
        game/assets/sprites/png/{idx}_{mat_id}.png (material variant)
"""

import tomllib
import struct
import zlib
import os
import sys
from pathlib import Path

ASSETS = Path("game/assets")

# -- palette indices -------------------------------------------------------
PAL_SIZE = 256
JUMPER_SUIT_SOURCE_1 = 216
JUMPER_SUIT_SOURCE_3 = 218
JUMPER_BIB_SOURCE_1 = 220
JUMPER_BIB_SOURCE_3 = 221
JUMPER_SKI_SOURCE = 231

# -- jumper material IDs (must match materials.rs) -------------------------
MAT_ID_JUMPER_BODY = 0x1000
MAT_ID_JUMPER_SKI = 0x2000
MAT_ID_START_LIGHT = 0x3000
MAT_ID_REPLAY_SPEED = 0x4000

# -- suit / ski / bib colour data (must match jumper_colors.rs) ------------
SUIT_COLORS: list[tuple[int, int, int, int]] = [
    (0, 53, 17, 53),
    (0, 55, 33, 11),
    (0, 11, 48, 18),
    (0, 24, 28, 63),
    (0, 63, 17, 17),
    (0, 33, 33, 33),
    (1, 10, 10, 10),
    (0, 45, 17, 63),
]
SKI_COLORS: list[tuple[int, int, int]] = [
    (63, 63, 32),
    (60, 60, 60),
    (33, 60, 33),
    (63, 43, 43),
]
FADE_DOWN = [1.0, 0.87, 0.75, 0.63]
FADE_UP = [1.0, 1.50, 2.00, 2.50]
BIB_SHADE_1 = (49, 45, 0)
BIB_SHADE_3 = (34, 31, 0)

# -- theme material colours (must match theme.rs) --------------------------
MAT_START_DQ_LIT = (54, 10, 10)
MAT_START_DQ_DARK = (47, 0, 0)
MAT_START_READY_LIT = (10, 54, 10)
MAT_START_READY_DARK = (0, 47, 0)
MAT_REPLAY_ACTIVE = (10, 63, 20)
MAT_REPLAY_INACTIVE = (0, 0, 0)

# Sprite enum indices (must match sprites.rs)
SPRITE_LOGO = 61
SPRITE_INFO_PANEL = 63
SPRITE_JUMPER_INFO_BOX = 64
SPRITE_START_LIGHT = 66
SPRITE_HILL_RECORD_MARKER = 67
SPRITE_REPLAY_MODE_ICON = 68
SPRITE_TAKEOFF_ARMS_UP = 111
SPRITE_INRUN_TRANSITION = 164

# -- helpers ----------------------------------------------------------------

def load_palette(path: Path) -> list[tuple[int, int, int, int]]:
    with open(path, "rb") as f:
        data = tomllib.load(f)["data"]
    pal: list[tuple[int, int, int, int]] = []
    for i in range(PAL_SIZE):
        r6, g6, b6 = data[i * 3 : i * 3 + 3]
        if i == 0:
            pal.append((0, 0, 0, 0))
        else:
            pal.append(((r6 * 255 // 63), (g6 * 255 // 63), (b6 * 255 // 63), 255))
    return pal


def load_sprites(path: Path) -> list[dict]:
    with open(path, "rb") as f:
        data = tomllib.load(f)
    sprites = data["sprites"]
    for s in sprites:
        raw = s["pixels"].replace(" ", "").replace("\n", "")
        s["_pixels"] = bytes.fromhex(raw)
    return sprites


def resolve_rgba6(r6: int, g6: int, b6: int) -> tuple[int, int, int, int]:
    return (r6 * 255 // 63, g6 * 255 // 63, b6 * 255 // 63, 255)


def suit_shade(col: int, shade: int) -> tuple[int, int, int, int]:
    suit = SUIT_COLORS[col]
    fade = FADE_DOWN if suit[0] == 0 else FADE_UP
    fd = fade[shade]
    return resolve_rgba6(
        min(round(fd * suit[1]), 63),
        min(round(fd * suit[2]), 63),
        min(round(fd * suit[3]), 63),
    )


def bib_shade(shade: int) -> tuple[int, int, int, int]:
    c = BIB_SHADE_1 if shade == 1 else BIB_SHADE_3
    return resolve_rgba6(*c)


def ski_rgba(col: int) -> tuple[int, int, int, int]:
    return resolve_rgba6(*SKI_COLORS[col])


def make_material_palette(
    base: list[tuple[int, int, int, int]],
    overrides: dict[int, tuple[int, int, int, int]],
) -> list[tuple[int, int, int, int]]:
    pal = list(base)
    for idx, color in overrides.items():
        pal[idx] = color
    return pal


def render_sprite_rgba(
    pal: list[tuple[int, int, int, int]],
    pixels: bytes,
    w: int,
    h: int,
) -> bytearray:
    buf = bytearray()
    for byte in pixels:
        if byte == 0:
            buf.extend((0, 0, 0, 0))
        else:
            buf.extend(pal[byte])
    return buf


# -- PNG writer (stdlib only) ------------------------------------------------

def write_png(path: Path, rgba: bytes, w: int, h: int) -> None:
    """Write RGBA (4 bytes per pixel) data as a PNG file."""
    raw = b""
    for y in range(h):
        raw += b"\x00"  # filter byte = None
        row_start = y * w * 4
        raw += rgba[row_start : row_start + w * 4]

    def chunk(chunk_type: bytes, data: bytes) -> bytes:
        c = chunk_type + data
        crc = struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
        return struct.pack(">I", len(data)) + c + crc

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    idat = zlib.compress(raw)
    iend = b""
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "wb") as f:
        f.write(sig)
        f.write(chunk(b"IHDR", ihdr))
        f.write(chunk(b"IDAT", idat))
        f.write(chunk(b"IEND", iend))


# -- material variant helpers -----------------------------------------------

def jumper_body_overrides(suit: int, bib: bool) -> dict[int, tuple[int, int, int, int]]:
    return {
        JUMPER_SUIT_SOURCE_1: suit_shade(suit, 1),
        JUMPER_SUIT_SOURCE_3: suit_shade(suit, 3),
        JUMPER_BIB_SOURCE_1: bib_shade(1) if bib else suit_shade(suit, 1),
        JUMPER_BIB_SOURCE_3: bib_shade(3) if bib else suit_shade(suit, 3),
    }


def jumper_ski_overrides(ski: int) -> dict[int, tuple[int, int, int, int]]:
    return {JUMPER_SKI_SOURCE: ski_rgba(ski)}


def start_light_overrides(dq: bool) -> dict[int, tuple[int, int, int, int]]:
    if dq:
        return {253: MAT_START_DQ_LIT, 254: MAT_START_DQ_DARK}
    return {253: MAT_START_READY_LIT, 254: MAT_START_READY_DARK}


def replay_speed_overrides(mode_idx: int) -> dict[int, tuple[int, int, int, int]]:
    active = [249, 250, 253, 249, 251, 249][mode_idx]
    result = {}
    for i in range(5):
        idx = 249 + i
        result[idx] = MAT_REPLAY_ACTIVE if idx == active else MAT_REPLAY_INACTIVE
    return result


# -- main --------------------------------------------------------------------

def main():
    palette_path = ASSETS / "palette.toml"
    sprites_path = ASSETS / "sprites" / "original.toml"
    out_dir = ASSETS / "sprites" / "png"

    base_pal = load_palette(palette_path)
    sprites = load_sprites(sprites_path)
    print(f"Loaded {len(sprites)} sprites, palette {len(base_pal)} entries")

    total = 0

    for s in sprites:
        idx = s["index"]
        w = s["width"]
        h = s["height"]
        pixels = s["_pixels"]
        default_pal = make_material_palette(base_pal, {})
        rgba = render_sprite_rgba(default_pal, pixels, w, h)

        out = out_dir / f"{idx}.png"
        write_png(out, rgba, w, h)
        total += 1

    # Jumper body variants: sprite 111-164, 8 suit colors, 2 bib states
    for sprite_idx in range(SPRITE_TAKEOFF_ARMS_UP, SPRITE_INRUN_TRANSITION + 1):
        s = sprites[sprite_idx]
        w, h, pixels = s["width"], s["height"], s["_pixels"]
        for suit in range(8):
            for bib in [False, True]:
                mat_id = MAT_ID_JUMPER_BODY | suit | ((1 if bib else 0) << 8)
                ov = jumper_body_overrides(suit, bib)
                pal = make_material_palette(base_pal, ov)
                rgba = render_sprite_rgba(pal, pixels, w, h)
                out = out_dir / f"{sprite_idx}_{mat_id}.png"
                write_png(out, rgba, w, h)
                total += 1

        for ski in range(4):
            mat_id = MAT_ID_JUMPER_SKI | ski
            ov = jumper_ski_overrides(ski)
            pal = make_material_palette(base_pal, ov)
            rgba = render_sprite_rgba(pal, pixels, w, h)
            out = out_dir / f"{sprite_idx}_{mat_id}.png"
            write_png(out, rgba, w, h)
            total += 1

    # Start light variants
    s = sprites[SPRITE_START_LIGHT]
    w, h, pixels = s["width"], s["height"], s["_pixels"]
    for dq in [False, True]:
        mat_id = MAT_ID_START_LIGHT | (1 if dq else 0)
        ov = start_light_overrides(dq)
        pal = make_material_palette(base_pal, ov)
        rgba = render_sprite_rgba(pal, pixels, w, h)
        out = out_dir / f"{SPRITE_START_LIGHT}_{mat_id}.png"
        write_png(out, rgba, w, h)
        total += 1

    # Replay mode icon variants
    s = sprites[SPRITE_REPLAY_MODE_ICON]
    w, h, pixels = s["width"], s["height"], s["_pixels"]
    for mode_idx in range(6):
        ov = replay_speed_overrides(mode_idx)
        active = [249, 250, 253, 249, 251, 249][mode_idx]
        mat_id = MAT_ID_REPLAY_SPEED | active
        pal = make_material_palette(base_pal, ov)
        rgba = render_sprite_rgba(pal, pixels, w, h)
        out = out_dir / f"{SPRITE_REPLAY_MODE_ICON}_{mat_id}.png"
        write_png(out, rgba, w, h)
        total += 1

    # Write manifest
    manifest = []
    for p in sorted(out_dir.iterdir()):
        if p.suffix == ".png":
            name = p.stem
            if "_" in name:
                idx_str, mat_str = name.split("_", 1)
                manifest.append((int(idx_str), int(mat_str)))
            else:
                manifest.append((int(name), 0))
    manifest.sort(key=lambda x: (x[0], x[1]))
    manifest_path = out_dir / "manifest.json"
    import json
    with open(manifest_path, "w") as f:
        json.dump([{"idx": idx, "mat_id": mat_id} for idx, mat_id in manifest], f)
    print(f"Generated {total} PNG files + manifest in {out_dir}")


if __name__ == "__main__":
    main()
