use rand::Rng;
use rand::RngExt;

use super::sg_cha_types::*;

fn clamp_to_map(point: (i32, i32), map_width: usize, map_height: usize) -> (i32, i32) {
    (
        point.0.clamp(0, map_width.saturating_sub(1) as i32),
        point.1.clamp(0, map_height.saturating_sub(1) as i32),
    )
}

fn push_unique_point(points: &mut Vec<(i32, i32)>, point: (i32, i32)) {
    if points.last().copied() != Some(point) {
        points.push(point);
    }
}

pub fn build_orthogonal_corridor_route(
    start: (i32, i32),
    end: (i32, i32),
    map_width: usize,
    map_height: usize,
    horizontal_first: bool,
    detour: Option<(i32, i32)>,
) -> Vec<(i32, i32)> {
    let mut points = Vec::with_capacity(if detour.is_some() { 5 } else { 3 });
    let start = clamp_to_map(start, map_width, map_height);
    let end = clamp_to_map(end, map_width, map_height);
    push_unique_point(&mut points, start);

    match detour {
        Some((detour_x, detour_y)) if horizontal_first => {
            push_unique_point(&mut points, clamp_to_map((detour_x, start.1), map_width, map_height));
            push_unique_point(&mut points, clamp_to_map((detour_x, detour_y), map_width, map_height));
            push_unique_point(&mut points, clamp_to_map((end.0, detour_y), map_width, map_height));
        }
        Some((detour_x, detour_y)) => {
            push_unique_point(&mut points, clamp_to_map((start.0, detour_y), map_width, map_height));
            push_unique_point(&mut points, clamp_to_map((detour_x, detour_y), map_width, map_height));
            push_unique_point(&mut points, clamp_to_map((detour_x, end.1), map_width, map_height));
        }
        None if horizontal_first => {
            push_unique_point(&mut points, clamp_to_map((end.0, start.1), map_width, map_height));
        }
        None => {
            push_unique_point(&mut points, clamp_to_map((start.0, end.1), map_width, map_height));
        }
    }

    push_unique_point(&mut points, end);
    points
}

pub fn build_straight_corridor_route(
    start: (i32, i32),
    end: (i32, i32),
    map_width: usize,
    map_height: usize,
    rng: &mut impl Rng,
) -> Vec<(i32, i32)> {
    let mut points = Vec::with_capacity(3);
    push_unique_point(&mut points, clamp_to_map(start, map_width, map_height));

    if start.0 == end.0 || start.1 == end.1 {
        push_unique_point(&mut points, clamp_to_map(end, map_width, map_height));
        return points;
    }

    if rng.random_bool(0.5) {
        push_unique_point(&mut points, clamp_to_map((end.0, start.1), map_width, map_height));
    } else {
        push_unique_point(&mut points, clamp_to_map((start.0, end.1), map_width, map_height));
    }
    push_unique_point(&mut points, clamp_to_map(end, map_width, map_height));
    points
}

pub fn build_square_turn_corridor_route(
    start: (i32, i32),
    end: (i32, i32),
    map_width: usize,
    map_height: usize,
    turns: usize,
    turn_offset: i32,
    rng: &mut impl Rng,
) -> Vec<(i32, i32)> {
    let turns = turns.max(1);
    let turn_offset = turn_offset.max(1);
    let offset = rng.random_range((turn_offset / 2).max(1)..=turn_offset);
    let horizontal_major = (end.0 - start.0).abs() >= (end.1 - start.1).abs();
    let mut points = Vec::with_capacity(turns * 2 + 3);
    push_unique_point(&mut points, clamp_to_map(start, map_width, map_height));

    let mut current = start;
    let mut sign = if rng.random_bool(0.5) { 1 } else { -1 };
    for step_index in 0..turns {
        let fraction = (step_index + 1) as f32 / (turns + 1) as f32;
        if horizontal_major {
            let major_x = (start.0 as f32 + (end.0 - start.0) as f32 * fraction).round() as i32;
            let major_x = major_x.clamp(0, map_width.saturating_sub(1) as i32);
            push_unique_point(&mut points, clamp_to_map((major_x, current.1), map_width, map_height));
            current.0 = major_x;
            current.1 = (current.1 + sign * offset).clamp(0, map_height.saturating_sub(1) as i32);
            push_unique_point(&mut points, clamp_to_map((current.0, current.1), map_width, map_height));
        } else {
            let major_y = (start.1 as f32 + (end.1 - start.1) as f32 * fraction).round() as i32;
            let major_y = major_y.clamp(0, map_height.saturating_sub(1) as i32);
            push_unique_point(&mut points, clamp_to_map((current.0, major_y), map_width, map_height));
            current.1 = major_y;
            current.0 = (current.0 + sign * offset).clamp(0, map_width.saturating_sub(1) as i32);
            push_unique_point(&mut points, clamp_to_map((current.0, current.1), map_width, map_height));
        }
        sign *= -1;
    }

    if horizontal_major {
        push_unique_point(&mut points, clamp_to_map((end.0, current.1), map_width, map_height));
    } else {
        push_unique_point(&mut points, clamp_to_map((current.0, end.1), map_width, map_height));
    }
    push_unique_point(&mut points, clamp_to_map(end, map_width, map_height));
    points
}

