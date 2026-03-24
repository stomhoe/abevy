#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover - local utility
    raise SystemExit("Pillow is required: pip install pillow") from exc


def _runs(mask: list[bool]) -> list[tuple[int, int]]:
    runs: list[tuple[int, int]] = []
    start: int | None = None
    for idx, value in enumerate(mask):
        if value and start is None:
            start = idx
        elif not value and start is not None:
            runs.append((start, idx))
            start = None
    if start is not None:
        runs.append((start, len(mask)))
    return runs


def _detect_grid(img: Image.Image) -> tuple[int, int, int, int]:
    rgba = img.convert("RGBA")
    w, h = rgba.size
    alpha = rgba.getchannel("A")

    cols = [max(alpha.crop((x, 0, x + 1, h)).getdata()) > 0 for x in range(w)]
    rows = [max(alpha.crop((0, y, w, y + 1)).getdata()) > 0 for y in range(h)]

    col_runs = _runs(cols)
    row_runs = _runs(rows)
    if not col_runs or not row_runs:
        return 0, 0, w, h

    col_gaps = [b - a for (_, a), (b, _) in zip(col_runs, col_runs[1:])]
    row_gaps = [b - a for (_, a), (b, _) in zip(row_runs, row_runs[1:])]

    cell_w = min((end - start for start, end in col_runs), default=w)
    cell_h = min((end - start for start, end in row_runs), default=h)
    cols_n = len(col_runs)
    rows_n = len(row_runs)

    if len(col_runs) > 1:
        # Prefer a regular grid if the image has consistent gaps.
        span_w = col_runs[-1][1] - col_runs[0][0]
        avg_gap_w = sum(col_gaps) / len(col_gaps)
        if avg_gap_w > 0:
            cell_w = round((span_w - avg_gap_w * (cols_n - 1)) / cols_n)
    if len(row_runs) > 1:
        span_h = row_runs[-1][1] - row_runs[0][0]
        avg_gap_h = sum(row_gaps) / len(row_gaps)
        if avg_gap_h > 0:
            cell_h = round((span_h - avg_gap_h * (rows_n - 1)) / rows_n)

    return cols_n, rows_n, cell_w, cell_h


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: inspect_spritesheet.py <png>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    img = Image.open(path)
    cols, rows, cell_w, cell_h = _detect_grid(img)
    print(f"{path}: {img.width}x{img.height} -> {cols} cols x {rows} rows, cell {cell_w}x{cell_h}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
