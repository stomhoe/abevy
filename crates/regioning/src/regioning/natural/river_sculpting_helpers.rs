use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use common::common_components::HashId;
use tilemap_shared::{ChunkGposMask, ChunkPos, GlobalGenSettings, GlobalTilePos, RegionPos};
use tilemap_shared::HashablePosVec;

use super::river_components::RiverRegionPlan;
use super::river_formation::is_near_region_border;

pub(super) fn river_noise_signed(
    source: GlobalTilePos,
    curr: GlobalTilePos,
    next: GlobalTilePos,
    step: usize,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
) -> f32 {
    let a = source.hash_value(settings, dimension_hash, 191 + step as u64);
    let b = curr.hash_value(settings, dimension_hash, 277 + (step as u64 * 3));
    let c = next.hash_value(settings, dimension_hash, 383 + (step as u64 * 5));
    let mixed = a ^ b.rotate_left(19) ^ c.rotate_left(37);
    ((mixed & 0xFFFF) as f32 / 65535.0) * 2.0 - 1.0
}

pub(super) fn smooth_river_path(
    path: &[GlobalTilePos],
    curve_iterations: usize,
    curve_jitter_tiles: f32,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    source: GlobalTilePos,
) -> Vec<GlobalTilePos> {
    if path.len() < 2 {
        return path.to_vec();
    }
    let mut points = path.iter().map(|point| point.0.as_vec2()).collect::<Vec<_>>();
    for iter in 0..curve_iterations {
        let mut next_points = Vec::with_capacity(points.len() * 2);
        next_points.push(points[0]);
        for window in points.windows(2) {
            let p0 = window[0];
            let p1 = window[1];
            next_points.push(p0 * 0.75 + p1 * 0.25);
            next_points.push(p0 * 0.25 + p1 * 0.75);
        }
        next_points.push(*points.last().unwrap_or(&points[0]));
        points = next_points;
        if curve_jitter_tiles > 0.0 && points.len() > 2 {
            for i in 1..points.len() - 1 {
                let tangent = (points[i + 1] - points[i - 1]).normalize_or_zero();
                if tangent.length_squared() <= f32::EPSILON {
                    continue;
                }
                let normal = Vec2::new(-tangent.y, tangent.x);
                let jitter = river_noise_signed(
                    source,
                    GlobalTilePos(points[i].round().as_ivec2()),
                    GlobalTilePos(points[i + 1].round().as_ivec2()),
                    iter * 4096 + i,
                    settings,
                    dimension_hash,
                ) * curve_jitter_tiles;
                points[i] += normal * jitter;
            }
        }
    }

    let mut smoothed = Vec::with_capacity(points.len());
    for point in points {
        let tile = GlobalTilePos(point.round().as_ivec2());
        if smoothed.last().copied() == Some(tile) {
            continue;
        }
        smoothed.push(tile);
    }
    if smoothed.len() < 2 {
        return path.to_vec();
    }
    smoothed
}

pub(super) fn plan_river_path_tiles(
    path: &[GlobalTilePos],
    half_width_start: i32,
    half_width_end: i32,
    source_region_pos: RegionPos,
    plan: &mut RiverRegionPlan,
) {
    if path.len() < 2 {
        return;
    }
    let seg_count = (path.len() - 1) as f32;
    for i in 0..path.len() - 1 {
        let t = (i as f32 / seg_count).clamp(0.0, 1.0);
        let half_width = lerp_i32(half_width_start, half_width_end, t).max(1);
        for point in bresenham_line(path[i], path[i + 1]) {
            for oy in -half_width..=half_width {
                for ox in -half_width..=half_width {
                    if ox * ox + oy * oy > half_width * half_width + 1 {
                        continue;
                    }
                    let tile = GlobalTilePos(point.0 + IVec2::new(ox, oy));
                    if !source_region_pos.contains_chunkpos(tile.to_chunkpos()) {
                        continue;
                    }
                    let chunk_pos = tile.to_chunkpos();
                    plan.river_tiles.entry(chunk_pos).or_default().set_gpos(chunk_pos, tile);
                    plan.claimed_chunks.insert(chunk_pos);
                }
            }
        }
    }
}