pub fn build_s_wiggle_corridor_route(
    start: (i32, i32),
    end: (i32, i32),
    map_width: usize,
    map_height: usize,
    turns: usize,
    wiggle_offset: i32,
    rng: &mut impl Rng,
) -> Vec<(i32, i32)> {
    let turns = turns.max(1);
    let wiggle_offset = wiggle_offset.max(1);
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let distance = dx.abs().max(dy.abs()).max(1);
    let sample_count = (distance / 3).clamp(12, 32) as usize * turns;
    let amplitude = rng.random_range(wiggle_offset..=(wiggle_offset * 2).max(wiggle_offset + 1)) as f32;
    let phase = if rng.random_bool(0.5) { 0.0 } else { std::f32::consts::PI };
    let horizontal_major = dx.abs() >= dy.abs();

    let mut points = Vec::with_capacity(sample_count + 1);
    for sample_index in 0..=sample_count {
        let t = sample_index as f32 / sample_count as f32;
        let base_x = start.0 as f32 + dx as f32 * t;
        let base_y = start.1 as f32 + dy as f32 * t;
        let primary = (t * std::f32::consts::PI * turns as f32 + phase).sin();
        let envelope = 1.0 - (2.0 * t - 1.0).abs().powf(2.2);
        let secondary = (t * std::f32::consts::TAU * (turns as f32 * 0.35 + 0.75) + phase).sin() * 0.35;
        let bias = rng.random_range(-0.18_f32..0.18_f32) * (1.0 - (2.0 * t - 1.0).abs());
        let wave = (primary * envelope + secondary + bias) * amplitude;
        let point = if horizontal_major {
            clamp_to_map((base_x.round() as i32, (base_y + wave).round() as i32), map_width, map_height)
        } else {
            clamp_to_map(((base_x + wave).round() as i32, base_y.round() as i32), map_width, map_height)
        };
        push_unique_point(&mut points, point);
    }

    if points.len() < 2 {
        points.push(clamp_to_map(end, map_width, map_height));
    }

    points
}

