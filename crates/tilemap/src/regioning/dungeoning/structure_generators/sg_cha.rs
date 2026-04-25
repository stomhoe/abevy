#[allow(unused_imports)] use bevy::{platform::collections::*, prelude::*};
use rand_pcg::Pcg64Mcg;

use common::common_components::HashId;
#[allow(unused_imports)] use common::log_targets::DUNGEONING_SYSTEM;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::regioning::{    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder, TerrGenDisabledGposForChunks},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::terrain::terrgen_async_resources::TerrGenBlockedGposMask;
use crate::tile::tile_resources::*;
use super::super::dungeoning_carve_helpers::{
    carve_corridor_polyline_typed as carve_corridor_path,
    carve_room_ellipse_typed as carve_room_ellipse,
    carve_room_pentacle,
    carve_room_rectangle_typed as carve_room_rectangle,
    carve_room_trapezoid_typed as carve_room_trapezoid,
    carve_room_regular_polygon_typed as carve_room_regular_polygon,
};
use super::super::dungeoning_ids::CHAMBERS_CORRIDORS;
use super::super::dungeoning_utils::{carve_external_wall_doorways, extend_occupied_gpos, queue_room_spawn_instance_message, ExternalDoorwayConfig, DeleteOtherTilesConfigMap, TerrGenDisableConfigMap};
use super::sg_cha_room_forming::{maybe_add_room_interior, pick_nearest_room_index, pick_weighted_open_parent_index, room_center, sample_corridor_child_slots, sample_room_spec_for_leaf_with_limit, sample_trapezoid_vertices_for_room, split_room_draft_horizontally, split_room_draft_vertically, RoomDraft};
use super::sg_cha_types::*;

const FLOOR_NONE: u8 = 0;
const FLOOR_MAIN: u8 = 1;
const FLOOR_B: u8 = 2;

fn seal_structure_border_band_typed(
    floor_map: &mut [u8],
    hazard_map: &mut [bool],
    map_width: usize,
    map_height: usize,
    border_band: usize,
) {
    if border_band == 0 || map_width == 0 || map_height == 0 {
        return;
    }
    if border_band * 2 >= map_width || border_band * 2 >= map_height {
        floor_map.fill(FLOOR_NONE);
        hazard_map.fill(false);
        return;
    }
    
    let end_x = map_width - border_band;
    let end_y = map_height - border_band;
    for y in 0..map_height {
        for x in 0..map_width {
            if x < border_band || x >= end_x || y < border_band || y >= end_y {
                let idx = y * map_width + x;
                floor_map[idx] = FLOOR_NONE;
                hazard_map[idx] = false;
            }
        }
    }
}

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

