#!/usr/bin/env python3
"""Trim transparent margins from every spritesheet frame and repack the sheet.

The input grid order is preserved. Each source cell is isolated before its
opaque bounding box is calculated, so pixels are never allowed to migrate into
an adjacent animation frame. The output uses one uniform cell size, which is
required by the runtime atlas and makes each frame use more of its icon slot.

Example:
    python tools/repack_spritesheet_frames.py \
        assets/texture/being/npc/wolf.png --rows 4 --cols 4
"""

from __future__ import annotations

import argparse
from pathlib import Path

try:
    from PIL import Image  # pyright: ignore[reportMissingImports]
except ImportError as exc:  # pragma: no cover - local utility
    raise SystemExit("Pillow is required: pip install pillow") from exc


def _cell_bounds(length: int, index: int, count: int) -> tuple[int, int]:
    """Return non-overlapping bounds that cover the whole source dimension."""
    return length * index // count, length * (index + 1) // count


def repack(
    source: Image.Image,
    rows: int,
    cols: int,
    padding: int = 0,
    alpha_threshold: int = 1,
) -> Image.Image:
    if rows < 1 or cols < 1:
        raise ValueError("rows and cols must be positive")
    if padding < 0:
        raise ValueError("padding cannot be negative")
    if not 0 <= alpha_threshold <= 255:
        raise ValueError("alpha threshold must be between 0 and 255")

    source = source.convert("RGBA")
    frames: list[tuple[Image.Image | None, int, int]] = []
    max_width = 0
    max_height = 0

    for row in range(rows):
        y0, y1 = _cell_bounds(source.height, row, rows)
        for col in range(cols):
            x0, x1 = _cell_bounds(source.width, col, cols)
            cell = source.crop((x0, y0, x1, y1))
            alpha = cell.getchannel("A")
            if alpha_threshold > 0:
                alpha = alpha.point(lambda value: 255 if value >= alpha_threshold else 0)
            bbox = alpha.getbbox()
            if bbox is None:
                frames.append((None, 0, 0))
                continue

            trimmed = cell.crop(bbox)
            max_width = max(max_width, trimmed.width)
            max_height = max(max_height, trimmed.height)
            frames.append((trimmed, trimmed.width, trimmed.height))

    if max_width == 0 or max_height == 0:
        raise ValueError("the sheet contains no visible pixels")

    cell_width = max_width + padding * 2
    cell_height = max_height + padding * 2
    output = Image.new("RGBA", (cell_width * cols, cell_height * rows), (0, 0, 0, 0))

    frame_index = 0
    for row in range(rows):
        for col in range(cols):
            frame, frame_width, frame_height = frames[frame_index]
            frame_index += 1
            if frame is None:
                continue

            # Centering keeps animation frames aligned while removing their
            # unused outer margins. The destination is always inside its own
            # cell, so no pixel island can cross a frame boundary.
            x = col * cell_width + (cell_width - frame_width) // 2
            y = row * cell_height + (cell_height - frame_height) // 2
            output.alpha_composite(frame, (x, y))

    return output


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="source PNG spritesheet")
    parser.add_argument("--rows", type=int, required=True, help="number of frame rows")
    parser.add_argument("--cols", type=int, required=True, help="number of frame columns")
    parser.add_argument("--padding", type=int, default=0, help="transparent pixels around each frame (default: 0)")
    parser.add_argument("--alpha-threshold", type=int, default=1, help="minimum alpha treated as visible (default: 1)")
    parser.add_argument("-o", "--output", type=Path, help="output PNG path; defaults to <name>.repacked.png")
    parser.add_argument("--in-place", action="store_true", help="replace the input file after successful processing")
    args = parser.parse_args()

    if args.in_place and args.output is not None:
        parser.error("--in-place and --output cannot be used together")
    if not args.input.is_file():
        parser.error(f"input file does not exist: {args.input}")

    output_path = args.input.with_name(f"{args.input.stem}.repacked.png") if args.output is None else args.output
    if args.in_place:
        output_path = args.input

    with Image.open(args.input) as source:
        output = repack(source, args.rows, args.cols, args.padding, args.alpha_threshold)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output.save(output_path, format="PNG")
    print(f"wrote {output_path} ({output.width}x{output.height})")


if __name__ == "__main__":
    main()
