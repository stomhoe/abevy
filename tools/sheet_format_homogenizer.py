#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path
from statistics import median

"""
Rewrite a spritesheet so every row uses the same horizontal cell spacing.

The tool scans the alpha channel, finds the visible sprite rows, breaks each
row into connected opaque components, and then re-pastes those components onto a
new canvas using a uniform column pitch. That makes the nth sprite in one row
line up with the nth sprite in every other row, even when the original sheet had
compressed or irregular spacing in some rows.

Reescribe una hoja de sprites para que cada fila use el mismo espaciado
horizontal entre celdas.

La herramienta analiza el canal alfa, detecta las filas visibles de sprites,
divide cada fila en componentes opacos conectados y vuelve a pegar esos
componentes en un lienzo nuevo usando un paso de columna uniforme. Asi el
sprite n de una fila queda alineado con el sprite n de cualquier otra fila,
aunque la hoja original tuviera espaciado irregular o filas mas comprimidas.
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


def _load_mask(img: Image.Image) -> list[list[bool]]:
    alpha = img.convert("RGBA").getchannel("A")
    alpha_data = alpha.load()
    return [
        [alpha_data[x, y] > 0 for x in range(img.width)]
        for y in range(img.height)
    ]


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


def _row_bands(mask: list[list[bool]]) -> list[tuple[int, int]]:
    return _runs([any(row) for row in mask])


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


def _assign_components_to_rows(
    components: list[tuple[int, int, int, int, int]],
    row_bands: list[tuple[int, int]],
) -> list[list[tuple[int, int, int, int, int]]]:
    row_components: list[list[tuple[int, int, int, int, int]]] = [[] for _ in row_bands]
    for comp in components:
        _, comp_y0, _, comp_y1, _ = comp
        best_idx: int | None = None
        best_overlap = 0
        for idx, (row_y0, row_y1) in enumerate(row_bands):
            overlap = min(comp_y1, row_y1) - max(comp_y0, row_y0)
            if overlap > best_overlap:
                best_overlap = overlap
                best_idx = idx
        if best_idx is not None and best_overlap > 0:
            row_components[best_idx].append(comp)
    return row_components


def _component_center_x(comp: tuple[int, int, int, int, int]) -> float:
    x0, _, x1, _, _ = comp
    return (x0 + x1) / 2.0


def _component_center_y(comp: tuple[int, int, int, int, int]) -> float:
    _, y0, _, y1, _ = comp
    return (y0 + y1) / 2.0


def _group_components_into_rows(
    components: list[tuple[int, int, int, int, int]],
) -> list[list[tuple[int, int, int, int, int]]]:
    if not components:
        return []

    sorted_components = sorted(components, key=_component_center_y)
    heights = [y1 - y0 for _, y0, _, y1, _ in sorted_components]
    row_gap_threshold = max(2, int(round(median(heights) * 0.5)))

    rows: list[list[tuple[int, int, int, int, int]]] = [[sorted_components[0]]]
    for comp in sorted_components[1:]:
        curr_row = rows[-1]
        prev_center_y = _component_center_y(curr_row[-1])
        curr_center_y = _component_center_y(comp)
        if curr_center_y - prev_center_y > row_gap_threshold:
            rows.append([comp])
        else:
            curr_row.append(comp)

    for row in rows:
        row.sort(key=_component_center_x)
    return rows


def _estimate_pitch(
    row_components: list[list[tuple[int, int, int, int, int]]],
    fallback_cell_w: int,
) -> int:
    deltas: list[float] = []
    max_comp_w = 0
    for row in row_components:
        if not row:
            continue
        centers = sorted(_component_center_x(comp) for comp in row)
        for left, right in zip(centers, centers[1:]):
            if right > left:
                deltas.append(right - left)
        for x0, _, x1, _, _ in row:
            max_comp_w = max(max_comp_w, x1 - x0)

    pitch = int(round(median(deltas))) if deltas else max(fallback_cell_w, max_comp_w)
    return max(pitch, max_comp_w + 2)


def _estimate_row_pitch(
    row_components: list[list[tuple[int, int, int, int, int]]],
    fallback_cell_h: int,
) -> int:
    if len(row_components) < 2:
        return max(1, fallback_cell_h)

    centers = [_component_center_y(row[0]) for row in row_components if row]
    deltas = [next_center - center for center, next_center in zip(centers, centers[1:]) if next_center > center]
    max_comp_h = 0
    for row in row_components:
        for _, y0, _, y1, _ in row:
            max_comp_h = max(max_comp_h, y1 - y0)

    pitch = int(round(median(deltas))) if deltas else fallback_cell_h
    return max(1, max(pitch, max_comp_h + 2, fallback_cell_h))


def _estimate_band_pitch(bands: list[tuple[int, int]], fallback_cell: int) -> int:
    if len(bands) < 2:
        return max(1, fallback_cell)
    deltas = [next_start - start for (start, _), (next_start, _) in zip(bands, bands[1:]) if next_start > start]
    if not deltas:
        return max(1, fallback_cell)
    return max(1, int(round(median(deltas))))


def _snap_bounds_to_cells(
    bounds: tuple[int, int, int, int],
    cell_w: int,
    cell_h: int,
    max_w: int,
    max_h: int,
) -> tuple[int, int, int, int]:
    x0, y0, x1, y1 = bounds
    left = max(0, (x0 // cell_w) * cell_w)
    top = max(0, (y0 // cell_h) * cell_h)
    right = min(max_w, ((x1 + cell_w - 1) // cell_w) * cell_w)
    bottom = min(max_h, ((y1 + cell_h - 1) // cell_h) * cell_h)
    return left, top, right, bottom


def _fit_start_to_bounds(ideal_start: int, size: int, lower: int, upper: int) -> int:
    if upper <= lower:
        return lower
    max_start = max(lower, upper - size)
    return min(max(ideal_start, lower), max_start)


def main() -> int:
    if len(sys.argv) not in {2, 3}:
        print("usage: sheet_format_homogenizer.py <input.png> [output.png]", file=sys.stderr)
        return 2

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2]) if len(sys.argv) == 3 else input_path

    img = Image.open(input_path).convert("RGBA")
    cols, rows, cell_w, cell_h = _detect_grid(img)
    mask = _load_mask(img)
    components = _connected_components(mask)
    if not components:
        img.save(output_path)
        print(f"{input_path}: {img.width}x{img.height} -> empty sheet, output {output_path}")
        return 0

    row_components = _group_components_into_rows(components)
    pitch = _estimate_pitch(row_components, cell_w)
    row_pitch = _estimate_row_pitch(row_components, cell_h)
    out_w = max(1, pitch * max((len(row) for row in row_components), default=1))
    out_h = max(1, row_pitch * len(row_components))

    out = Image.new("RGBA", (out_w, out_h), (0, 0, 0, 0))
    for row_idx, row in enumerate(row_components):
        for col_idx, (x0, y0, x1, y1, _) in enumerate(row):
            cell = img.crop((x0, y0, x1, y1))
            match cell.getbbox():
                case None:
                    continue
                case cell_bbox:
                    cell = cell.crop(cell_bbox)
                    cell_x0, cell_y0, cell_x1, cell_y1 = cell_bbox
                    comp_w = cell_x1 - cell_x0
                    comp_h = cell_y1 - cell_y0
                    slot_x0 = col_idx * pitch + (pitch - comp_w) // 2
                    slot_y0 = row_idx * row_pitch + (row_pitch - comp_h) // 2
                    out.paste(cell, (slot_x0, slot_y0), cell)

    match out.getbbox():
        case None:
            pass
        case bbox:
            out = out.crop(bbox)

    out.save(output_path)

    print(
        f"{input_path}: {img.width}x{img.height} -> {cols} cols x {rows} rows, cell {cell_w}x{cell_h}"
    )
    print(
        f"normalized: {len(row_components)} sprite rows, {max((len(row) for row in row_components), default=0)} cells/row, pitch {pitch}px x {row_pitch}px, output {output_path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