fn build_straight_corridor_route(
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

fn build_square_turn_corridor_route(
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

fn build_s_wiggle_corridor_route(
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
    let sample_count = (distance / 4).clamp(8, 24) as usize * turns;
    let amplitude = rng.random_range((wiggle_offset / 2).max(1)..=wiggle_offset) as f32;
    let phase = if rng.random_bool(0.5) { 0.0 } else { std::f32::consts::FRAC_PI_2 };
    let horizontal_major = dx.abs() >= dy.abs();

    let mut points = Vec::with_capacity(sample_count + 1);
    for sample_index in 0..=sample_count {
        let t = sample_index as f32 / sample_count as f32;
        let wave = (t * std::f32::consts::TAU * turns as f32 + phase).sin() * amplitude;
        let base_x = start.0 as f32 + (end.0 - start.0) as f32 * t;
        let base_y = start.1 as f32 + (end.1 - start.1) as f32 * t;
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

fn build_macro_detour_corridor_route(
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

fn pick_room_aware_corridor_route(
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

fn stamp_corridor_wall_ribs_typed(
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

fn carve_regular_polygon_wall_dividers_typed(
    floor_map_bool: &mut [bool],
    wall_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    sides: i32,
    rotation_deg: f32,
    gap_half_width: i32,
) {
    let w = w.max(0);
    let h = h.max(0);
    if w == 0 || h == 0 {
        return;
    }

    let sides = sides.max(3) as usize;
    let cx = x + w / 2;
    let cy = y + h / 2;
    let radius = (w.min(h) / 2).max(1) as f32;
    let rotation_rad = rotation_deg.to_radians();
    let step = std::f32::consts::TAU / sides as f32;

    let mut vertices: Vec<(i32, i32)> = Vec::with_capacity(sides);
    for i in 0..sides {
        let angle = rotation_rad + step * i as f32;
        let vx = cx as f32 + radius * angle.cos();
        let vy = cy as f32 + radius * angle.sin();
        vertices.push((vx.round() as i32, vy.round() as i32));
    }

    let gap_half_width = gap_half_width.max(0) as usize;
    let draw_line_with_gap = |start: (i32, i32), end: (i32, i32), floor_map_bool: &mut [bool], wall_map: &mut [bool]| {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let steps = (dx.max(dy) * 2).max(1) as usize;
        let gap_idx = steps / 2;

        for i in 0..=steps {
            if i.abs_diff(gap_idx) <= gap_half_width {
                continue;
            }

            let t = i as f32 / steps as f32;
            let x = (x0 as f32 + (x1 - x0) as f32 * t).round() as i32;
            let y = (y0 as f32 + (y1 - y0) as f32 * t).round() as i32;
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
    };

    let center = (cx, cy);
    for &vertex in &vertices {
        draw_line_with_gap(vertex, center, floor_map_bool, wall_map);
    }
}

inventory::submit! {
    crate::regioning::dungeoning::dungeoning_ids::StructureGeneratorDescriptor {
        structure_hash_id: CHAMBERS_CORRIDORS,
    }
}

#[allow(unused_parens, )]
pub fn corridor_dungeon_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    templs_map: Res<TileEntityMap>,
    mut room_pack_spawn: super::super::dungeoning_utils::DungeonRoomPackSpawnSystemParams,
    templ_size_query: Query<&SizeInTiles, (With<game_common::game_common_components::Templ>, common::AnyDisabling)>,
    settings: Query<&GlobalGenSettings>,
    mut compliances_to_emit: Local<Vec<StructureBuildCompliance>>,
    mut room_drafts: Local<Vec<RoomDraft>>,
    _tiles_buffer: Local<Vec<(GlobalTilePos, TileRef, Option<DeleteOtherTilesInSamePos>)>>,
) {
    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };
    compliances_to_emit.clear();
    room_pack_spawn.begin_pass();
    room_drafts.clear();
    
    for build_order in reader.read() {

        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != CHAMBERS_CORRIDORS {
            continue;
        }
        let room_spawn_config = super::super::dungeoning_utils::DungeonRoomPackSpawnConfig::from_typed_args(
            &structured_gen_cfg.typed_args,
            &room_pack_spawn.command_registry,
            structured_gen_cfg.structure_id().as_str(),
        );
        let mut beings_remaining = structured_gen_cfg.max_being_count.unwrap_or(u32::MAX);
        let floor_tile_id = structured_gen_cfg.args
            .get("floor_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("dunewbie"));

        let wall_tile_id = structured_gen_cfg.args
            .get("wall_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("gray"));

        let delete_other_tiles_by_tile_id = DeleteOtherTilesConfigMap::from_args(&structured_gen_cfg.args);
        let terrgen_disable_by_tile_id = TerrGenDisableConfigMap::from_args(&structured_gen_cfg.args);

        let Ok(floor_entity_ent) = templs_map.0.get_cloned(floor_tile_id) else {
            error!(target: DUNGEONING_SYSTEM, "TileTempl with id '{:?}' not found in TileEntityMap when making ChambersCorridorsDungeon, skipping structure spawn", floor_tile_id);
            continue;
        };
        let Ok(_wall_entity_ent) = templs_map.0.get_cloned(wall_tile_id) else {
            error!(target: DUNGEONING_SYSTEM, "TileTempl with id '{:?}' not found in TileEntityMap when making ChambersCorridorsDungeon, skipping structure spawn", wall_tile_id);
            continue;
        };

        let floor_entity = TileRef(floor_tile_id);
        let wall_entity = TileRef(wall_tile_id);

        let chunk_positions = &build_order.chunks_pos;
        if chunk_positions.is_empty() {
            continue;
        }

        let min_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).min().unwrap();
        let max_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).max().unwrap();
        let min_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).min().unwrap();
        let max_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).max().unwrap();

        let chunk_width = (max_chunk_x - min_chunk_x + 1) as usize;
        let chunk_height = (max_chunk_y - min_chunk_y + 1) as usize;
        let map_width = chunk_width * ChunkPos::CHUNK_SIZE.x as usize;
        let map_height = chunk_height * ChunkPos::CHUNK_SIZE.y as usize;
        if map_width == 0 || map_height == 0 {
            continue;
        }

        let origin_chunk = ChunkPos::new(min_chunk_x, min_chunk_y);
        let origin_tile = origin_chunk.to_tilepos();

        let mut max_rooms = 50;
        let mut border_band = 1;
        let mut dungeon_non_curved_corridors_chance = 0.10_f32;
        let mut corridor_zigzag_chance = 0.35_f32;
        let mut corridor_macro_detour_chance = 0.15_f32;
        let mut corridor_macro_detour_offset_min = 10_i32;
        let mut corridor_macro_detour_offset_max = 100_i32;
        let mut corridor_macro_detour_stack_min = 1_i32;
        let mut corridor_macro_detour_stack_max = 3_i32;
        let mut corridor_zigzag_curve_chance = 0.5_f32;
        let mut corridor_zigzag_strength_min = 5_i32;
        let mut corridor_zigzag_strength_max = 20_i32;
        let mut corridor_zigzag_offset = 4_i32;
        let mut corridor_zigzag_rib_spacing_min = 3_i32;
        let mut corridor_zigzag_rib_spacing_max = 6_i32;
        let mut corridor_zigzag_rib_depth_min = 4_i32;
        let mut corridor_zigzag_rib_depth_max = 8_i32;
        let mut corridor_radius: i32 = 1;
        let mut regular_polygon_divider_chance = 0.5_f32;
        let mut regular_polygon_divider_gap_half_width: i32 = 1;
        let mut regular_polygon_sides_min: i32 = 5;
        let mut regular_polygon_sides_max: i32 = 8;
        let mut room_density_min: f32 = 0.05;
        let mut room_density_max: f32 = 0.8;
        if let Some(v) = structured_gen_cfg.args.get("max_rooms") {
            if let Some(s) = v.first() {
                max_rooms = s.parse::<u32>().unwrap_or(max_rooms);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("border_band") {
            if let Some(s) = v.first() {
                border_band = s.parse::<usize>().unwrap_or(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_non_curved_dungeon_chance") {
            if let Some(s) = v.first() {
                dungeon_non_curved_corridors_chance = s.parse::<f32>().unwrap_or(dungeon_non_curved_corridors_chance).clamp(0.0, 1.0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_chance").or_else(|| structured_gen_cfg.args.get("corridor_detour_chance")) {
            if let Some(s) = v.first() {
                corridor_zigzag_chance = s.parse::<f32>().unwrap_or(corridor_zigzag_chance).clamp(0.0, 1.0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_macro_detour_chance") {
            if let Some(s) = v.first() {
                corridor_macro_detour_chance = s.parse::<f32>().unwrap_or(corridor_macro_detour_chance).clamp(0.0, 1.0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_macro_detour_offset_min").or_else(|| structured_gen_cfg.args.get("corridor_detour_offset_min")) {
            if let Some(s) = v.first() {
                corridor_macro_detour_offset_min = s.parse::<i32>().unwrap_or(corridor_macro_detour_offset_min).max(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_macro_detour_offset_max").or_else(|| structured_gen_cfg.args.get("corridor_detour_offset_max")) {
            if let Some(s) = v.first() {
                corridor_macro_detour_offset_max = s.parse::<i32>().unwrap_or(corridor_macro_detour_offset_max).max(1);
            }
        }
        if corridor_macro_detour_offset_max < corridor_macro_detour_offset_min {
            corridor_macro_detour_offset_max = corridor_macro_detour_offset_min;
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_macro_detour_stack_min").or_else(|| structured_gen_cfg.args.get("corridor_detour_stack_min")) {
            if let Some(s) = v.first() {
                corridor_macro_detour_stack_min = s.parse::<i32>().unwrap_or(corridor_macro_detour_stack_min).max(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_macro_detour_stack_max").or_else(|| structured_gen_cfg.args.get("corridor_detour_stack_max")) {
            if let Some(s) = v.first() {
                corridor_macro_detour_stack_max = s.parse::<i32>().unwrap_or(corridor_macro_detour_stack_max).max(1);
            }
        }
        if corridor_macro_detour_stack_max < corridor_macro_detour_stack_min {
            corridor_macro_detour_stack_max = corridor_macro_detour_stack_min;
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_curve_chance") {
            if let Some(s) = v.first() {
                corridor_zigzag_curve_chance = s.parse::<f32>().unwrap_or(corridor_zigzag_curve_chance).clamp(0.0, 1.0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_rib_spacing_min").or_else(|| structured_gen_cfg.args.get("corridor_zigzag_wall_rib_spacing_min")) {
            if let Some(s) = v.first() {
                corridor_zigzag_rib_spacing_min = s.parse::<i32>().unwrap_or(corridor_zigzag_rib_spacing_min).max(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_rib_spacing_max").or_else(|| structured_gen_cfg.args.get("corridor_zigzag_wall_rib_spacing_max")) {
            if let Some(s) = v.first() {
                corridor_zigzag_rib_spacing_max = s.parse::<i32>().unwrap_or(corridor_zigzag_rib_spacing_max).max(1);
            }
        }
        if corridor_zigzag_rib_spacing_max < corridor_zigzag_rib_spacing_min {
            corridor_zigzag_rib_spacing_max = corridor_zigzag_rib_spacing_min;
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_rib_depth_min").or_else(|| structured_gen_cfg.args.get("corridor_zigzag_wall_rib_depth_min")) {
            if let Some(s) = v.first() {
                corridor_zigzag_rib_depth_min = s.parse::<i32>().unwrap_or(corridor_zigzag_rib_depth_min).max(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_rib_depth_max").or_else(|| structured_gen_cfg.args.get("corridor_zigzag_wall_rib_depth_max")) {
            if let Some(s) = v.first() {
                corridor_zigzag_rib_depth_max = s.parse::<i32>().unwrap_or(corridor_zigzag_rib_depth_max).max(1);
            }
        }
        if corridor_zigzag_rib_depth_max < corridor_zigzag_rib_depth_min {
            corridor_zigzag_rib_depth_max = corridor_zigzag_rib_depth_min;
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_strength_min").or_else(|| structured_gen_cfg.args.get("corridor_zigzag_turns_min")) {
            if let Some(s) = v.first() {
                corridor_zigzag_strength_min = s.parse::<i32>().unwrap_or(corridor_zigzag_strength_min).max(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_strength_max").or_else(|| structured_gen_cfg.args.get("corridor_zigzag_turns_max")) {
            if let Some(s) = v.first() {
                corridor_zigzag_strength_max = s.parse::<i32>().unwrap_or(corridor_zigzag_strength_max).max(1);
            }
        }
        if corridor_zigzag_strength_max < corridor_zigzag_strength_min {
            corridor_zigzag_strength_max = corridor_zigzag_strength_min;
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_zigzag_offset").or_else(|| structured_gen_cfg.args.get("corridor_curve_offset")).or_else(|| structured_gen_cfg.args.get("corridor_detour_max_offset")) {
            if let Some(s) = v.first() {
                corridor_zigzag_offset = s.parse::<i32>().unwrap_or(corridor_zigzag_offset).max(0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("corridor_radius") {
            if let Some(s) = v.first() {
                corridor_radius = s.parse::<i32>().unwrap_or(1).max(1);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("regular_polygon_divider_chance") {
            if let Some(s) = v.first() {
                regular_polygon_divider_chance = s.parse::<f32>().unwrap_or(regular_polygon_divider_chance).clamp(0.0, 1.0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("regular_polygon_divider_gap_half_width") {
            if let Some(s) = v.first() {
                regular_polygon_divider_gap_half_width = s.parse::<i32>().unwrap_or(regular_polygon_divider_gap_half_width).max(0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("regular_polygon_sides") {
            if let Some(s) = v.first() {
                let sides = s.parse::<i32>().unwrap_or(regular_polygon_sides_min).max(3);
                regular_polygon_sides_min = sides;
                regular_polygon_sides_max = sides;
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("regular_polygon_sides.min").or_else(|| structured_gen_cfg.args.get("regular_polygon_sides.min_sides")) {
            if let Some(s) = v.first() {
                regular_polygon_sides_min = s.parse::<i32>().unwrap_or(regular_polygon_sides_min).max(3);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("regular_polygon_sides.max").or_else(|| structured_gen_cfg.args.get("regular_polygon_sides.max_sides")) {
            if let Some(s) = v.first() {
                regular_polygon_sides_max = s.parse::<i32>().unwrap_or(regular_polygon_sides_max).max(3);
            }
        }
        if regular_polygon_sides_max < regular_polygon_sides_min {
            regular_polygon_sides_max = regular_polygon_sides_min;
        }
        if let Some(v) = structured_gen_cfg.args.get("room_density.min") {
            if let Some(s) = v.first() {
                room_density_min = s.parse::<f32>().unwrap_or(room_density_min).clamp(0.0, 1.0);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("room_density.max") {
            if let Some(s) = v.first() {
                room_density_max = s.parse::<f32>().unwrap_or(room_density_max).clamp(0.0, 1.0);
            }
        }
        if room_density_max < room_density_min {
            room_density_max = room_density_min;
        }

        let mut floor_map = vec![FLOOR_NONE; map_width * map_height];
        let mut hazard_map = vec![false; map_width * map_height];

        let mut size_configs: HashMap<RoomShape, RoomSizeConfig> = HashMap::default();
        let global_size_config = RoomSizeConfig::default_global();
        let mut min_leaf_w = global_size_config.width.min;
        let mut min_leaf_h = global_size_config.height.min;
        if let Some(v) = structured_gen_cfg.args.get("min_leaf_w") {
            if let Some(s) = v.first() {
                min_leaf_w = s.parse::<i32>().unwrap_or(min_leaf_w).max(7);
            }
        }
        if let Some(v) = structured_gen_cfg.args.get("min_leaf_h") {
            if let Some(s) = v.first() {
                min_leaf_h = s.parse::<i32>().unwrap_or(min_leaf_h).max(7);
            }
        }

        for shape in [RoomShape::Rectangle, RoomShape::Ellipse, RoomShape::Trapezoid, RoomShape::RegularPolygon, RoomShape::Pentacle] {
            let shape_str = shape.as_str();
            let mut cfg = global_size_config;
            let width_min_key = format!("room_size.{}.width.min", shape_str);
            let width_max_key = format!("room_size.{}.width.max", shape_str);
            let width_min_alias_key = format!("room_size.{}.min_w", shape_str);
            let width_max_alias_key = format!("room_size.{}.max_w", shape_str);
            let height_min_key = format!("room_size.{}.height.min", shape_str);
            let height_max_key = format!("room_size.{}.height.max", shape_str);
            let height_min_alias_key = format!("room_size.{}.min_h", shape_str);
            let height_max_alias_key = format!("room_size.{}.max_h", shape_str);

            if let Some(v) = structured_gen_cfg.args.get(&width_min_key).or_else(|| structured_gen_cfg.args.get(&width_min_alias_key)) {
                if let Some(s) = v.first() { cfg.width.min = s.parse().unwrap_or(cfg.width.min); }
            }
            if let Some(v) = structured_gen_cfg.args.get(&width_max_key).or_else(|| structured_gen_cfg.args.get(&width_max_alias_key)) {
                if let Some(s) = v.first() { cfg.width.max = s.parse().ok(); }
            }
            if let Some(v) = structured_gen_cfg.args.get(&height_min_key).or_else(|| structured_gen_cfg.args.get(&height_min_alias_key)) {
                if let Some(s) = v.first() { cfg.height.min = s.parse().unwrap_or(cfg.height.min); }
            }
            if let Some(v) = structured_gen_cfg.args.get(&height_max_key).or_else(|| structured_gen_cfg.args.get(&height_max_alias_key)) {
                if let Some(s) = v.first() { cfg.height.max = s.parse().ok(); }
            }
            if let RoomShape::Pentacle = shape {
                cfg.width.min = cfg.width.min.max(15);
                cfg.height.min = cfg.height.min.max(15);
            }
            size_configs.insert(shape, cfg);
        }

        let seed = origin_chunk.hash_value(&settings, build_order.dimension_ref.0, build_order.i);
        let mut rng = Pcg64Mcg::seed_from_u64(seed);
        let corridors_are_non_curved = rng.random_range(0.0..1.0) < dungeon_non_curved_corridors_chance;
        let claimed_area = map_width.saturating_mul(map_height);
        let room_density = rng.random_range(room_density_min..=room_density_max);
        let room_area_budget = ((claimed_area as f32) * room_density).round().max(1.0) as i32;
        debug!(target: DUNGEONING_SYSTEM, "structure={} room_density={:.3} room_area_budget={} claimed_area={} density_range=[{:.3}, {:.3}]", structured_gen_cfg.structure_id(), room_density, room_area_budget, claimed_area, room_density_min, room_density_max);
        room_drafts.clear();
        room_drafts.push(RoomDraft { x: 0, y: 0, w: map_width as i32, h: map_height as i32 });

        let mut actual_rooms = Vec::new();
        let mut used_room_area = 0_i32;
        let mut curved_zigzag_routes: Vec<Vec<(i32, i32)>> = Vec::with_capacity(max_rooms as usize);
        while actual_rooms.len() < max_rooms as usize && !room_drafts.is_empty() && used_room_area < room_area_budget {
            let remaining_area_budget = room_area_budget.saturating_sub(used_room_area);
            let total_area_weight: usize = room_drafts.iter().map(|room| room.area().max(1) as usize).sum();
            let mut room_pick = rng.random_range(0..total_area_weight);
            let mut room_idx = 0;
            for (idx, room) in room_drafts.iter().enumerate() {
                let room_weight = room.area().max(1) as usize;
                if room_pick < room_weight {
                    room_idx = idx;
                    break;
                }
                room_pick -= room_weight;
            }
            let mut room = room_drafts.swap_remove(room_idx);
            let Some(plan) = sample_room_spec_for_leaf_with_limit(&room, &structured_gen_cfg.args, &size_configs, &global_size_config, Some(remaining_area_budget), actual_rooms.is_empty(), &mut rng) else { break; };
            let plan_area = plan.area().max(1);

            loop {
                if room.w == plan.w && room.h == plan.h {
                    actual_rooms.push(plan.into_room(room.x, room.y));
                    used_room_area = used_room_area.saturating_add(plan_area);
                    break;
                }

                if room.w < plan.w || room.h < plan.h {
                    break;
                }

                let surplus_w = room.w - plan.w;
                let surplus_h = room.h - plan.h;
                let split_horizontally = if surplus_h == 0 {
                    true
                } else if surplus_w == 0 {
                    false
                } else {
                    surplus_w >= surplus_h
                };

                if split_horizontally {
                    let keep_left = rng.random_bool(0.5);
                    let (next_room, remainder) = split_room_draft_horizontally(room, plan.w, keep_left);
                    room = next_room;
                    if let Some(remainder) = remainder {
                        if remainder.w >= min_leaf_w && remainder.h >= min_leaf_h {
                            room_drafts.push(remainder);
                        }
                    }
                } else {
                    let keep_top = rng.random_bool(0.5);
                    let (next_room, remainder) = split_room_draft_vertically(room, plan.h, keep_top);
                    room = next_room;
                    if let Some(remainder) = remainder {
                        if remainder.w >= min_leaf_w && remainder.h >= min_leaf_h {
                            room_drafts.push(remainder);
                        }
                    }
                }
            }
        }

        let mut regular_polygon_sides_by_room: Vec<i32> = Vec::with_capacity(actual_rooms.len());
        for room in &actual_rooms {
            let rx = room.x;
            let ry = room.y;
            match room.shape {
                RoomShape::Rectangle => {
                    regular_polygon_sides_by_room.push(0);
                    carve_room_rectangle(&mut floor_map, map_width, map_height, rx, ry, room.w, room.h, FLOOR_MAIN);
                }
                RoomShape::Ellipse => {
                    regular_polygon_sides_by_room.push(0);
                    carve_room_ellipse(&mut floor_map, map_width, map_height, rx, ry, room.w, room.h, FLOOR_MAIN);
                }
                RoomShape::Trapezoid => {
                    regular_polygon_sides_by_room.push(0);
                    let (v0, v1, v2, v3) = sample_trapezoid_vertices_for_room(room, &mut rng);
                    carve_room_trapezoid(&mut floor_map, map_width, map_height, v0, v1, v2, v3, FLOOR_MAIN);
                }
                RoomShape::RegularPolygon => {
                    let sides = rng.random_range(regular_polygon_sides_min..=regular_polygon_sides_max).max(3);
                    regular_polygon_sides_by_room.push(sides);
                    carve_room_regular_polygon(&mut floor_map, map_width, map_height, rx, ry, room.w, room.h, sides, 0.0, FLOOR_MAIN);
                }
                RoomShape::Pentacle => {
                    regular_polygon_sides_by_room.push(0);
                    carve_room_pentacle(&mut floor_map, map_width, map_height, rx, ry, room.w, room.h, FLOOR_MAIN, FLOOR_B);
                }
            }

            maybe_add_room_interior(&mut rng, room.shape, rx, ry, room.w, room.h, |x, y| {
                let idx = y * map_width + x;
                floor_map[idx] = FLOOR_NONE;
            });
        }

        let mut corridor_map = vec![false; map_width * map_height];
        if actual_rooms.len() > 1 {
            let mut remaining_rooms: Vec<usize> = (0..actual_rooms.len()).collect();
            let root_idx = remaining_rooms.swap_remove(rng.random_range(0..remaining_rooms.len()));
            let mut open_parents: Vec<(usize, usize)> = Vec::with_capacity(actual_rooms.len());
            let mut connected_rooms: Vec<usize> = Vec::with_capacity(actual_rooms.len());
            connected_rooms.push(root_idx);

            let root_slots = sample_corridor_child_slots(&mut rng).max(1);
            open_parents.push((root_idx, root_slots));

            while !remaining_rooms.is_empty() {
                if open_parents.is_empty() {
                    let fallback_idx = connected_rooms[rng.random_range(0..connected_rooms.len())];
                    open_parents.push((fallback_idx, 1));
                }

                let Some(parent_slot_idx) = pick_weighted_open_parent_index(&open_parents, &mut rng) else { continue; };
                let parent_idx = open_parents[parent_slot_idx].0;
                let Some(child_idx) = pick_nearest_room_index(parent_idx, &remaining_rooms, &actual_rooms, &mut rng) else { continue; };
                let child_pos = remaining_rooms.iter().position(|candidate_idx| *candidate_idx == child_idx).unwrap_or(0);
                let child_idx = remaining_rooms.swap_remove(child_pos);

                let c1 = room_center(&actual_rooms[parent_idx]);
                let c2 = room_center(&actual_rooms[child_idx]);
                let rad = Some(corridor_radius);
                if corridors_are_non_curved {
                    let route = build_straight_corridor_route(c1, c2, map_width, map_height, &mut rng);
                    carve_corridor_path(&mut floor_map, map_width, map_height, &mut corridor_map, rad, &route, FLOOR_MAIN);

                    let parent_slots = &mut open_parents[parent_slot_idx].1;
                    *parent_slots = parent_slots.saturating_sub(1);
                    if *parent_slots == 0 {
                        open_parents.swap_remove(parent_slot_idx);
                    }

                    connected_rooms.push(child_idx);
                    let child_slots = if remaining_rooms.is_empty() { 0 } else { sample_corridor_child_slots(&mut rng) };
                    if child_slots > 0 {
                        open_parents.push((child_idx, child_slots));
                    }
                    continue;
                }

                let (route, _route_is_curved_zigzag) = pick_room_aware_corridor_route(
                    c1,
                    c2,
                    &actual_rooms,
                    parent_idx,
                    child_idx,
                    map_width,
                    map_height,
                    corridor_zigzag_chance,
                    corridor_zigzag_curve_chance,
                    corridor_zigzag_strength_min,
                    corridor_zigzag_strength_max,
                    corridor_zigzag_offset,
                    corridor_macro_detour_chance,
                    corridor_macro_detour_offset_min,
                    corridor_macro_detour_offset_max,
                    corridor_macro_detour_stack_min,
                    corridor_macro_detour_stack_max,
                    &mut rng,
                );

                if _route_is_curved_zigzag {
                    curved_zigzag_routes.push(route.clone());
                }

                carve_corridor_path(&mut floor_map, map_width, map_height, &mut corridor_map, rad, &route, FLOOR_MAIN);

                let parent_slots = &mut open_parents[parent_slot_idx].1;
                *parent_slots = parent_slots.saturating_sub(1);
                if *parent_slots == 0 {
                    open_parents.swap_remove(parent_slot_idx);
                }

                connected_rooms.push(child_idx);
                let child_slots = if remaining_rooms.is_empty() { 0 } else { sample_corridor_child_slots(&mut rng) };
                if child_slots > 0 {
                    open_parents.push((child_idx, child_slots));
                }
            }
        }

        seal_structure_border_band_typed(&mut floor_map, &mut hazard_map, map_width, map_height, border_band);

        let mut floor_map_bool = floor_map.iter().map(|&floor| floor != FLOOR_NONE).collect::<Vec<_>>();
        let mut wall_map = vec![false; map_width * map_height];
        for y in 0..map_height {
            for x in 0..map_width {
                let idx = y * map_width + x;
                if !floor_map_bool[idx] && !hazard_map[idx] { continue; }
                let neighbors = [
                    (y.saturating_sub(1), x),
                    (y + 1, x),
                    (y, x.saturating_sub(1)),
                    (y, x + 1),
                    (y.saturating_sub(1), x.saturating_sub(1)),
                    (y.saturating_sub(1), x + 1),
                    (y + 1, x.saturating_sub(1)),
                    (y + 1, x + 1),
                ];
                for (ny, nx) in neighbors {
                    if ny < map_height && nx < map_width {
                        let nidx = ny * map_width + nx;
                        if !floor_map_bool[nidx] && !hazard_map[nidx] {
                            wall_map[nidx] = true;
                        }
                    }
                }
            }
        }

        let rib_spacing_min = corridor_zigzag_rib_spacing_min.max(1) as usize;
        let rib_spacing_max = corridor_zigzag_rib_spacing_max.max(corridor_zigzag_rib_spacing_min).max(1) as usize;
        let rib_depth_min = corridor_zigzag_rib_depth_min.max(1) as usize;
        let rib_depth_max = corridor_zigzag_rib_depth_max.max(corridor_zigzag_rib_depth_min).max(1) as usize;
        for path in curved_zigzag_routes.iter() {
            stamp_corridor_wall_ribs_typed(
                &mut floor_map_bool,
                &mut wall_map,
                map_width,
                map_height,
                corridor_radius,
                path,
                rib_spacing_min,
                rib_spacing_max,
                rib_depth_min,
                rib_depth_max,
                &mut rng,
            );
        }

        for (room, &sides) in actual_rooms.iter().zip(regular_polygon_sides_by_room.iter()) {
            if let RoomShape::RegularPolygon = room.shape {
                if sides == 0 || rng.random_range(0.0..1.0) >= regular_polygon_divider_chance {
                    continue;
                }
                carve_regular_polygon_wall_dividers_typed(
                    &mut floor_map_bool,
                    &mut wall_map,
                    map_width,
                    map_height,
                    room.x,
                    room.y,
                    room.w,
                    room.h,
                    sides,
                    0.0,
                    regular_polygon_divider_gap_half_width,
                );
            }
        }

        let opened_doorways = carve_external_wall_doorways(
            &mut floor_map_bool,
            Some(&mut hazard_map),
            &mut wall_map,
            map_width,
            map_height,
            ExternalDoorwayConfig::from_args(&structured_gen_cfg.args),
            &mut rng,
        );
        trace!(target: DUNGEONING_SYSTEM, "structure={} opened_external_doorways={}", structured_gen_cfg.structure_id(), opened_doorways);

        let floor_delete_other_tiles = delete_other_tiles_by_tile_id.get("floor_tile_id");
        let wall_delete_other_tiles = delete_other_tiles_by_tile_id.get("wall_tile_id");
        let disable_floor_terrgen = terrgen_disable_by_tile_id.should_disable_for("floor_tile_id");
        let floor_template_size = templ_size_query.get(floor_entity_ent).copied().unwrap_or_default().inner();

        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(build_order.chunks_pos.len());
        let mut terrgen_disabled_gpos_for_chunks = TerrGenDisabledGposForChunks::default();
        let mut tiles4chunk: TilesFromBuilder = Vec::new();
        for &chunk_pos in &build_order.chunks_pos {
            tiles4chunk.clear();
            let mut blocked_gpos = TerrGenBlockedGposMask::default();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk() {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= map_width || idx_y >= map_height { continue; }
                let map_idx = idx_y * map_width + idx_x;
                if floor_map_bool[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                    if disable_floor_terrgen {
                        extend_occupied_gpos(&mut blocked_gpos, chunk_pos, tile_pos, floor_template_size);
                    }
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, wall_delete_other_tiles.clone()));
                }
            }
            chunk_tiles.push((chunk_pos, std::mem::take(&mut tiles4chunk)));
            terrgen_disabled_gpos_for_chunks.insert_for_chunk(chunk_pos, blocked_gpos);
        }

        for room in &actual_rooms {
            let Some(anchor_gpos) = room.sample_spawn_anchor(&floor_map, &hazard_map, map_width, map_height, origin_tile, &mut rng, FLOOR_NONE) else { continue; };
            let queued = queue_room_spawn_instance_message(
                room.shape.as_str(),
                anchor_gpos,
                build_order.dimension_ref,
                None,
                &mut beings_remaining,
                &room_spawn_config,
                &room_pack_spawn.source_lookup,
                &mut room_pack_spawn.pending_messages,
                &mut rng,
            );
            if !queued {
                continue;
            }
            trace!(target: DUNGEONING_SYSTEM, "Queued room_spawn InstancePack for structure={} shape={} at {}", structured_gen_cfg.structure_id(), room.shape.as_str(), anchor_gpos);
        }

        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunks: chunk_tiles,
            terrgen_disabled_gpos_for_chunks,
            terrgen_disabled_for_chunks: Vec::new(),
            forced_chunk_biomes: Vec::new(),
        });
    }

    writer.write_batch(compliances_to_emit.drain(..));
    room_pack_spawn.finish_pass();
}
