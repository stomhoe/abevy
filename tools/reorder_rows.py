#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TypedDict, cast

from anim_format import (
    extract_anim_blocks,
    format_clip_line,
    get_field_value,
    parse_clip_line,
    parse_header,
    parse_text_value,
)

"""
Reorder spritesheet rows and rewrite matching row targets in an .anim file.

The script expects a spritesheet animation definition with one or more animation
blocks. It reorders the sheet's row bands according to the requested order,
then remaps every row clip target so it continues to point at the same visual
row after the shuffle.

Example:
    reorder_rows.py assets/ron/sprite/animation/being/npc/animal/tiama.anim \
        --order 2,3,0,1

This moves the first two rows to the end while preserving their internal order.
"""

try:
    from PIL import Image  # pyright: ignore[reportMissingImports]
except ImportError as exc:  # pragma: no cover - local utility
    raise SystemExit("Pillow is required: pip install pillow") from exc


ROOT_DIR = Path(__file__).resolve().parents[1]
ASSETS_DIR = ROOT_DIR / "assets"


@dataclass(frozen=True)
class AnimRecord:
    block: str
    anim_id: str
    img_path: str
    rows: int
    cols: int


class AnimClip(TypedDict):
    target: int
    is_row: bool
    partial: tuple[int, int] | None
    start_frame: int | None


def _parse_records(text: str) -> list[AnimRecord]:
    records: list[AnimRecord] = []
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
        records.append(
            AnimRecord(
                block=block,
                anim_id=header.anim_id,
                img_path=img_path,
                rows=int(rows_cols_parts[0]),
                cols=int(rows_cols_parts[1]),
            )
        )
    return records


def _resolve_asset_path(img_path: str) -> Path:
    path = Path(img_path)
    if path.is_absolute():
        return path
    return ASSETS_DIR / path


def _parse_order(order_text: str, rows: int) -> list[int]:
    try:
        order = [int(part.strip()) for part in order_text.split(",") if part.strip() != ""]
    except ValueError as exc:
        raise ValueError(f"invalid row order: {order_text!r}") from exc

    if len(order) != rows:
        raise ValueError(f"row order length {len(order)} does not match row count {rows}")
    if sorted(order) != list(range(rows)):
        raise ValueError(f"row order must be a permutation of 0..{rows - 1}: {order}")
    return order


def _row_heights(height: int, rows: int) -> list[int]:
    base = height // rows
    heights = [base for _ in range(rows)]
    heights[-1] = height - base * (rows - 1)
    return heights


def _reorder_rows(img: Image.Image, order: list[int], rows: int) -> Image.Image:
    rgba = img.convert("RGBA")
    heights = _row_heights(rgba.height, rows)
    row_tops: list[int] = []
    y = 0
    for h in heights:
        row_tops.append(y)
        y += h

    out = Image.new("RGBA", rgba.size, (0, 0, 0, 0))
    out_y = 0
    for old_row in order:
        row_h = heights[old_row]
        src_y = row_tops[old_row]
        band = rgba.crop((0, src_y, rgba.width, src_y + row_h))
        out.paste(band, (0, out_y), band)
        out_y += row_h

    return out


def _remap_targets(block: str, mapping: list[int]) -> str:
    lines = block.splitlines()
    out: list[str] = []
    in_clips = False
    depth = 0

    for line in lines:
        stripped = line.strip()
        if stripped == "clips {":
            in_clips = True
            depth = 1
            out.append(line)
            continue

        if in_clips:
            depth += line.count("{") - line.count("}")
            if depth <= 0:
                in_clips = False
                out.append(line)
                continue
            if stripped.startswith(("row ", "col ", "column ", "clip ")):
                clip = cast(AnimClip, parse_clip_line(stripped))
                if clip["is_row"]:
                    target = int(clip["target"])
                    if target < 0 or target >= len(mapping):
                        raise ValueError(f"row target {target} is out of bounds for mapping size {len(mapping)}")
                    clip["target"] = mapping[target]
                out.append(format_clip_line(clip, indent=line[: len(line) - len(line.lstrip())]))
                continue

        out.append(line)

    return "\n".join(out)


def _rewrite_anim_file(input_path: Path, order: list[int], dry_run: bool = False) -> None:
    text = input_path.read_text(encoding="utf-8")
    records = _parse_records(text)
    if not records:
        raise ValueError(f"no animation records found in {input_path}")

    unique_pairs = {(rec.img_path, rec.rows) for rec in records}
    if len(unique_pairs) != 1:
        raise ValueError(
            f"expected one image/row layout in {input_path}, found: "
            + ", ".join(f"{img_path} ({rows} rows)" for img_path, rows in sorted(unique_pairs))
        )

    img_path, rows = next(iter(unique_pairs))
    if rows != len(order):
        raise ValueError(f"row order length {len(order)} does not match animation row count {rows}")

    mapping = [0 for _ in range(rows)]
    for new_idx, old_idx in enumerate(order):
        mapping[old_idx] = new_idx

    image_path = _resolve_asset_path(img_path)
    if not image_path.exists():
        raise FileNotFoundError(f"spritesheet image not found: {image_path}")

    with Image.open(image_path) as img:
        reordered = _reorder_rows(img, order, rows)
        if not dry_run:
            reordered.save(image_path)

    updated_text = text
    for rec in records:
        updated_block = _remap_targets(rec.block, mapping)
        updated_text = updated_text.replace(rec.block, updated_block, 1)

    if not dry_run:
        input_path.write_text(updated_text, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="reorder_rows.py",
        description="Reorder a spritesheet's rows and update matching animation targets.",
    )
    parser.add_argument("anim_ron", type=Path, help="Path to the input .anim file")
    parser.add_argument(
        "--order",
        required=True,
        help="Comma-separated list describing the new row order, e.g. 2,3,0,1",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate and report without writing files",
    )
    args = parser.parse_args()

    if not args.anim_ron.exists():
        print(f"input file does not exist: {args.anim_ron}", file=sys.stderr)
        return 2

    try:
        records = _parse_records(args.anim_ron.read_text(encoding="utf-8"))
        if not records:
            raise ValueError(f"no animation records found in {args.anim_ron}")
        rows = records[0].rows
        order = _parse_order(args.order, rows)
        _rewrite_anim_file(args.anim_ron, order, dry_run=args.dry_run)
    except (FileNotFoundError, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 1

    print(f"reordered {args.anim_ron} rows using order {order}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
