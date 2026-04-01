#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

"""
Inspect a spritesheet image and report both its coarse grid shape and the
actual occupied sprite rows found in the alpha mask. The script is designed for
sheet layouts where the visible cells are not perfectly uniform: some rows may
start later, end earlier, or contain extra gaps at the sides. To handle that, it
scans opaque pixels, groups them into connected components, and then counts the
components that belong to each horizontal band of the sheet. The result is a
more practical summary for sheets that have variable row lengths or missing edge
cells, while still preserving the simple width/height/cell-size output.

Inspecciona una hoja de sprites y muestra tanto su forma de cuadrícula aproximada
como las filas realmente ocupadas que detecta en la mascara alfa. El script esta
pensado para distribuciones donde las celdas visibles no son perfectamente
uniformes: algunas filas pueden empezar mas tarde, terminar antes o tener huecos
extra en los lados. Para resolverlo, recorre los pixeles opacos, agrupa sus
componentes conectados y cuenta los componentes que pertenecen a cada banda
horizontal de la imagen. El resultado es un resumen mas util para hojas con filas
de longitud variable o celdas faltantes en los bordes, sin perder la salida
simple de ancho, alto y tamano de celda.
"""

try:
    from PIL import Image  # pyright: ignore[reportMissingImports]
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


def _connected_components(mask: list[list[bool]]) -> list[tuple[int, int, int, int, int]]:
    height = len(mask)
    if height == 0:
        return []
    width = len(mask[0])
    visited = [[False for _ in range(width)] for _ in range(height)]
    components: list[tuple[int, int, int, int, int]] = []

    for y in range(height):
        for x in range(width):
            if not mask[y][x] or visited[y][x]:
                continue

            stack = [(x, y)]
            visited[y][x] = True
            min_x = max_x = x
            min_y = max_y = y
            area = 0

            while stack:
                curr_x, curr_y = stack.pop()
                area += 1
                if curr_x < min_x:
                    min_x = curr_x
                if curr_x > max_x:
                    max_x = curr_x
                if curr_y < min_y:
                    min_y = curr_y
                if curr_y > max_y:
                    max_y = curr_y

                for next_y in range(max(0, curr_y - 1), min(height, curr_y + 2)):
                    for next_x in range(max(0, curr_x - 1), min(width, curr_x + 2)):
                        if visited[next_y][next_x] or not mask[next_y][next_x]:
                            continue
                        visited[next_y][next_x] = True
                        stack.append((next_x, next_y))

            components.append((min_x, min_y, max_x + 1, max_y + 1, area))

    return components


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


def _row_bands(mask: list[list[bool]]) -> list[tuple[int, int]]:
    return _runs([any(row) for row in mask])


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: inspect_spritesheet.py <png>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    img = Image.open(path)
    cols, rows, cell_w, cell_h = _detect_grid(img)
    rgba = img.convert("RGBA")
    alpha = rgba.getchannel("A")
    alpha_data = alpha.load()
    mask = [
        [alpha_data[x, y] > 0 for x in range(img.width)]
        for y in range(img.height)
    ]

    row_bands = _row_bands(mask)
    components = _connected_components(mask)
    significant_components = [comp for comp in components if comp[4] >= 8]

    row_counts: list[int] = []
    for row_start, row_end in row_bands:
        count = 0
        for _, comp_y0, _, comp_y1, _ in significant_components:
            if comp_y1 <= row_start or comp_y0 >= row_end:
                continue
            count += 1
        row_counts.append(count)

    print(
        f"{path}: {img.width}x{img.height} -> {cols} cols x {rows} rows, cell {cell_w}x{cell_h}"
    )
    print(f"{path}: detected {len(row_bands)} sprite rows")
    for idx, ((row_start, row_end), count) in enumerate(zip(row_bands, row_counts)):
        print(f"  row {idx}: {count} significant cells (y={row_start}..{row_end - 1})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
