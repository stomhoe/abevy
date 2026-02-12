#!/usr/bin/env python3

import argparse
import random
import sys

from PIL import (  # pyright: ignore[reportMissingImports]
    Image,
    ImageChops,
    ImageDraw,
    ImageFilter,
)

TILE_SIZE = 128
GRID_PROB = 0.7      # % of patches aligned to tile grid
JITTER = 6           # pixel jitter when grid-aligned
COVERAGE_BIAS = 0.6  # % of patches biased toward low-coverage tiles


def random_transform(img):
    r = random.random()
    if r < 0.2:
        return img.transpose(Image.FLIP_LEFT_RIGHT)
    elif r < 0.4:
        return img.transpose(Image.FLIP_TOP_BOTTOM)
    elif r < 0.6:
        return img.transpose(Image.ROTATE_90)
    elif r < 0.8:
        return img.transpose(Image.ROTATE_180)
    else:
        return img.transpose(Image.ROTATE_270)


def feather_mask(w, h, feather):
    feather = max(0, min(feather, min(w, h) // 2))
    mask = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(mask)
    left = feather
    top = feather
    right = w - feather - 1
    bottom = h - feather - 1
    if right < left or bottom < top:
        draw.rectangle((0, 0, w - 1, h - 1), fill=255)
    else:
        draw.rectangle((left, top, right, bottom), fill=255)
    if feather > 0:
        mask = mask.filter(ImageFilter.GaussianBlur(feather))
    return mask


def paste_feathered(dest, patch, x, y, feather):
    if feather <= 0:
        dest.paste(patch, (x, y))
        return
    mask = feather_mask(patch.width, patch.height, feather)
    dest.paste(patch, (x, y), mask)


def seam_heal(big, tiles, seam_size, jitter, feather, passes):
    big_w, big_h = big.size
    seam_size = max(2, seam_size)
    for _ in range(passes):
        # Vertical seams
        for tx in range(1, tiles):
            x = tx * TILE_SIZE
            rx = max(0, x - seam_size)
            rw = min(seam_size * 2, big_w - rx)
            rh = random.randint(seam_size * 2, min(big_h, seam_size * 6))
            ry = random.randint(0, big_h - rh)
            src_x = max(0, min(big_w - rw, rx + random.randint(-jitter, jitter)))
            src_y = max(0, min(big_h - rh, ry + random.randint(-jitter, jitter)))
            patch = big.crop((src_x, src_y, src_x + rw, src_y + rh))
            paste_feathered(big, patch, rx, ry, feather)
        # Horizontal seams
        for ty in range(1, tiles):
            y = ty * TILE_SIZE
            ry = max(0, y - seam_size)
            rh = min(seam_size * 2, big_h - ry)
            rw = random.randint(seam_size * 2, min(big_w, seam_size * 6))
            rx = random.randint(0, big_w - rw)
            src_x = max(0, min(big_w - rw, rx + random.randint(-jitter, jitter)))
            src_y = max(0, min(big_h - rh, ry + random.randint(-jitter, jitter)))
            patch = big.crop((src_x, src_y, src_x + rw, src_y + rh))
            paste_feathered(big, patch, rx, ry, feather)


def main():
    parser = argparse.ArgumentParser(
        description="Tile an image and aggressively break repetition using tile-aware randomization."
    )

    parser.add_argument("input", help="Input image path (32x32 tile recommended)")
    parser.add_argument("output", help="Output image path")

    parser.add_argument("--tiles", type=int, default=2,
                        help="Number of tiles per axis (default: 8)")

    parser.add_argument("--min-flips", type=int, default=20,
                        help="Minimum number of patch operations")
    parser.add_argument("--max-flips", type=int, default=20,
                        help="Maximum number of patch operations")

    parser.add_argument("--min-patch", type=int, default=30,
                        help="Minimum patch size in pixels")
    parser.add_argument("--max-patch", type=int, default=120,
                        help="Maximum patch size in pixels")

    parser.add_argument("--feather", type=int, default=0,
                        help="Feather size (px) when pasting patches (default: 12)")

    parser.add_argument("--seam-heal", type=int, default=0,
                        help="Number of seam-heal passes (default: 2)")
    parser.add_argument("--seam-size", type=int, default=0,
                        help="Half-width of seam healing band in pixels (default: 12)")
    parser.add_argument("--seam-jitter", type=int, default=0,
                        help="Pixel jitter for seam healing source (default: 8)")

    parser.add_argument("--noise", type=float, default=0.0,
                        help="Optional noise strength to break repetition (0 disables)")

    parser.add_argument("--seed", type=int, default=None,
                        help="Random seed (reproducible output)")

    args = parser.parse_args()

    if args.seed is not None:
        random.seed(args.seed)

    try:
        img = Image.open(args.input).convert("RGBA")
    except Exception as e:
        print(f"Failed to load image: {e}", file=sys.stderr)
        sys.exit(1)

    w, h = img.size
    if w != TILE_SIZE or h != TILE_SIZE:
        print(f"Warning: input tile is {w}x{h}, expected {TILE_SIZE}x{TILE_SIZE}")

    tiles = args.tiles
    big_w = w * tiles
    big_h = h * tiles

    big = Image.new("RGBA", (big_w, big_h))

    # --- Coverage tracking (tile space)
    cov_w = big_w // TILE_SIZE
    cov_h = big_h // TILE_SIZE
    coverage = [[0 for _ in range(cov_w)] for _ in range(cov_h)]

    # --- Initial tiling with per-tile variation
    for ty in range(tiles):
        for tx in range(tiles):
            tile = random_transform(img.copy())
            big.paste(tile, (tx * w, ty * h))

    flips = random.randint(args.min_flips, args.max_flips)

    for _ in range(flips):
        # --- Choose patch origin
        use_coverage = random.random() < COVERAGE_BIAS
        use_grid = random.random() < GRID_PROB

        if use_coverage:
            flat = [(coverage[y][x], x, y)
                    for y in range(cov_h)
                    for x in range(cov_w)]
            flat.sort()
            _, cx, cy = random.choice(flat[:len(flat) // 3])
            base_x = cx * TILE_SIZE
            base_y = cy * TILE_SIZE
        else:
            base_x = random.randint(0, big_w - args.min_patch)
            base_y = random.randint(0, big_h - args.min_patch)

        if use_grid:
            rx = base_x + random.randint(-JITTER, JITTER)
            ry = base_y + random.randint(-JITTER, JITTER)
        else:
            rx = base_x
            ry = base_y

        rx = max(0, min(big_w - args.min_patch, rx))
        ry = max(0, min(big_h - args.min_patch, ry))

        rw = random.randint(
            args.min_patch,
            min(args.max_patch, big_w - rx)
        )
        rh = random.randint(
            args.min_patch,
            min(args.max_patch, big_h - ry)
        )

        patch = big.crop((rx, ry, rx + rw, ry + rh))
        patch = random_transform(patch)
        paste_feathered(big, patch, rx, ry, args.feather)

        # --- Update coverage
        cx0 = rx // TILE_SIZE
        cy0 = ry // TILE_SIZE
        cx1 = min(cov_w - 1, (rx + rw) // TILE_SIZE)
        cy1 = min(cov_h - 1, (ry + rh) // TILE_SIZE)

        for cy in range(cy0, cy1 + 1):
            for cx in range(cx0, cx1 + 1):
                coverage[cy][cx] += 1

    # --- Automated seam healing pass
    if args.seam_heal > 0:
        seam_heal(big, tiles, args.seam_size, args.seam_jitter, args.feather, args.seam_heal)

    # --- Optional noise to reduce repetition
    if args.noise and args.noise > 0:
        noise = Image.effect_noise((big_w, big_h), args.noise).convert("L")
        noise = Image.merge("RGBA", (noise, noise, noise, Image.new("L", (big_w, big_h), 64)))
        big = Image.alpha_composite(big, noise)

    # --- Save output
    big.save(args.output)


if __name__ == "__main__":
    main()
