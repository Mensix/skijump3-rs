#!/usr/bin/env python3
"""Convert legacy SJ3 .SJH + PCX files into skijump3-rs custom hill files."""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image


def read_sjh(path: Path) -> dict[str, object]:
    lines = path.read_text(encoding="latin-1").splitlines()
    if len(lines) < 13 or not lines[0].startswith("*"):
        raise ValueError(f"{path} is not a supported SJH file")
    return {
        "id": path.stem,
        "name": lines[1],
        "kr": int(lines[2]),
        "front_index": lines[3],
        "back_index": lines[4],
        "back_brightness": int(lines[5]),
        "back_mirror": int(lines[6]),
        "vx_final": int(lines[7]),
        "pk_hundred": int(lines[8]),
        "pl_save_ten_thousand": int(lines[9]),
        "author": lines[10],
        "checksum": int(lines[11]),
        "profile_checksum": int(lines[12]),
    }


def pcx_to_rgba(path: Path, transparent_zero: bool) -> Image.Image:
    img = Image.open(path)
    if img.mode == "P":
        indices = img.copy()
        rgba = img.convert("RGBA")
        if transparent_zero:
            alpha = indices.convert("L").point(lambda p: 0 if p == 0 else 255)
            rgba.putalpha(alpha)
        return rgba

    rgba = img.convert("RGBA")
    if transparent_zero:
        pix = rgba.load()
        for y in range(rgba.height):
            for x in range(rgba.width):
                r, g, b, a = pix[x, y]
                if (r, g, b) == (0, 0, 0):
                    pix[x, y] = (r, g, b, 0)
    return rgba


def terrain_from_front(front: Image.Image) -> tuple[list[int], list[int], int]:
    alpha = front.getchannel("A")
    line_lengths: list[int] = []
    for y in range(front.height):
        row = alpha.crop((0, y, front.width, y + 1)).getdata()
        last = -1
        for x, a in enumerate(row):
            if a:
                last = x
        line_lengths.append(last + 1)

    profile_y: list[int] = []
    for x in range(front.width):
        y = 0
        for candidate_y, line_len in enumerate(line_lengths):
            y = candidate_y
            if line_len > x:
                break
        profile_y.append(y)

    profile_y.extend([profile_y[-1]] * (1300 - len(profile_y)))

    tip_x = 0
    former_y = 0
    for x, y in enumerate(profile_y[: front.width]):
        if y - former_y > 3:
            tip_x = x
        former_y = y
    tip_x -= 1
    return line_lengths, profile_y, tip_x


def bake_markers(
    img: Image.Image, profile_y: list[int], tip_x: int, kr: int, pk: float
) -> None:
    px = img.load()
    for x in range(tip_x, img.width - 10):
        x2 = x - tip_x
        y2 = profile_y[x] - profile_y[tip_x]
        hp = round((x2 * x2 + y2 * y2) ** 0.5 * pk * 0.5) * 5
        if hp >= (2 / 3 * kr) * 10 and hp <= kr * 12:
            color = (255, 93, 93, 255) if hp < kr * 10 else (93, 93, 255, 255)
            for dy in range(3):
                y = profile_y[x] + 1 + dy
                if 0 <= y < img.height:
                    px[x, y] = color


def toml_array(values: list[int]) -> str:
    return "[" + ", ".join(str(v) for v in values) + "]"


def find_source_file(name: str, *dirs: Path) -> Path:
    for directory in dirs:
        path = directory / name
        if path.exists():
            return path
    searched = ", ".join(str(d) for d in dirs)
    raise FileNotFoundError(f"{name} not found in: {searched}")


def write_custom_hill(root: Path, source_dir: Path, sjh_path: Path) -> None:
    hill = read_sjh(sjh_path)
    terrain_id = str(hill["front_index"])
    search_dirs = (sjh_path.parent, source_dir, root, root.parent)
    front = pcx_to_rgba(
        find_source_file(f"FRONT{terrain_id}.PCX", *search_dirs),
        transparent_zero=True,
    )
    back = pcx_to_rgba(
        find_source_file(f"BACK{hill['back_index']}.PCX", *search_dirs),
        transparent_zero=False,
    )

    line_lengths, profile_y, tip_x = terrain_from_front(front)

    bake_markers(front, profile_y, tip_x, int(hill["kr"]), int(hill["pk_hundred"]) / 100.0)

    terrain_dir = root / "hills" / "generated" / f"HILL{terrain_id}"
    terrain_dir.mkdir(parents=True, exist_ok=True)
    front.save(terrain_dir / "front_visual.png")
    back.save(terrain_dir / "back_visual.png")

    (terrain_dir / "terrain.toml").write_text(
        "\n".join(
            [
                "format_version = 1",
                f"width = {front.width}",
                f"height = {front.height}",
                f"back_width = {back.width}",
                f"back_height = {back.height}",
                f"tip_x = {tip_x}",
                f"line_lengths = {toml_array(line_lengths)}",
                f"profile_y = {toml_array(profile_y)}",
                "",
            ]
        ),
        encoding="utf-8",
    )

    custom_dir = root / "custom_hills"
    custom_dir.mkdir(parents=True, exist_ok=True)
    custom_id = str(hill["id"])
    (custom_dir / f"{custom_id}.toml").write_text(
        f'''id = "{custom_id}"
name = "{hill['name']} custom hill"

[[hills]]
id = "{custom_id}"
name = "{hill['name']}"
terrain_index = "{terrain_id}"
kr = {hill['kr']}
front_index = "{hill['front_index']}"
back_index = "{hill['back_index']}"
back_brightness = {hill['back_brightness']}
back_mirror = {str(bool(hill['back_mirror'])).lower()}
vx_final = {hill['vx_final']}
pk_hundred = {hill['pk_hundred']}
pl_save_ten_thousand = {hill['pl_save_ten_thousand']}
author = "{hill['author']}"
checksum = {hill['checksum']}
profile_checksum = {hill['profile_checksum']}
''',
        encoding="utf-8",
    )
    print(f"converted {sjh_path.name} -> custom_hills/{custom_id}.toml, HILL{terrain_id}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("sjh", nargs="+", type=Path)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--source-dir", type=Path, default=Path.cwd())
    args = parser.parse_args()
    root = args.root.resolve()
    source_dir = args.source_dir.resolve()
    for sjh in args.sjh:
        write_custom_hill(root, source_dir, sjh.resolve())


if __name__ == "__main__":
    main()
