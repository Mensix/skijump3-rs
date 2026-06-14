#!/usr/bin/env python3
"""Convert indexed sprites + palette to base PNG files + source color mapping.

Output: game/assets/sprites/png/{idx}.png  (base sprite, rendered with default palette)
        game/assets/sprites/png/source_colors.toml  (override source palette indices → RGBA)

Material variants are baked at game startup by matching source RGBA colors
in the base PNG and replacing with override colors. This lets modders add
new suit/ski/UI colors by editing Rust code only — no Python re-run needed.
"""

import tomllib
import struct
import zlib
from pathlib import Path

ASSETS = Path("game/assets")

PAL_SIZE = 256

# Override source palette indices (must match jumper_colors.rs)
SOURCE_INDICES = {216, 218, 220, 221, 231, 249, 250, 251, 252, 253, 254}


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


def render_sprite_rgba(pal: list[tuple[int, int, int, int]], pixels: bytes, w: int, h: int) -> bytearray:
    buf = bytearray()
    for byte in pixels:
        if byte == 0:
            buf.extend((0, 0, 0, 0))
        elif byte in SOURCE_INDICES:
            c = pal[byte]
            buf.extend((c[0], c[1], c[2], byte))
        else:
            buf.extend(pal[byte])
    return buf


def write_png(path: Path, rgba: bytes, w: int, h: int) -> None:
    raw = b""
    for y in range(h):
        raw += b"\x00"
        row_start = y * w * 4
        raw += rgba[row_start : row_start + w * 4]

    def chunk(chunk_type: bytes, data: bytes) -> bytes:
        c = chunk_type + data
        crc = struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
        return struct.pack(">I", len(data)) + c + crc

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    idat = zlib.compress(raw)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "wb") as f:
        f.write(sig)
        f.write(chunk(b"IHDR", ihdr))
        f.write(chunk(b"IDAT", idat))
        f.write(chunk(b"IEND", iend := b""))


def main():
    palette_path = ASSETS / "palette.toml"
    sprites_path = ASSETS / "sprites" / "original.toml"
    out_dir = ASSETS / "sprites" / "png"

    base_pal = load_palette(palette_path)
    sprites = load_sprites(sprites_path)
    print(f"Loaded {len(sprites)} sprites, palette {len(base_pal)} entries")

    # Generate base PNG for each sprite (default palette, no overrides)
    for s in sprites:
        idx = s["index"]
        rgba = render_sprite_rgba(base_pal, s["_pixels"], s["width"], s["height"])
        write_png(out_dir / f"{idx}.png", rgba, s["width"], s["height"])
    print(f"Generated {len(sprites)} base PNGs")

    for p in out_dir.iterdir():
        if p.name.endswith(".png") and "_" in p.stem:
            p.unlink(missing_ok=True)
    for old in ["manifest.json", "source_colors.json", "source_colors.toml"]:
        (out_dir / old).unlink(missing_ok=True)

    print(f"Done — {len(sprites)} base PNGs (source indices encoded in alpha) in {out_dir}")


if __name__ == "__main__":
    main()
