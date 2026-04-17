#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path

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


def _get_alpha_mask(img: Image.Image) -> list[list[bool]]:
    alpha = img.convert("RGBA").getchannel("A")
    return [[alpha.getpixel((x, y)) > 0 for x in range(img.width)] for y in range(img.height)]


def _connected_components(mask: list[list[bool]]) -> list[tuple[int, int, int, int]]:
    height = len(mask)
    if height == 0:
        return []
    width = len(mask[0])
    visited = [[False for _ in range(width)] for _ in range(height)]
    components: list[tuple[int, int, int, int]] = []

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

            if area > 2:
                components.append((min_x, min_y, max_x + 1, max_y + 1))

    return components


def _group_components_into_rows(
    components: list[tuple[int, int, int, int]], expected_rows: int | None = None
) -> list[list[tuple[int, int, int, int]]]:
    if not components:
        return []

    centers = [((component[1] + component[3]) / 2.0, component) for component in components]
    centers.sort(key=lambda item: item[0])

    if expected_rows is not None and expected_rows > 1:
        centroids = []
        min_center = centers[0][0]
        max_center = centers[-1][0]
        step = (max_center - min_center) / (expected_rows - 1)
        for index in range(expected_rows):
            centroids.append(min_center + step * index)

        for _ in range(10):
            groups: list[list[tuple[float, tuple[int, int, int, int]]]] = [[], ] * expected_rows
            groups = [[] for _ in range(expected_rows)]
            for center, component in centers:
                best_index = min(
                    range(expected_rows), key=lambda idx: abs(center - centroids[idx])
                )
                groups[best_index].append((center, component))
            new_centroids = []
            for idx, group in enumerate(groups):
                if group:
                    new_centroids.append(sum(center for center, _ in group) / len(group))
                else:
                    new_centroids.append(centroids[idx])
            centroids = new_centroids

        rows = [[component for _, component in group] for group in groups]
        rows = [row for row in rows if row]
        rows.sort(key=lambda row: min(component[1] for component in row))
        return rows

    diffs = [
        center2 - center1
        for (center1, _), (center2, _) in zip(centers, centers[1:])
    ]
    if not diffs:
        return [[component for _, component in centers]]

    median_diff = sorted(diffs)[len(diffs) // 2]
    threshold = max(4.0, median_diff * 4.0)

    rows: list[list[tuple[int, int, int, int]]] = []
    current_row = [centers[0][1]]
    previous_center = centers[0][0]

    for center, component in centers[1:]:
        if center - previous_center > threshold:
            rows.append(current_row)
            current_row = [component]
        else:
            current_row.append(component)
        previous_center = center

    rows.append(current_row)
    return rows


def _row_bands(img: Image.Image, expected_rows: int | None = None) -> list[tuple[int, int]]:
    mask = _get_alpha_mask(img)
    components = _connected_components(mask)
    rows = _group_components_into_rows(components, expected_rows)
    return [
        (min(component[1] for component in row), max(component[3] for component in row))
        for row in rows
    ]


def _pad_rows(img: Image.Image, extra: int, expected_rows: int | None = None) -> Image.Image:
    row_bands = _row_bands(img, expected_rows)
    if len(row_bands) < 2:
        return img.copy()

    row_heights = [y1 - y0 for y0, y1 in row_bands]
    out_height = sum(row_heights) + extra * (len(row_heights) - 1)
    out = Image.new("RGBA", (img.width, out_height), (0, 0, 0, 0))

    y = 0
    for (y0, y1), height in zip(row_bands, row_heights):
        row = img.crop((0, y0, img.width, y1))
        out.paste(row, (0, y))
        y += height + extra

    return out


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Increase vertical separation between sprite rows using pixel-island row detection."
    )
    parser.add_argument("input", help="Input PNG file")
    parser.add_argument("padding", type=int, help="Extra pixels to add between detected rows")
    parser.add_argument("output", nargs="?", help="Output PNG file (defaults to input)")
    parser.add_argument(
        "--rows",
        type=int,
        default=None,
        help="Optional expected number of rows to infer when row separation is ambiguous",
    )
    args = parser.parse_args()

    if args.padding < 0:
        print("padding must be >= 0", file=sys.stderr)
        return 2

    if args.rows is not None and args.rows <= 0:
        print("--rows must be > 0", file=sys.stderr)
        return 2

    input_path = Path(args.input)
    output_path = Path(args.output) if args.output else input_path
    img = Image.open(input_path).convert("RGBA")
    out = _pad_rows(img, args.padding, args.rows)
    out.save(output_path)

    row_bands = _row_bands(img, args.rows)
    print(
        f"{input_path}: {img.width}x{img.height} -> {len(row_bands)} rows, +{args.padding}px between rows, output {output_path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
