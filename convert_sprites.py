#!/usr/bin/env python3
"""Convert indexed sprites + palette to type-3 indexed PNG files.

Output: game/assets/sprites/png/{idx}.png  (indexed PNG, palette-based)

The PNGs carry the original 256-entry RGBA palette (indices 0-255) where
index 0 is transparent. Material variants are baked at game startup by
reading the palette indices directly from the PNG and applying colour
overrides before uploading to SDL2.
"""

import tomllib
import struct
import zlib
from pathlib import Path

ASSETS = Path("game/assets")
PAL_SIZE = 256


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


def write_indexed_png(path: Path, indices: bytes, pal: list, w: int, h: int) -> None:
    raw = b""
    for y in range(h):
        raw += b"\x00"
        raw += indices[y * w : (y + 1) * w]

    def chunk(ctype: bytes, data: bytes) -> bytes:
        c = ctype + data
        crc = struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
        return struct.pack(">I", len(data)) + c + crc

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 3, 0, 0, 0)
    plte = b"".join(bytes([c[0], c[1], c[2]]) for c in pal)
    trns = bytes(c[3] for c in pal)
    idat = zlib.compress(raw)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "wb") as f:
        f.write(sig)
        f.write(chunk(b"IHDR", ihdr))
        f.write(chunk(b"PLTE", plte))
        f.write(chunk(b"tRNS", trns))
        f.write(chunk(b"IDAT", idat))
        f.write(chunk(b"IEND", b""))


def main():
    palette_path = ASSETS / "palette.toml"
    sprites_path = ASSETS / "sprites" / "original.toml"
    out_dir = ASSETS / "sprites" / "png"

    base_pal = load_palette(palette_path)
    sprites = load_sprites(sprites_path)
    print(f"Loaded {len(sprites)} sprites, palette {len(base_pal)} entries")

    for s in sprites:
        write_indexed_png(out_dir / f"{s['index']}.png", s["_pixels"], base_pal, s["width"], s["height"])
    print(f"Generated {len(sprites)} base indexed PNGs")

    for old in ["manifest.json", "source_colors.json", "source_colors.toml"]:
        (out_dir / old).unlink(missing_ok=True)
    for p in out_dir.iterdir():
        if p.suffix == ".png" and "_" in p.stem:
            p.unlink(missing_ok=True)

    print(f"Done — {len(sprites)} indexed PNGs in {out_dir}")


if __name__ == "__main__":
    main()
