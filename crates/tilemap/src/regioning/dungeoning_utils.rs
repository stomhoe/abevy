use game_common::game_common_components::ArgsMap;
use rand::Rng;
use rand_pcg::Pcg64Mcg;

pub fn parse_arg<T: std::str::FromStr + Clone>(args: &ArgsMap, key: &str, default: T) -> T {
    args.get(key)
        .and_then(|v| v.first())
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(default)
}

pub fn parse_opt_arg<T: std::str::FromStr>(args: &ArgsMap, key: &str) -> Option<T> {
    args.get(key)
        .and_then(|v| v.first())
        .and_then(|s| s.parse::<T>().ok())
}

pub fn carve_room_rectangle(
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    for yy in 0..h {
        for xx in 0..w {
            let tx = (x + xx) as usize;
            let ty = (y + yy) as usize;
            if tx < tile_width && ty < tile_height {
                floor_map[ty * tile_width + tx] = true;
            }
        }
    }
}

pub fn carve_room_circle(
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    let cx = x + w / 2;
    let cy = y + h / 2;
    let radius = (w.min(h) / 2).max(1);
    let radius_sq = radius * radius;
    for yy in 0..h {
        for xx in 0..w {
            let dx = (x + xx) - cx;
            let dy = (y + yy) - cy;
            if dx * dx + dy * dy <= radius_sq {
                let tx = (x + xx) as usize;
                let ty = (y + yy) as usize;
                if tx < tile_width && ty < tile_height {
                    floor_map[ty * tile_width + tx] = true;
                }
            }
        }
    }
}

pub fn carve_room_triangle(
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    let top_y = y;
    let bottom_y = y + h - 1;
    let center_x = x + w / 2;
    let height = (bottom_y - top_y).max(1) as f32;
    for yy in 0..h {
        let y0 = y + yy;
        let t = (y0 - top_y) as f32 / height;
        let half_width = (w as f32 / 2.0) * (1.0 - t);
        for xx in 0..w {
            let x0 = x + xx;
            let dx = (x0 - center_x).abs() as f32;
            if dx <= half_width {
                let tx = x0 as usize;
                let ty = y0 as usize;
                if tx < tile_width && ty < tile_height {
                    floor_map[ty * tile_width + tx] = true;
                }
            }
        }
    }
}

pub fn carve_corridor_horizontal_floor(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    carve_margin: usize,
    corridor_width: usize,
    corridor_wiggle_chance: f32,
    corridor_wiggle_step_max: i32,
    y_base: i32,
    x0: i32,
    x1: i32,
) {
    let (sx, ex) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
    for x in sx..=ex {
        if x < carve_margin as i32 || x >= (tile_width - carve_margin) as i32 { continue; }
        for dy in -(corridor_width as i32)..=corridor_width as i32 {
            let mut y = y_base + dy;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                y += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if y < carve_margin as i32 || y >= (tile_height - carve_margin) as i32 { continue; }
            let x = x as usize;
            let y = y as usize;
            floor_map[y * tile_width + x] = true;
        }
    }
}

pub fn carve_corridor_vertical_floor(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    carve_margin: usize,
    corridor_width: usize,
    corridor_wiggle_chance: f32,
    corridor_wiggle_step_max: i32,
    x_base: i32,
    y0: i32,
    y1: i32,
) {
    let (sy, ey) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
    for y in sy..=ey {
        if y < carve_margin as i32 || y >= (tile_height - carve_margin) as i32 { continue; }
        for dx in -(corridor_width as i32)..=corridor_width as i32 {
            let mut x = x_base + dx;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                x += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if x < carve_margin as i32 || x >= (tile_width - carve_margin) as i32 { continue; }
            let x = x as usize;
            let y = y as usize;
            floor_map[y * tile_width + x] = true;
        }
    }
}

pub fn carve_corridor_horizontal(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    corridor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    corridor_radius: i32,
    corridor_wiggle_chance: f32,
    corridor_wiggle_step_max: i32,
    y_base: i32,
    x_start: i32,
    x_end: i32,
) {
    let (sx, ex) = if x_start <= x_end {(x_start, x_end)} else {(x_end, x_start)};
    for x in sx..=ex {
        for dy in -corridor_radius..=corridor_radius {
            let mut yy = y_base + dy;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                yy += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if x >= 0 && (x as usize) < tile_width && yy >= 0 && (yy as usize) < tile_height {
                let idx = (yy as usize) * tile_width + (x as usize);
                floor_map[idx] = true;
                corridor_map[idx] = true;
            }
        }
    }
}

pub fn carve_corridor_vertical(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    corridor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    corridor_radius: i32,
    corridor_wiggle_chance: f32,
    corridor_wiggle_step_max: i32,
    x_base: i32,
    y_start: i32,
    y_end: i32,
) {
    let (sy, ey) = if y_start <= y_end {(y_start, y_end)} else {(y_end, y_start)};
    for y in sy..=ey {
        for dx in -corridor_radius..=corridor_radius {
            let mut xx = x_base + dx;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                xx += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if xx >= 0 && (xx as usize) < tile_width && y >= 0 && (y as usize) < tile_height {
                let idx = (y as usize) * tile_width + (xx as usize);
                floor_map[idx] = true;
                corridor_map[idx] = true;
            }
        }
    }
}