pub fn build_macro_detour_corridor_route(
    start: (i32, i32),
    end: (i32, i32),
    map_width: usize,
    map_height: usize,
    detour_offset_min: i32,
    detour_offset_max: i32,
    detour_stack_min: i32,
    detour_stack_max: i32,
    rng: &mut impl Rng,
) -> Vec<(i32, i32)> {
    let detour_offset_min = detour_offset_min.max(1);
    let detour_offset_max = detour_offset_max.max(detour_offset_min);
    let detour_stack_min = detour_stack_min.max(1);
    let detour_stack_max = detour_stack_max.max(detour_stack_min);
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let horizontal_major = dx.abs() >= dy.abs();
    let detour_sign = if rng.random_bool(0.5) { 1 } else { -1 };
    let detour_layers = rng.random_range(detour_stack_min..=detour_stack_max).max(1) as usize;
    let mut points = Vec::with_capacity(detour_layers.saturating_mul(4).saturating_add(2));
    push_unique_point(&mut points, clamp_to_map(start, map_width, map_height));

    let route_start_t = 0.18_f32;
    let route_end_t = 0.82_f32;
    let layer_span = (route_end_t - route_start_t) / detour_layers as f32;
    let detour_pattern = rng.random_range(0..4);

    for layer_index in 0..detour_layers {
        let detour_height = rng.random_range(detour_offset_min..=detour_offset_max);
        let detour_apex = detour_height.saturating_add(rng.random_range((detour_height / 2).max(1)..=detour_height));
        let layer_start_t = route_start_t + layer_span * layer_index as f32;
        let layer_sign = match detour_pattern {
            0 => detour_sign,
            1 => {
                if layer_index % 2 == 0 { detour_sign } else { -detour_sign }
            }
            2 => {
                if layer_index + 1 == detour_layers { -detour_sign } else { detour_sign }
            }
            _ => {
                if rng.random_bool(0.5) { detour_sign } else { -detour_sign }
            }
        };
        let layer_t0 = match detour_pattern {
            0 => layer_start_t,
            1 => (layer_start_t + (layer_index as f32 * 0.03)).min(route_end_t - 0.08),
            2 => (layer_start_t + if layer_index + 1 == detour_layers { 0.12 } else { -0.02 }).clamp(route_start_t, route_end_t - 0.10),
            _ => (layer_start_t + rng.random_range(-0.04_f32..0.04_f32)).clamp(route_start_t, route_end_t - 0.12),
        };
        let layer_t1 = match detour_pattern {
            0 => (layer_t0 + layer_span * 0.42).min(route_end_t - 0.04),
            1 => (layer_t0 + layer_span * rng.random_range(0.30_f32..0.52_f32)).min(route_end_t - 0.03),
            2 => (layer_t0 + layer_span * 0.58).min(route_end_t - 0.03),
            _ => (layer_t0 + layer_span * rng.random_range(0.22_f32..0.60_f32)).min(route_end_t - 0.03),
        };
        let layer_t2 = match detour_pattern {
            0 => (layer_t0 + layer_span * 0.82).min(route_end_t),
            1 => (layer_t1 + layer_span * 0.30).min(route_end_t),
            2 => (layer_t1 + layer_span * 0.18).min(route_end_t),
            _ => (layer_t1 + layer_span * rng.random_range(0.24_f32..0.58_f32)).min(route_end_t),
        };

        if horizontal_major {
            let entry_x = (start.0 as f32 + dx as f32 * layer_t0).round() as i32;
            let apex_x = (start.0 as f32 + dx as f32 * layer_t1).round() as i32;
            let exit_x = (start.0 as f32 + dx as f32 * layer_t2).round() as i32;
            let base_entry_y = (start.1 as f32 + dy as f32 * layer_t0).round() as i32;
            let base_apex_y = (start.1 as f32 + dy as f32 * layer_t1).round() as i32;
            let base_exit_y = (start.1 as f32 + dy as f32 * layer_t2).round() as i32;
            push_unique_point(&mut points, clamp_to_map((entry_x, base_entry_y + detour_sign * detour_height), map_width, map_height));
            push_unique_point(&mut points, clamp_to_map((apex_x, base_apex_y + detour_sign * detour_apex), map_width, map_height));
            push_unique_point(&mut points, clamp_to_map((exit_x, base_exit_y + detour_sign * detour_height), map_width, map_height));
        } else {
            let entry_y = (start.1 as f32 + dy as f32 * layer_t0).round() as i32;
            let apex_y = (start.1 as f32 + dy as f32 * layer_t1).round() as i32;
            let exit_y = (start.1 as f32 + dy as f32 * layer_t2).round() as i32;
            let base_entry_x = (start.0 as f32 + dx as f32 * layer_t0).round() as i32;
            let base_apex_x = (start.0 as f32 + dx as f32 * layer_t1).round() as i32;
            let base_exit_x = (start.0 as f32 + dx as f32 * layer_t2).round() as i32;
            push_unique_point(&mut points, clamp_to_map((base_entry_x + layer_sign * detour_height, entry_y), map_width, map_height));
            push_unique_point(&mut points, clamp_to_map((base_apex_x + layer_sign * detour_apex, apex_y), map_width, map_height));
            push_unique_point(&mut points, clamp_to_map((base_exit_x + layer_sign * detour_height, exit_y), map_width, map_height));
        }
    }

    push_unique_point(&mut points, clamp_to_map(end, map_width, map_height));
    points
}

