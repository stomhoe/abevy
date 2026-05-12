use rand::RngExt;
use rand::rngs::StdRng as Pcg64Mcg;

pub fn carve_corridor_horizontal_floor(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    carve_margin: Option<usize>,
    corridor_width: Option<usize>,
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    y_base: i32,
    x0: i32,
    x1: i32,
) {
    let carve_margin = carve_margin.unwrap_or(1).clamp(0, 32);
    let corridor_width = corridor_width.unwrap_or(1).clamp(1, 16);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

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
    carve_margin: Option<usize>,
    corridor_width: Option<usize>,
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    x_base: i32,
    y0: i32,
    y1: i32,
) {
    let carve_margin = carve_margin.unwrap_or(1).clamp(0, 32);
    let corridor_width = corridor_width.unwrap_or(1).clamp(1, 16);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

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
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    y_base: i32,
    x_start: i32,
    x_end: i32,
) {
    let corridor_radius = corridor_radius.clamp(1, 8);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

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
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    x_base: i32,
    y_start: i32,
    y_end: i32,
) {
    let corridor_radius = corridor_radius.clamp(1, 8);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

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

fn stamp_corridor_disk_u8(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    corridor_map: &mut [bool],
    x: i32,
    y: i32,
    corridor_radius: i32,
    floor_kind: u8,
    should_carve_tile: &mut impl FnMut(i32, i32) -> bool,
) {
    let radius_sq = corridor_radius * corridor_radius;
    for dy in -corridor_radius..=corridor_radius {
        for dx in -corridor_radius..=corridor_radius {
            if dx * dx + dy * dy > radius_sq {
                continue;
            }

            let xx = x + dx;
            let yy = y + dy;
            if xx < 0 || yy < 0 {
                continue;
            }

            let xx = xx as usize;
            let yy = yy as usize;
            if xx >= tile_width || yy >= tile_height {
                continue;
            }

            if !should_carve_tile(xx as i32, yy as i32) {
                continue;
            }

            let idx = yy * tile_width + xx;
            floor_map[idx] = floor_kind;
            corridor_map[idx] = true;
        }
    }
}

pub fn carve_corridor_polyline_typed(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    corridor_map: &mut [bool],
    corridor_radius: i32,
    path: &[(i32, i32)],
    floor_kind: u8,
    mut should_carve_tile: impl FnMut(i32, i32) -> bool,
) {
    let corridor_radius = corridor_radius.clamp(1, 8);
    let Some(_) = path.get(1) else {
        return;
    };

    for window in path.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let steps = (dx.max(dy) * 2).max(1) as usize;

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = (x0 as f32 + (x1 - x0) as f32 * t).round() as i32;
            let y = (y0 as f32 + (y1 - y0) as f32 * t).round() as i32;
            stamp_corridor_disk_u8(
                floor_map,
                tile_width,
                tile_height,
                corridor_map,
                x,
                y,
                corridor_radius,
                floor_kind,
                &mut should_carve_tile,
            );
        }
    }
}

fn set_floor_tile_u8(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    floor_kind: u8,
) {
    if x < 0 || y < 0 {
        return;
    }
    let x = x as usize;
    let y = y as usize;
    if x >= tile_width || y >= tile_height {
        return;
    }
    floor_map[y * tile_width + x] = floor_kind;
}

fn carve_line_u8(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    floor_kind: u8,
) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        set_floor_tile_u8(floor_map, tile_width, tile_height, x, y, floor_kind);
        if x == x1 && y == y1 {
            break;
        }

        let err2 = err * 2;
        if err2 > -dy {
            err -= dy;
            x += sx;
        }
        if err2 < dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn carve_room_rectangle_typed(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    floor_kind: u8,
) {
    let w = w.max(0);
    let h = h.max(0);
    for yy in 0..h {
        for xx in 0..w {
            set_floor_tile_u8(floor_map, tile_width, tile_height, x + xx, y + yy, floor_kind);
        }
    }
}

pub fn carve_room_ellipse_typed(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    floor_kind: u8,
) {
    let cx = x + w / 2;
    let cy = y + h / 2;
    let rx = (w as f32 / 2.0).max(1.0);
    let ry = (h as f32 / 2.0).max(1.0);
    let rx_sq = rx * rx;
    let ry_sq = ry * ry;

    for yy in 0..h {
        for xx in 0..w {
            let dx = (x + xx) as f32 - cx as f32;
            let dy = (y + yy) as f32 - cy as f32;
            if (dx * dx) / rx_sq + (dy * dy) / ry_sq <= 1.0 {
                set_floor_tile_u8(floor_map, tile_width, tile_height, x + xx, y + yy, floor_kind);
            }
        }
    }
}

