#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

"""
Reorder spritesheet rows and rewrite matching row targets in an .anim.ron file.

The script expects a spritesheet animation definition with one or more animation
records. It reorders the sheet's row bands according to the requested order,
then remaps every row clip target so it continues to point at the same visual
row after the shuffle.

Example:
    reorder_rows.py assets/ron/sprite/animation/being/npc/animal/tiama.anim.ron \
        --order 2,3,0,1

This moves the first two rows to the end while preserving their internal order.
"""

try:
    from PIL import Image  # pyright: ignore[reportMissingImports]
except ImportError as exc:  # pragma: no cover - local utility
    raise SystemExit("Pillow is required: pip install pillow") from exc


ROOT_DIR = Path(__file__).resolve().parents[1]
ASSETS_DIR = ROOT_DIR / "assets"

RECORD_START_RE = re.compile(r"\(\s*id:\s*\"")
ID_RE = re.compile(r'id:\s*"([^"]+)"')
IMG_RE = re.compile(r'img_path:\s*"([^"]+)"')
ROWS_COLS_RE = re.compile(r"rows_cols:\s*\(\s*(\d+)\s*,\s*(\d+)\s*\)")
CLIP_RE = re.compile(r"\(\s*target:\s*(\d+),\s*is_row:\s*(true|false)(?P<body>.*?)\n\s*\),", re.S)


@dataclass(frozen=True)
class AnimRecord:
    block: str
    anim_id: str
    img_path: str
    rows: int
    cols: int


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


def _parse_records(text: str) -> list[AnimRecord]:
    records: list[AnimRecord] = []
    for block in _extract_record_blocks(text):
        id_match = ID_RE.search(block)
        img_match = IMG_RE.search(block)
        rows_cols_match = ROWS_COLS_RE.search(block)
        if not (id_match and img_match and rows_cols_match):
            continue
        records.append(
            AnimRecord(
                block=block,
                anim_id=id_match.group(1),
                img_path=img_match.group(1),
                rows=int(rows_cols_match.group(1)),
                cols=int(rows_cols_match.group(2)),
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
    def clip_replacer(match: re.Match[str]) -> str:
        target = int(match.group(1))
        is_row = match.group(2) == "true"
        if is_row:
            if target < 0 or target >= len(mapping):
                raise ValueError(f"row target {target} is out of bounds for mapping size {len(mapping)}")
            target = mapping[target]
        clip_text = match.group(0)
        return re.sub(
            r"(target:\s*)\d+",
            lambda m: f"{m.group(1)}{target}",
            clip_text,
            count=1,
        )

    return CLIP_RE.sub(clip_replacer, block)


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
    parser.add_argument("anim_ron", type=Path, help="Path to the input .anim.ron file")
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
