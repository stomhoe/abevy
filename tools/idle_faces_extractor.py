#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

"""
Extract the idle facing frames from a spritesheet animation definition.

The script reads an .anim.ron file, finds the animations whose ids end in
_north_idle, _south_idle, _west_idle, or _east_idle, and writes one PNG per
facing direction using the common id prefix.

Usage:
    idle_faces_extractor.py <input.anim.ron> [output_dir]
"""

try:
    from PIL import Image  # pyright: ignore[reportMissingImports]
except ImportError as exc:  # pragma: no cover - local utility
    raise SystemExit("Pillow is required: pip install pillow") from exc


ROOT_DIR = Path(__file__).resolve().parents[1]
ASSETS_DIR = ROOT_DIR / "assets"
DIRECTIONS = ("north", "south", "west", "east")
IDLE_ID_RE = re.compile(r"^(?P<prefix>.+)_(?P<direction>north|south|west|east)_idle$")
RECORD_START_RE = re.compile(r'\(\s*id:\s*"')
ID_RE = re.compile(r'id:\s*"([^"]+)"')
IMG_RE = re.compile(r'img_path:\s*"([^"]+)"')
ROWS_COLS_RE = re.compile(r"rows_cols:\s*\(\s*(\d+)\s*,\s*(\d+)\s*\)")
CLIP_RE = re.compile(r"\(\s*target:\s*(\d+),\s*is_row:\s*(true|false)(?P<body>.*?)\n\s*\),", re.S)
PARTIAL_RE = re.compile(r"partial:\s*Some\(\(\s*(\d+)\s*,\s*(\d+)\s*\)\)")
START_FRAME_RE = re.compile(r"start_frame:\s*Some\(\s*(\d+)\s*\)")


@dataclass(frozen=True)
class IdleFaceSpec:
    anim_id: str
    prefix: str
    direction: str
    img_path: str
    rows: int
    cols: int
    target: int
    is_row: bool
    partial_start: int
    start_frame: int


def _extract_record_blocks(text: str) -> list[str]:
    blocks: list[str] = []
    for match in RECORD_START_RE.finditer(text):
        start = match.start()
        depth = 0
        end = None
        for idx in range(start, len(text)):
            char = text[idx]
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
                if depth == 0:
                    end = idx + 1
                    break
        if end is not None:
            blocks.append(text[start:end])
    return blocks


def _parse_idle_face_specs(text: str) -> list[IdleFaceSpec]:
    specs: list[IdleFaceSpec] = []
    for block in _extract_record_blocks(text):
        id_match = ID_RE.search(block)
        img_match = IMG_RE.search(block)
        rows_cols_match = ROWS_COLS_RE.search(block)
        clip_match = CLIP_RE.search(block)
        if not (id_match and img_match and rows_cols_match and clip_match):
            continue

        anim_id = id_match.group(1)
        idle_match = IDLE_ID_RE.fullmatch(anim_id)
        if idle_match is None:
            continue

        prefix = idle_match.group("prefix")
        direction = idle_match.group("direction")
        rows = int(rows_cols_match.group(1))
        cols = int(rows_cols_match.group(2))

        target = int(clip_match.group(1))
        is_row = clip_match.group(2) == "true"
        clip_body = clip_match.group("body")

        partial_match = PARTIAL_RE.search(clip_body)
        partial_start = int(partial_match.group(1)) if partial_match else 0

        start_frame_match = START_FRAME_RE.search(clip_body)
        start_frame = int(start_frame_match.group(1)) if start_frame_match else 0

        specs.append(
            IdleFaceSpec(
                anim_id=anim_id,
                prefix=prefix,
                direction=direction,
                img_path=img_match.group(1),
                rows=rows,
                cols=cols,
                target=target,
                is_row=is_row,
                partial_start=partial_start,
                start_frame=start_frame,
            )
        )
    return specs


def _resolve_asset_path(img_path: str) -> Path:
    path = Path(img_path)
    if path.is_absolute():
        return path
    return ASSETS_DIR / path


def _select_cell(spec: IdleFaceSpec) -> tuple[int, int]:
    frame_index = spec.partial_start + spec.start_frame
    if spec.is_row:
        return spec.target, frame_index
    return frame_index, spec.target


def _crop_frame(img: Image.Image, spec: IdleFaceSpec) -> Image.Image:
    row_idx, col_idx = _select_cell(spec)
    if row_idx < 0 or row_idx >= spec.rows:
        raise ValueError(
            f"Animation '{spec.anim_id}' selects row {row_idx}, but the sheet only has {spec.rows} rows"
        )
    if col_idx < 0 or col_idx >= spec.cols:
        raise ValueError(
            f"Animation '{spec.anim_id}' selects column {col_idx}, but the sheet only has {spec.cols} columns"
        )

    cell_w = img.width // spec.cols
    cell_h = img.height // spec.rows
    left = col_idx * cell_w
    top = row_idx * cell_h
    return img.crop((left, top, left + cell_w, top + cell_h))


def _write_faces(specs: list[IdleFaceSpec], output_dir: Path) -> list[Path]:
    by_prefix: dict[str, dict[str, IdleFaceSpec]] = {}
    for spec in specs:
        by_prefix.setdefault(spec.prefix, {})
        if spec.direction in by_prefix[spec.prefix]:
            raise ValueError(
                f"Duplicate idle entry for prefix '{spec.prefix}' direction '{spec.direction}'"
            )
        by_prefix[spec.prefix][spec.direction] = spec

    output_paths: list[Path] = []
    for prefix, faces in by_prefix.items():
        missing = [direction for direction in DIRECTIONS if direction not in faces]
        if missing:
            raise ValueError(
                f"Idle faces for '{prefix}' are incomplete, missing: {', '.join(missing)}"
            )

        for direction in DIRECTIONS:
            spec = faces[direction]
            img_path = _resolve_asset_path(spec.img_path)
            if not img_path.exists():
                raise FileNotFoundError(f"Spritesheet image not found: {img_path}")

            with Image.open(img_path) as img:
                rgba = img.convert("RGBA")
                frame = _crop_frame(rgba, spec)
                out_path = output_dir / f"{prefix}_{direction}.png"
                out_path.parent.mkdir(parents=True, exist_ok=True)
                frame.save(out_path)
                output_paths.append(out_path)

    return output_paths


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="idle_faces_extractor.py",
        description="Extract idle facing frames from an .anim.ron spritesheet definition.",
    )
    parser.add_argument("anim_ron", type=Path, help="Path to the input .anim.ron file")
    parser.add_argument(
        "output_dir",
        type=Path,
        nargs="?",
        help="Directory where output PNGs should be written (default: alongside the input file)",
    )
    args = parser.parse_args()

    input_path = args.anim_ron
    if not input_path.exists():
        print(f"input file does not exist: {input_path}", file=sys.stderr)
        return 2

    output_dir = args.output_dir if args.output_dir is not None else input_path.parent

    text = input_path.read_text(encoding="utf-8")
    specs = _parse_idle_face_specs(text)
    if not specs:
        print(f"no idle entries were found in {input_path}", file=sys.stderr)
        return 1

    try:
        output_paths = _write_faces(specs, output_dir)
    except (FileNotFoundError, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 1

    for path in output_paths:
        print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