pub fn carve_room_triangle_vertices_typed(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    v0: (i32, i32),
    v1: (i32, i32),
    v2: (i32, i32),
    floor_kind: u8,
) {
    let (x0, y0) = v0;
    let (x1, y1) = v1;
    let (x2, y2) = v2;

    let min_x = x0.min(x1).min(x2).max(0) as usize;
    let max_x = x0.max(x1).max(x2).min((tile_width - 1) as i32) as usize;
    let min_y = y0.min(y1).min(y2).max(0) as usize;
    let max_y = y0.max(y1).max(y2).min((tile_height - 1) as i32) as usize;

    if min_x > max_x || min_y > max_y {
        return;
    }

    fn sign(px: i32, py: i32, x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
        (px - x2) * (y1 - y2) - (x1 - x2) * (py - y2)
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as i32;
            let py = y as i32;

            let d1 = sign(px, py, x0, y0, x1, y1);
            let d2 = sign(px, py, x1, y1, x2, y2);
            let d3 = sign(px, py, x2, y2, x0, y0);

            let has_neg = (d1 < 0) || (d2 < 0) || (d3 < 0);
            let has_pos = (d1 > 0) || (d2 > 0) || (d3 > 0);

            if !(has_neg && has_pos) {
                floor_map[y * tile_width + x] = floor_kind;
            }
        }
    }
}

pub fn carve_room_trapezoid_typed(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    v0: (i32, i32),
    v1: (i32, i32),
    v2: (i32, i32),
    v3: (i32, i32),
    floor_kind: u8,
) {
    carve_room_triangle_vertices_typed(floor_map, tile_width, tile_height, v0, v1, v2, floor_kind);
    carve_room_triangle_vertices_typed(floor_map, tile_width, tile_height, v0, v2, v3, floor_kind);
}

pub fn carve_room_regular_polygon_typed(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    sides: i32,
    rotation_deg: f32,
    floor_kind: u8,
) {
    let w = w.max(0);
    let h = h.max(0);
    if w == 0 || h == 0 {
        return;
    }

    let sides = sides.max(3);
    let cx = x + w / 2;
    let cy = y + h / 2;
    let radius = (w.min(h) / 2).max(1) as f32;
    let rotation_rad = rotation_deg.to_radians();
    let step = std::f32::consts::TAU / sides as f32;

    let mut vertices: Vec<(i32, i32)> = Vec::with_capacity(sides as usize);
    for i in 0..sides {
        let angle = rotation_rad + step * i as f32;
        let vx = cx as f32 + radius * angle.cos();
        let vy = cy as f32 + radius * angle.sin();
        vertices.push((vx.round() as i32, vy.round() as i32));
    }

    let center = (cx, cy);
    for i in 0..sides as usize {
        let v1 = vertices[i];
        let v2 = vertices[(i + 1) % vertices.len()];
        carve_room_triangle_vertices_typed(floor_map, tile_width, tile_height, center, v1, v2, floor_kind);
    }
}

pub fn carve_room_pentacle(
    floor_map: &mut [u8],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    wall_floor_kind: u8,
    pentagram_floor_kind: u8,
) {
    let w = w.max(0);
    let h = h.max(0);
    if w == 0 || h == 0 {
        return;
    }

    carve_room_ellipse_typed(
        floor_map,
        tile_width,
        tile_height,
        x,
        y,
        w,
        h,
        wall_floor_kind,
    );

    let cx = x + w / 2;
    let cy = y + h / 2;
    let radius = (w.min(h).saturating_sub(2) / 2).max(2) as f32;
    let angles = [90.0_f32, 162.0, 234.0, 306.0, 18.0];

    let mut vertices = [(0, 0); 5];
    for (i, angle_deg) in angles.iter().copied().enumerate() {
        let angle = angle_deg.to_radians();
        let vx = cx as f32 + radius * angle.cos();
        let vy = cy as f32 + radius * angle.sin();
        vertices[i] = (vx.round() as i32, vy.round() as i32);
    }

    let star_order = [0usize, 2, 4, 1, 3];
    for i in 0..star_order.len() {
        let (sx, sy) = vertices[star_order[i]];
        let (ex, ey) = vertices[star_order[(i + 1) % star_order.len()]];
        carve_line_u8(
            floor_map,
            tile_width,
            tile_height,
            sx,
            sy,
            ex,
            ey,
            pentagram_floor_kind,
        );
    }
}

pub fn carve_corridor_horizontal_typed(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [u8],
    floor_kind: u8,
    corridor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    corridor_radius: i32,
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    y_base: i32,
    x_start: i32,
    x_end: i32,
) {
    let corridor_radius = corridor_radius.clamp(1, 8);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

    let (sx, ex) = if x_start <= x_end {(x_start, x_end)} else {(x_end, x_start)};
    for x in sx..=ex {
        for dy in -corridor_radius..=corridor_radius {
            let mut yy = y_base + dy;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                yy += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if x >= 0 && (x as usize) < tile_width && yy >= 0 && (yy as usize) < tile_height {
                let idx = (yy as usize) * tile_width + (x as usize);
                floor_map[idx] = floor_kind;
                corridor_map[idx] = true;
            }
        }
    }
}

pub fn carve_corridor_vertical_typed(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [u8],
    floor_kind: u8,
    corridor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    corridor_radius: i32,
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    x_base: i32,
    y_start: i32,
    y_end: i32,
) {
    let corridor_radius = corridor_radius.clamp(1, 8);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

    let (sy, ey) = if y_start <= y_end {(y_start, y_end)} else {(y_end, y_start)};
    for y in sy..=ey {
        for dx in -corridor_radius..=corridor_radius {
            let mut xx = x_base + dx;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                xx += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if xx >= 0 && (xx as usize) < tile_width && y >= 0 && (y as usize) < tile_height {
                let idx = (y as usize) * tile_width + (xx as usize);
                floor_map[idx] = floor_kind;
                corridor_map[idx] = true;
            }
        }
    }
}