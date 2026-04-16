use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use common::common_components::HashId;
use tilemap_shared::{GlobalGenSettings, GlobalTilePos, RegionPos};
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

pub(super) fn extend_river_mouth(
    main_path: &[GlobalTilePos],
    half_width_end: i32,
    ocean_target: Option<GlobalTilePos>,
    source_region_pos: RegionPos,
    plan: &mut RiverRegionPlan,
) {
    let Some(&mouth) = main_path.last() else {
        return;
    };
    let Some(&prev) = main_path.get(main_path.len().saturating_sub(2)) else {
        return;
    };

    let flow = mouth.0 - prev.0;
    let forward = IVec2::new(flow.x.signum(), flow.y.signum());
    if forward == IVec2::ZERO {
        return;
    }

    let flank = IVec2::new(-forward.y, forward.x);
    let mouth_extension_steps = 4_i32;
    let flare_width = half_width_end.saturating_add(1).max(half_width_end);
    let ocean_target = ocean_target.unwrap_or(GlobalTilePos(mouth.0 + forward * mouth_extension_steps));
    let targets = [
        (GlobalTilePos(ocean_target.0), flare_width),
        (GlobalTilePos(mouth.0 + forward * (mouth_extension_steps - 1) + flank), half_width_end),
        (GlobalTilePos(mouth.0 + forward * (mouth_extension_steps - 1) - flank), half_width_end),
        (GlobalTilePos(mouth.0 + forward * (mouth_extension_steps - 2) + flank), half_width_end.saturating_sub(1).max(1)),
        (GlobalTilePos(mouth.0 + forward * (mouth_extension_steps - 2) - flank), half_width_end.saturating_sub(1).max(1)),
    ];

    for (target, target_half_width_end) in targets {
        let flare_path = bresenham_line(mouth, target);
        if path_touches_forbidden_border_chunks(flare_path.as_slice(), half_width_end.max(1), target_half_width_end, source_region_pos) {
            continue;
        }
        plan_river_path_tiles(flare_path.as_slice(), half_width_end.max(1), target_half_width_end, source_region_pos, plan);
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