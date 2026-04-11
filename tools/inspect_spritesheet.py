#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path
from typing import TypedDict, cast

from anim_format import (
    extract_anim_blocks,
    get_field_value,
    parse_clip_line,
    parse_header,
    parse_text_value,
)

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


class AnimClip(TypedDict):
    target: int
    is_row: bool
    partial: tuple[int, int] | None
    start_frame: int | None


class AnimEntry(TypedDict):
    id: str
    rows: int
    cols: int
    img_path: str
    clips: list[AnimClip]


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


def _row_column_occupancy(
    mask: list[list[bool]],
    cols: int,
    row_bands: list[tuple[int, int]],
) -> list[list[bool]]:
    if cols <= 0:
        return [[] for _ in row_bands]

    height = len(mask)
    width = len(mask[0]) if height > 0 else 0
    occupancy: list[list[bool]] = []
    for row_start, row_end in row_bands:
        row_occ: list[bool] = []
        for col_idx in range(cols):
            col_start = round(col_idx * width / cols)
            col_end = round((col_idx + 1) * width / cols)
            occupied = False
            for y in range(row_start, row_end):
                if occupied:
                    break
                for x in range(col_start, col_end):
                    if mask[y][x]:
                        occupied = True
                        break
            row_occ.append(occupied)
        occupancy.append(row_occ)
    return occupancy


def _parse_anim_entry(entry: str) -> AnimEntry:
    header = parse_header(entry)
    img_path = get_field_value(entry, "img_path")
    rows_cols = get_field_value(entry, "rows_cols")
    if img_path is None or rows_cols is None:
        raise ValueError("could not parse animation entry")
    img_path = parse_text_value(img_path)

    rows_cols_parts = rows_cols.replace(",", " ").replace("(", " ").replace(")", " ").split()
    if len(rows_cols_parts) != 2:
        raise ValueError("could not parse animation entry")

    clips: list[AnimClip] = []
    in_clips = False
    depth = 0
    for line in entry.splitlines():
        stripped = line.strip()
        if stripped == "clips {":
            in_clips = True
            depth = 1
            continue
        if in_clips:
            depth += line.count("{") - line.count("}")
            if depth <= 0:
                break
            if not stripped.startswith(("row ", "col ", "column ", "clip ")):
                continue
            clip = cast(AnimClip, parse_clip_line(stripped))
            partial = clip["partial"]
            start_frame = clip["start_frame"]
            clips.append({
                "target": int(clip["target"]),
                "is_row": bool(clip["is_row"]),
                "partial": None if partial is None else (int(partial[0]), int(partial[1])),
                "start_frame": None if start_frame is None else int(start_frame),
            })

    return {
        "id": header.anim_id,
        "rows": int(rows_cols_parts[0]),
        "cols": int(rows_cols_parts[1]),
        "img_path": img_path,
        "clips": clips,
    }


def _validate_anim_against_sheet(
    anim_path: Path,
    anim_source: str,
    sheet_cols: int,
    sheet_rows: int,
    row_bands: list[tuple[int, int]],
    row_occupancy: list[list[bool]],
) -> None:
    # LLM guidance: when this script is run with both a PNG and an anim file, treat the PNG as
    # the source of truth for the actual sprite layout and use the detected alpha mask to verify
    # the animation metadata. The goal is not to guess artistic intent, but to catch concrete
    # mistakes such as wrong row/column counts, clip targets that exceed the sheet, or a row clip
    # whose `partial` range disagrees with the occupied columns on a trimmed row. If a row is fully
    # occupied, do not invent a partial range just because the clip is missing one; only report
    # issues when the file clearly conflicts with the sheet.
    try:
        entries: list[AnimEntry] = [_parse_anim_entry(entry) for entry in extract_anim_blocks(anim_source)]
    except ValueError as exc:
        print(f"{anim_path}: could not parse anim file: {exc}")
        return

    print(f"{anim_path}: validating {len(entries)} animation entries against the spritesheet")
    for entry in entries:
        anim_id = entry["id"]
        rows = entry["rows"]
        cols = entry["cols"]
        img_path = entry["img_path"]
        clips: list[AnimClip] = entry["clips"]

        problems: list[str] = []
        if rows != sheet_rows or cols != sheet_cols:
            problems.append(
                f"rows_cols {rows}x{cols} does not match detected sheet {sheet_rows}x{sheet_cols}"
            )

        for clip in clips:
            if not clip["is_row"]:
                continue

            target = int(clip["target"])
            frame_len = cols
            if target >= len(row_bands):
                problems.append(
                    f"row target {target} is out of bounds for detected row count {len(row_bands)}"
                )
                continue

            occupied_cols = [idx for idx, occupied in enumerate(row_occupancy[target]) if occupied]
            expected_partial: tuple[int, int] | None
            if occupied_cols:
                expected_partial = (occupied_cols[0], occupied_cols[-1])
            else:
                expected_partial = None

            partial = clip["partial"]
            if partial is not None:
                start, end = partial
                if start > end:
                    problems.append(
                        f"row {target} partial {start}..{end} is invalid because start > end"
                    )
                    continue
                if end >= frame_len:
                    problems.append(
                        f"row {target} partial {start}..{end} exceeds frame bound {frame_len}"
                    )
                    continue

            if expected_partial is not None and not all(row_occupancy[target]) and clip["start_frame"] is not None:
                expected_start, expected_end = expected_partial
                if partial is None:
                    problems.append(
                        f"row {target} is trimmed to cols {expected_start}..{expected_end} but move clip has no partial"
                    )
                elif partial != expected_partial:
                    problems.append(
                        f"row {target} move clip partial {partial[0]}..{partial[1]} does not match occupied cols {expected_start}..{expected_end}"
                    )

        if problems:
            print(f"  {anim_id}: {img_path}")
            for problem in problems:
                print(f"    - {problem}")
        else:
            print(f"  {anim_id}: ok")


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print("usage: inspect_spritesheet.py <png> [anim]", file=sys.stderr)
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
    row_occupancy = _row_column_occupancy(mask, cols, row_bands)
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
    for idx, ((row_start, row_end), count, occ) in enumerate(zip(row_bands, row_counts, row_occupancy)):
        occupied_cols = [col_idx for col_idx, is_occupied in enumerate(occ) if is_occupied]
        if occupied_cols:
            span = f"{occupied_cols[0]}..{occupied_cols[-1]}"
        else:
            span = "none"
        occ_mask = "".join("1" if is_occupied else "0" for is_occupied in occ)
        print(
            f"  row {idx}: {count} significant cells, cols {span} ({occ_mask}) "
            f"(y={row_start}..{row_end - 1})"
        )

    if len(sys.argv) == 3:
        anim_path = Path(sys.argv[2])
        _validate_anim_against_sheet(anim_path, anim_path.read_text(), cols, rows, row_bands, row_occupancy)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