fn point_inside_room(point: (i32, i32), room: &Room) -> bool {
    point.0 >= room.x && point.0 < room.x + room.w && point.1 >= room.y && point.1 < room.y + room.h
}

fn route_crosses_other_rooms(route: &[(i32, i32)], actual_rooms: &[Room], start_room_idx: usize, end_room_idx: usize) -> bool {
    if route.len() < 3 {
        return false;
    }

    for (point_index, &point) in route.iter().enumerate() {
        if point_index == 0 || point_index + 1 == route.len() {
            continue;
        }

        for (room_idx, room) in actual_rooms.iter().enumerate() {
            if room_idx == start_room_idx || room_idx == end_room_idx {
                continue;
            }

            if point_inside_room(point, room) {
                return true;
            }
        }
    }

    false
}

pub fn pick_room_aware_corridor_route(
    c1: (i32, i32),
    c2: (i32, i32),
    actual_rooms: &[Room],
    start_room_idx: usize,
    end_room_idx: usize,
    map_width: usize,
    map_height: usize,
    corridor_zigzag_chance: f32,
    corridor_zigzag_curve_chance: f32,
    corridor_zigzag_strength_min: i32,
    corridor_zigzag_strength_max: i32,
    corridor_zigzag_offset: i32,
    corridor_macro_detour_chance: f32,
    corridor_macro_detour_offset_min: i32,
    corridor_macro_detour_offset_max: i32,
    corridor_macro_detour_stack_min: i32,
    corridor_macro_detour_stack_max: i32,
    rng: &mut impl Rng,
) -> (Vec<(i32, i32)>, bool) {
    let corridor_dx = (c2.0 - c1.0).abs() as usize;
    let corridor_dy = (c2.1 - c1.1).abs() as usize;
    let corridor_span = corridor_dx.max(corridor_dy).max(1);
    let long_corridor_bonus = (((corridor_span.saturating_sub(12)) as f32) / 36.0).clamp(0.0, 0.45);
    let zigzag_chance = (corridor_zigzag_chance + long_corridor_bonus).clamp(0.0, 1.0);
    let zigzag_strength_min = corridor_zigzag_strength_min.max(1) as usize;
    let zigzag_strength_max = corridor_zigzag_strength_max.max(corridor_zigzag_strength_min).max(1) as usize;
    let fallback_turns = (corridor_span / 14).clamp(2, 5);

    for _ in 0..6 {
        let route = if rng.random_range(0.0..1.0) < zigzag_chance {
            let sampled_zigzag_strength = rng.random_range(zigzag_strength_min..=zigzag_strength_max);
            let span_biased_zigzag_strength = (corridor_span / 6).clamp(zigzag_strength_min, zigzag_strength_max);
            let zigzag_strength = sampled_zigzag_strength.max(span_biased_zigzag_strength);
            let zigzag_turns = zigzag_strength.max(2);
            let zigzag_offset = corridor_zigzag_offset.saturating_add((zigzag_strength / 2) as i32);
            if rng.random_range(0.0..1.0) < corridor_zigzag_curve_chance {
                build_s_wiggle_corridor_route(c1, c2, map_width, map_height, zigzag_turns, zigzag_offset, rng)
            } else {
                build_square_turn_corridor_route(c1, c2, map_width, map_height, zigzag_turns, zigzag_offset, rng)
            }
        } else if c1.0 == c2.0 || c1.1 == c2.1 {
            build_straight_corridor_route(c1, c2, map_width, map_height, rng)
        } else if rng.random_range(0.0..1.0) < corridor_macro_detour_chance {
            build_macro_detour_corridor_route(
                c1,
                c2,
                map_width,
                map_height,
                corridor_macro_detour_offset_min,
                corridor_macro_detour_offset_max,
                corridor_macro_detour_stack_min,
                corridor_macro_detour_stack_max,
                rng,
            )
        } else {
            let fallback_roll = rng.random_range(0.0..1.0);
            if fallback_roll < 0.15 {
                build_straight_corridor_route(c1, c2, map_width, map_height, rng)
            } else if fallback_roll < 0.78 {
                build_square_turn_corridor_route(c1, c2, map_width, map_height, fallback_turns, corridor_zigzag_offset, rng)
            } else {
                build_s_wiggle_corridor_route(c1, c2, map_width, map_height, fallback_turns, corridor_zigzag_offset, rng)
            }
        };

        if !route_crosses_other_rooms(&route, actual_rooms, start_room_idx, end_room_idx) {
            return (route, false);
        }
    }

    let fallback_route = build_straight_corridor_route(c1, c2, map_width, map_height, rng);
    (fallback_route, false)
}