pub(super) fn path_touches_forbidden_border_chunks(
    path: &[GlobalTilePos],
    half_width_start: i32,
    half_width_end: i32,
    source_region_pos: RegionPos,
) -> bool {
    if path.len() < 2 {
        return false;
    }
    let seg_count = (path.len() - 1) as f32;
    for i in 0..path.len() - 1 {
        let t = (i as f32 / seg_count).clamp(0.0, 1.0);
        let half_width = lerp_i32(half_width_start, half_width_end, t).max(1);
        for point in bresenham_line(path[i], path[i + 1]) {
            for oy in -half_width..=half_width {
                for ox in -half_width..=half_width {
                    if ox * ox + oy * oy > half_width * half_width + 1 {
                        continue;
                    }
                    let tile = GlobalTilePos(point.0 + IVec2::new(ox, oy));
                    if !source_region_pos.contains_chunkpos(tile.to_chunkpos()) {
                        continue;
                    }
                    if is_near_region_border(tile, Some(source_region_pos)) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub(super) fn path_has_nonconsecutive_overlap(path: &[GlobalTilePos]) -> bool {
    if path.len() < 3 {
        return false;
    }

    let mut first_seen_segment_i: HashMap<GlobalTilePos, usize> = HashMap::default();
    for segment_i in 0..path.len() - 1 {
        for point in bresenham_line(path[segment_i], path[segment_i + 1]) {
            let Some(&first_segment_i) = first_seen_segment_i.get(&point) else {
                first_seen_segment_i.insert(point, segment_i);
                continue;
            };
            if first_segment_i.saturating_add(1) < segment_i {
                return true;
            }
        }
    }
    false
}

pub(super) fn maybe_add_river_delta(
    mouth: GlobalTilePos,
    main_path: &[GlobalTilePos],
    half_width: i32,
    delta_chance: f32,
    delta_spread: i32,
    source_region_pos: RegionPos,
    plan: &mut RiverRegionPlan,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
) {
    if delta_chance <= 0.0 || main_path.len() < 2 {
        return;
    }
    let hash = mouth.hash_value(settings, dimension_hash, 777);
    let roll = (hash % 10_000) as f32 / 10_000.0;
    if roll > delta_chance {
        return;
    }

    let prev = main_path[main_path.len() - 2];
    let flow = mouth.0 - prev.0;
    let branch_dirs = [IVec2::new(-flow.y, flow.x), IVec2::new(flow.y, -flow.x)];
    for branch_dir in branch_dirs {
        if branch_dir == IVec2::ZERO {
            continue;
        }
        let branch_target = GlobalTilePos(mouth.0 + branch_dir.signum() * delta_spread.max(1));
        let branch_path = bresenham_line(mouth, branch_target);
        if path_touches_forbidden_border_chunks(branch_path.as_slice(), half_width.max(1), half_width.saturating_add(1), source_region_pos) {
            continue;
        }
        plan_river_path_tiles(branch_path.as_slice(), half_width.max(1), half_width.saturating_add(1), source_region_pos, plan);
    }
}

pub(super) fn maybe_add_river_island_gap(
    path: &[GlobalTilePos],
    half_width_start: i32,
    half_width_end: i32,
    source_region_pos: RegionPos,
    plan: &mut RiverRegionPlan,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    source: GlobalTilePos,
) {
    if path.len() < 12 {
        return;
    }

    let center_i = path.len() / 2;
    let center_roll = river_noise_signed(
        source,
        path[center_i.saturating_sub(1)],
        path[center_i],
        center_i,
        settings,
        dimension_hash,
    );
    if center_roll < -0.25 {
        return;
    }

    let (split_flow, split_normal) = path_frame_dirs(path, center_i);
    if split_flow == IVec2::ZERO {
        return;
    }

    let split_half_width = 13 + (source.hash_value(settings, dimension_hash, 881) % 5) as i32;
    let split_span = 9 + (source.hash_value(settings, dimension_hash, 883) % 5) as usize;
    let start_i = center_i.saturating_sub(split_span);
    let end_i = (center_i + split_span).min(path.len() - 1);
    let route_start_i = start_i.saturating_sub(1);
    let route_end_i = (end_i + 1).min(path.len() - 1);

    clear_river_path_segment(
        path,
        route_start_i,
        route_end_i,
        half_width_start.max(1),
        half_width_end.max(1),
        source_region_pos,
        &mut plan.river_tiles,
        &mut plan.gravel_tiles,
    );

    let island_offset = split_half_width + half_width_end.max(1) + 15;
    let left_route = build_offset_detour_route(
        path,
        route_start_i,
        route_end_i,
        -1,
        island_offset,
        split_normal,
        settings,
        dimension_hash,
        source,
        1301,
    );
    let right_route = build_offset_detour_route(
        path,
        route_start_i,
        route_end_i,
        1,
        island_offset,
        split_normal,
        settings,
        dimension_hash,
        source,
        1701,
    );

    if left_route.len() >= 2 && !path_touches_forbidden_border_chunks(&left_route, half_width_start, half_width_end, source_region_pos) {
        plan_river_path_tiles(&left_route, half_width_start, half_width_end, source_region_pos, plan);
    }
    if right_route.len() >= 2 && !path_touches_forbidden_border_chunks(&right_route, half_width_start, half_width_end, source_region_pos) {
        plan_river_path_tiles(&right_route, half_width_start, half_width_end, source_region_pos, plan);
    }
}

pub(super) fn maybe_add_river_gravel_deposits(
    path: &[GlobalTilePos],
    half_width_end: i32,
    source_region_pos: RegionPos,
    plan: &mut RiverRegionPlan,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    source: GlobalTilePos,
    gravel_deposit_max: usize,
    gravel_deposit_spacing: usize,
) {
    if path.len() < 2 || half_width_end < 1 {
        return;
    }

    let gravel_segment_limit = (((path.len() - 1) as f32) * 0.9).floor().max(1.0) as usize;
    let gravel_deposit_max = gravel_deposit_max.min(gravel_segment_limit).max(1);
    let mut deposits_added = 0usize;

    for (step_i, segment) in path.windows(2).enumerate().take(gravel_segment_limit) {
        if deposits_added >= gravel_deposit_max {
            break;
        }
        let from = segment[0];
        let (flow, normal) = path_frame_dirs(path, step_i);
        if flow == IVec2::ZERO {
            continue;
        }
        let bank_side = if source.hash_value(settings, dimension_hash, 997 + step_i as u64) & 1 == 0 { 1 } else { -1 };
        let anchor = GlobalTilePos(from.0 + normal * bank_side * (half_width_end.max(1) + 1));
        let deposit_phase = source.hash_value(settings, dimension_hash, 1091 + step_i as u64) % gravel_deposit_spacing as u64;
        if deposit_phase != 0 {
            continue;
        }

        let base_radius = 3 + (source.hash_value(settings, dimension_hash, 991 + step_i as u64) % 3) as i32;
        for offset_i in -1..=1 {
            let along = flow * offset_i;
            let side_bias = normal * bank_side * (1 + (source.hash_value(settings, dimension_hash, 1601 + step_i as u64 + offset_i.unsigned_abs() as u64) % 2) as i32);
            let center = GlobalTilePos(anchor.0 + along + side_bias);
            let radius = base_radius + (source.hash_value(settings, dimension_hash, 1701 + step_i as u64 + offset_i.unsigned_abs() as u64) % 2) as i32;
            stamp_blob_disk(&mut plan.gravel_tiles, &mut plan.claimed_chunks, center, radius, source_region_pos);
        }
        deposits_added = deposits_added.saturating_add(1);
    }
}

pub(super) fn closest_point_on_path(source: GlobalTilePos, path: &[GlobalTilePos]) -> Option<GlobalTilePos> {
    path.iter().copied().min_by_key(|point| source.distance_squared(point))
}

pub(super) fn segment_reenters_visited_path(
    from: GlobalTilePos,
    to: GlobalTilePos,
    visited: &HashSet<GlobalTilePos>,
) -> bool {
    let segment = bresenham_line(from, to);
    if segment.len() <= 2 {
        return false;
    }

    segment
        .iter()
        .skip(1)
        .take(segment.len().saturating_sub(2))
        .any(|point| visited.contains(point))
}

pub(super) fn bresenham_line(from: GlobalTilePos, to: GlobalTilePos) -> Vec<GlobalTilePos> {
    let mut points = Vec::new();
    let mut x0 = from.0.x;
    let mut y0 = from.0.y;
    let x1 = to.0.x;
    let y1 = to.0.y;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push(GlobalTilePos(IVec2::new(x0, y0)));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    points
}

pub(super) fn lerp_i32(a: i32, b: i32, t: f32) -> i32 {
    ((a as f32) + ((b - a) as f32) * t).round() as i32
}

fn path_frame_dirs(path: &[GlobalTilePos], i: usize) -> (IVec2, IVec2) {
    let prev = path[i.saturating_sub(1)];
    let next = path[(i + 1).min(path.len() - 1)];
    let flow = (next.0 - prev.0).signum();
    let normal = IVec2::new(-flow.y, flow.x);
    (flow, normal)
}

fn build_offset_detour_route(
    path: &[GlobalTilePos],
    start_i: usize,
    end_i: usize,
    side_sign: i32,
    offset_peak: i32,
    normal: IVec2,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    source: GlobalTilePos,
    salt: u64,
) -> Vec<GlobalTilePos> {
    if end_i <= start_i || end_i >= path.len() {
        return Vec::new();
    }

    if normal == IVec2::ZERO {
        return Vec::new();
    }

    let span = (end_i - start_i).max(1) as f32;
    let mut route = Vec::with_capacity(end_i - start_i + 1);
    for i in start_i..=end_i {
        let t = (i - start_i) as f32 / span;
        let arch = (std::f32::consts::PI * t).sin().powf(4.0);
        let jitter = (source.hash_value(settings, dimension_hash, salt + i as u64) % 3) as i32 - 1;
        let offset = ((offset_peak as f32 * arch) + (jitter as f32 * arch * 0.35)).round() as i32;
        route.push(GlobalTilePos(path[i].0 + normal * side_sign * offset));
    }
    route
}

fn clear_river_path_segment(
    path: &[GlobalTilePos],
    start_i: usize,
    end_i: usize,
    half_width_start: i32,
    half_width_end: i32,
    source_region_pos: RegionPos,
    river_mask_map: &mut HashMap<ChunkPos, ChunkGposMask>,
    gravel_mask_map: &mut HashMap<ChunkPos, ChunkGposMask>,
) {
    if path.len() < 2 || end_i <= start_i || end_i >= path.len() {
        return;
    }

    let seg_count = (end_i - start_i) as f32;
    for i in start_i..end_i {
        let t = (i - start_i) as f32 / seg_count;
        let base_half_width = lerp_i32(half_width_start, half_width_end, t).max(1);
        let neck = (std::f32::consts::PI * t).sin().powf(1.65);
        let edge_distance = t.min(1.0 - t);
        let endpoint_bonus = (((0.18 - edge_distance) / 0.18).clamp(0.0, 1.0).powf(1.4) * 2.0).round() as i32;
        let half_width = ((base_half_width as f32) * (0.78 + 0.22 * neck)).round().max(1.0) as i32 + endpoint_bonus;
        for point in bresenham_line(path[i], path[i + 1]) {
            for oy in -half_width..=half_width {
                for ox in -half_width..=half_width {
                    if ox * ox + oy * oy > half_width * half_width + 1 {
                        continue;
                    }
                    let tile = GlobalTilePos(point.0 + IVec2::new(ox, oy));
                    if !source_region_pos.contains_chunkpos(tile.to_chunkpos()) {
                        continue;
                    }
                    clear_mask_tile(river_mask_map, tile);
                    clear_mask_tile(gravel_mask_map, tile);
                }
            }
        }
    }
}

fn stamp_blob_disk(
    mask_map: &mut HashMap<ChunkPos, ChunkGposMask>,
    claimed_chunks: &mut HashSet<ChunkPos>,
    center: GlobalTilePos,
    radius: i32,
    source_region_pos: RegionPos,
) {
    for oy in -radius..=radius {
        for ox in -radius..=radius {
            if ox * ox + oy * oy > radius * radius {
                continue;
            }
            let tile = GlobalTilePos(center.0 + IVec2::new(ox, oy));
            if !source_region_pos.contains_chunkpos(tile.to_chunkpos()) {
                continue;
            }
            insert_mask_tile(mask_map, tile);
            claimed_chunks.insert(tile.to_chunkpos());
        }
    }
}

fn insert_mask_tile(mask_map: &mut HashMap<ChunkPos, ChunkGposMask>, tile: GlobalTilePos) {
    let chunk_pos = tile.to_chunkpos();
    mask_map.entry(chunk_pos).or_default().set_gpos(chunk_pos, tile);
}

fn clear_mask_tile(mask_map: &mut HashMap<ChunkPos, ChunkGposMask>, tile: GlobalTilePos) {
    let chunk_pos = tile.to_chunkpos();
    let Some(mask) = mask_map.get_mut(&chunk_pos) else {
        return;
    };
    mask.clear_gpos(chunk_pos, tile);
    if mask.is_empty() {
        mask_map.remove(&chunk_pos);
    }
}

pub(super) fn rebuild_claimed_chunks_from_masks(plan: &mut RiverRegionPlan) {
    plan.claimed_chunks.clear();
    plan.claimed_chunks.extend(plan.river_tiles.keys().copied());
    plan.claimed_chunks.extend(plan.gravel_tiles.keys().copied());
}
