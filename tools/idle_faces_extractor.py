#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TypedDict, cast

from anim_format import extract_anim_blocks, get_field_value, parse_clip_line, parse_header, parse_text_value

"""
Extract the idle facing frames from a spritesheet animation definition.

The script reads an .anim file, finds the animations whose ids end in
_north_idle, _south_idle, _west_idle, or _east_idle, and writes one PNG per
facing direction using the common id prefix.

Usage:
    idle_faces_extractor.py <input.anim> [output_dir]
"""

try:
    from PIL import Image  # pyright: ignore[reportMissingImports]
except ImportError as exc:  # pragma: no cover - local utility
    raise SystemExit("Pillow is required: pip install pillow") from exc


ROOT_DIR = Path(__file__).resolve().parents[1]
ASSETS_DIR = ROOT_DIR / "assets"
DIRECTIONS = ("north", "south", "west", "east")
IDLE_ID_RE = re.compile(r"^(?P<prefix>.+)_(?P<direction>north|south|west|east)_idle$")


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


class IdleFaceClip(TypedDict):
    target: int
    is_row: bool
    partial: tuple[int, int] | None
    start_frame: int | None


def _parse_idle_face_specs(text: str) -> list[IdleFaceSpec]:
    specs: list[IdleFaceSpec] = []
    for block in extract_anim_blocks(text):
        header = parse_header(block)
        img_path = get_field_value(block, "img_path")
        rows_cols = get_field_value(block, "rows_cols")
        if img_path is None or rows_cols is None:
            continue
        img_path = parse_text_value(img_path)
        rows_cols_parts = rows_cols.replace(",", " ").replace("(", " ").replace(")", " ").split()
        if len(rows_cols_parts) != 2:
            continue

        clips_section = []
        in_clips = False
        depth = 0
        for line in block.splitlines():
            stripped = line.strip()
            if stripped == "clips {":
                in_clips = True
                depth = 1
                continue
            if in_clips:
                depth += line.count("{") - line.count("}")
                if depth <= 0:
                    break
                if stripped.startswith(("row ", "col ", "column ", "clip ")):
                    clips_section.append(stripped)
        if not clips_section:
            continue

        anim_id = header.anim_id
        idle_match = IDLE_ID_RE.fullmatch(anim_id)
        if idle_match is None:
            continue

        prefix = idle_match.group("prefix")
        direction = idle_match.group("direction")
        rows = int(rows_cols_parts[0])
        cols = int(rows_cols_parts[1])

        clip = cast(IdleFaceClip, parse_clip_line(clips_section[0]))
        target = int(clip["target"])
        is_row = bool(clip["is_row"])

        partial = clip["partial"]
        partial_start = partial[0] if partial is not None else 0

        start_frame = clip["start_frame"] if clip["start_frame"] is not None else 0

        specs.append(
            IdleFaceSpec(
                anim_id=anim_id,
                prefix=prefix,
                direction=direction,
                img_path=img_path,
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
        description="Extract idle facing frames from an .anim spritesheet definition.",
    )
    parser.add_argument("anim_ron", type=Path, help="Path to the input .anim file")
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