pub fn stamp_corridor_wall_ribs_typed(
    floor_map_bool: &mut [bool],
    wall_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    corridor_radius: i32,
    path: &[(i32, i32)],
    rib_spacing_min: usize,
    rib_spacing_max: usize,
    rib_depth_min: usize,
    rib_depth_max: usize,
    rng: &mut impl Rng,
) {
    if path.len() < 3 {
        return;
    }

    let corridor_radius = corridor_radius.max(1);
    let rib_spacing_min = rib_spacing_min.max(1);
    let rib_spacing_max = rib_spacing_max.max(rib_spacing_min);
    let rib_depth_min = rib_depth_min.max(1);
    let rib_depth_max = rib_depth_max.max(rib_depth_min);

    let mut next_index = rng.random_range(rib_spacing_min..=rib_spacing_max);
    let mut side_bias = if rng.random_bool(0.5) { 1 } else { -1 };

    while next_index + 1 < path.len() {
        let current_index = next_index;
        let prev = path[current_index - 1];
        let current = path[current_index];
        let next = path[current_index + 1];

        let dir_x = (next.0 - prev.0).signum();
        let dir_y = (next.1 - prev.1).signum();
        if dir_x == 0 && dir_y == 0 {
            side_bias *= -1;
            next_index += rng.random_range(rib_spacing_min..=rib_spacing_max);
            continue;
        }

        let rib_depth = rng.random_range(rib_depth_min..=rib_depth_max).max(corridor_radius as usize + 2);
        let rib_width = (rib_depth / 2).clamp(2, 5);
        let side_depths = [rib_depth, rib_depth];
        let side_widths = [rib_width, rib_width];

        for (side_idx, side_sign) in [side_bias, -side_bias].into_iter().enumerate() {
            let normal_x = -dir_y * side_sign;
            let normal_y = dir_x * side_sign;
            let base_x = current.0 + normal_x * corridor_radius;
            let base_y = current.1 + normal_y * corridor_radius;
            if base_x < 0 || base_y < 0 {
                continue;
            }

            let base_ux = base_x as usize;
            let base_uy = base_y as usize;
            if base_ux >= tile_width || base_uy >= tile_height {
                continue;
            }

            let rib_depth = side_depths[side_idx];
            let rib_width = side_widths[side_idx];
            for depth_step in 0..=rib_depth {
                let inward_step = depth_step as i32;
                let step_back_x = base_x - normal_x * inward_step;
                let step_back_y = base_y - normal_y * inward_step;
                let taper = (rib_width.saturating_sub(depth_step / 3)).max(2) as i32;

                for lateral in -taper..=taper {
                    let x = step_back_x + dir_x * lateral;
                    let y = step_back_y + dir_y * lateral;
                    if x < 0 || y < 0 {
                        continue;
                    }

                    let ux = x as usize;
                    let uy = y as usize;
                    if ux >= tile_width || uy >= tile_height {
                        continue;
                    }

                    let idx = uy * tile_width + ux;
                    floor_map_bool[idx] = false;
                    wall_map[idx] = true;
                }
            }
        }

        side_bias *= -1;
        next_index += rng.random_range(rib_spacing_min..=rib_spacing_max);
    }
}
