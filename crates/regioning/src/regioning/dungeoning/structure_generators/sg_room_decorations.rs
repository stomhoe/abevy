use rand::Rng;
use rand::RngExt;

const ROOM_DECORATION_CHANCE: f32 = 0.5;

pub fn maybe_add_room_decorations(
    rng: &mut impl Rng,
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    mut clear_tile: impl FnMut(usize, usize),
) {
    if room_w < 7 || room_h < 7 {
        return;
    }
    if rng.random_range(0.0..1.0) >= ROOM_DECORATION_CHANCE {
        return;
    }

    match rng.random_range(0..3) {
        0 => add_circumference_pillars(room_x, room_y, room_w, room_h, &mut clear_tile),
        1 => add_inner_rectangle_pillars(room_x, room_y, room_w, room_h, &mut clear_tile),
        _ => add_plus_divider(room_x, room_y, room_w, room_h, rng, &mut clear_tile),
    }
}


fn add_circumference_pillars(
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    clear_tile: &mut impl FnMut(usize, usize),
) {
    let pillar_count = ((room_w.max(room_h) as usize) / 2).clamp(8, 16);
    let center_x = room_x as f32 + (room_w as f32 - 1.0) * 0.5;
    let center_y = room_y as f32 + (room_h as f32 - 1.0) * 0.5;
    let radius_x = (room_w as f32 * 0.66 * 0.5).max(2.0);
    let radius_y = (room_h as f32 * 0.66 * 0.5).max(2.0);

    for i in 0..pillar_count {
        let angle = (i as f32 / pillar_count as f32) * std::f32::consts::TAU;
        let x = (center_x + radius_x * angle.cos()).round() as i32;
        let y = (center_y + radius_y * angle.sin()).round() as i32;
        if x >= room_x && x < room_x + room_w && y >= room_y && y < room_y + room_h {
            clear_tile(x as usize, y as usize);
        }
    }
}

fn add_inner_rectangle_pillars(
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    clear_tile: &mut impl FnMut(usize, usize),
) {
    let inset_x = (room_w / 4).clamp(2, 6);
    let inset_y = (room_h / 4).clamp(2, 6);
    let start_x = room_x + inset_x;
    let end_x = room_x + room_w - inset_x - 1;
    let start_y = room_y + inset_y;
    let end_y = room_y + room_h - inset_y - 1;
    if end_x <= start_x || end_y <= start_y {
        return;
    }

    let step = if room_w.min(room_h) >= 18 { 4 } else { 3 };
    let mut y = start_y;
    while y <= end_y {
        let mut x = start_x;
        while x <= end_x {
            clear_tile(x as usize, y as usize);
            x += step;
        }
        y += step;
    }
}

fn add_plus_divider(
    room_x: i32,
    room_y: i32,
    room_w: i32,
    room_h: i32,
    rng: &mut impl Rng,
    clear_tile: &mut impl FnMut(usize, usize),
) {
    let center_x = room_x + room_w / 2;
    let center_y = room_y + room_h / 2;
    if room_w < 9 || room_h < 9 {
        add_inner_rectangle_pillars(room_x, room_y, room_w, room_h, clear_tile);
        return;
    }

    let vertical_gap_y = pick_non_center_slot(rng, room_y + 1, room_y + room_h - 1, center_y);
    let horizontal_gap_x = pick_non_center_slot(rng, room_x + 1, room_x + room_w - 1, center_x);

    for y in (room_y + 1)..(room_y + room_h - 1) {
        if y != vertical_gap_y {
            clear_tile(center_x as usize, y as usize);
        }
    }

    for x in (room_x + 1)..(room_x + room_w - 1) {
        if x != horizontal_gap_x {
            clear_tile(x as usize, center_y as usize);
        }
    }
}

fn pick_non_center_slot(rng: &mut impl Rng, min: i32, max: i32, avoid: i32) -> i32 {
    let mut candidate = rng.random_range(min..max);
    if candidate == avoid {
        candidate = if candidate + 1 < max {
            candidate + 1
        } else {
            candidate - 1
        };
    }
    candidate
}