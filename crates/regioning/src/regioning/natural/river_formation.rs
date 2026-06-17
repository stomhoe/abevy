use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use common::common_components::HashId;
use common::log_targets::RIVER_SYSTEM;
use rand::SeedableRng;
use std::collections::VecDeque;
use tilemap_shared::*;

use super::river_components::{RiverMouthRejectReason, RiverRegionDebugInfo, RiverRegionPlan};
use super::river_sculpting_helpers::{closest_point_on_path, maybe_add_river_delta, maybe_add_river_gravel_deposits, maybe_add_river_island_gap, path_has_nonconsecutive_overlap, path_touches_forbidden_border_chunks, plan_river_path_tiles, rebuild_claimed_chunks_from_masks, river_noise_signed, segment_reenters_visited_path, smooth_river_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RiverBuildRejectReason {
    NoMainMouth,
    MainPathTooShort,
    MainPathOverlap,
    MainPathForbiddenBorder,
    TributaryTooShort,
    TributaryOverlap,
}

pub(super) fn generate_river_region_plan(
    inland_map: &HashMap<GlobalTilePos, f32>,
    coast_points: &HashSet<GlobalTilePos>,
    cfg: &StructuredGenConfig,
    settings: &GlobalGenSettings,
    dimension_ref: DimensionRef,
    source_region_pos: RegionPos,
    plan: &mut RiverRegionPlan,
    region_debug: &mut RiverRegionDebugInfo,
) -> bool {
    plan.claimed_chunks.clear();
    plan.river_tiles.clear();
    plan.gravel_tiles.clear();

    region_debug.river_source_points.clear();
    region_debug.river_mouth_points.clear();
    region_debug.failed_probe_points.clear();

    if inland_map.is_empty() {
        error!(target: RIVER_SYSTEM, "river generation failed for region {:?} in dimension {:?}: no inlandness points were provided", source_region_pos, dimension_ref);
        return false;
    }

    let land_threshold: f32 = cfg.args.parse_arg("river_land_threshold", -0.05);
    let source_min_inlandness: f32 = cfg.args.parse_arg("river_source_min_inlandness", 0.2);
    let delta_chance: f32 = cfg.args.parse_arg("river_delta_chance", 0.18_f32).clamp(0.0, 1.0);
    let delta_spread: i32 = cfg.args.parse_arg("river_delta_spread", 1_i32).max(1);
    let max_sources: usize = cfg.args.parse_arg("river_max_sources", 2_usize).max(1);
    let max_tributaries: usize = cfg.args.parse_arg("river_max_tributaries", 1_usize).max(0);
    let max_tributaries = max_tributaries.min(max_sources.saturating_sub(1));
    let max_sources_for_selection = max_tributaries.saturating_add(1);
    let max_steps: usize = cfg.args.parse_arg("river_worm_length", 240_usize).max(8);
    let source_stride: u64 = cfg.args.parse_arg("river_source_hash_stride", 19_u64).max(1);
    let source_mouth_min_distance: usize = cfg.args.parse_arg("river_source_mouth_min_distance", 8_usize).max(1);
    let default_half_width_start: i32 = cfg.args.parse_arg("river_main_half_width_start", 2_i32).max(1);
    let default_half_width_end: i32 = cfg.args.parse_arg("river_main_half_width_end", 4_i32).max(default_half_width_start);
    let main_width_dist = parse_capped_normal_dist_arg(&cfg.typed_args, "river_main_half_width_normal_dist");
    let min_island_area_chunks: f32 = cfg.args.parse_arg("river_min_island_area_chunks", 170.0_f32).max(1.0);    let river_gravel_deposit_max: usize = cfg.args.parse_arg("river_gravel_deposit_max", 12_usize).max(1);
    let river_gravel_deposit_spacing: usize = cfg.args.parse_arg("river_gravel_deposit_spacing", 2_usize).max(1);
    let land_points: HashMap<GlobalTilePos, f32> = inland_map
        .iter()
        .filter(|(pos, _)| !coast_points.contains(*pos) && source_region_pos.contains_chunkpos(pos.to_chunkpos()))
        .map(|(pos, val)| (*pos, *val))
        .collect();
    if land_points.is_empty() {
        error!(target: RIVER_SYSTEM, "river generation failed for region {:?} in dimension {:?}: no inland land points remained after filtering ocean samples and region bounds", source_region_pos, dimension_ref);
        return false;
    }

    let spacing = estimate_sample_spacing(&land_points);
    let (component_of, component_sizes) = build_land_components(&land_points, land_threshold, spacing);
    let sample_area_chunks = ((spacing as f32 / ChunkPos::CHUNK_SIZE.x as f32) * (spacing as f32 / ChunkPos::CHUNK_SIZE.y as f32)).max(0.001);
    let min_component_samples = (min_island_area_chunks / sample_area_chunks).ceil() as usize;
    let source_min_separation_steps: usize = cfg.args.parse_arg("river_source_min_separation_steps", 10_usize).max(1);
    let min_source_distance_tiles = (spacing * source_min_separation_steps as i32).max(spacing);
    let sources = pick_river_sources(
        &land_points,
        &component_of,
        &component_sizes,
        min_component_samples.max(1),
        source_min_inlandness,
        spacing,
        settings,
        dimension_ref.0,
        source_stride,
        max_sources_for_selection,
        min_source_distance_tiles,
        Some(source_region_pos),
    );
    if sources.is_empty() {
        error!(target: RIVER_SYSTEM, "river generation failed for region {:?} in dimension {:?}: source selection produced no river sources", source_region_pos, dimension_ref);
        return false;
    }

    let trace_params = RiverTraceParams {
        neighbor_radius: cfg.args.parse_arg("river_trace_neighbor_radius", 2_i32).clamp(1, 4),
        directional_inertia: cfg.args.parse_arg("river_directional_inertia", 0.75_f32).clamp(0.0, 2.0),
        downhill_weight: cfg.args.parse_arg("river_downhill_weight", 3.0_f32).max(0.1),
        uphill_penalty: cfg.args.parse_arg("river_uphill_penalty", 2.5_f32).max(0.0),
    };
    let curve_iterations: usize = cfg.args.parse_arg("river_curve_iterations", 2_usize).min(4);
    let curve_jitter_tiles: f32 = cfg.args.parse_arg("river_curve_jitter_tiles", (spacing as f32 * 0.30).clamp(1.0, 8.0));
    let river_zigzag_chance: f32 = cfg.args.parse_arg("river_zigzag_chance", 0.25_f32).clamp(0.0, 1.0);
    let river_zigzag_min_path_len: usize = cfg.args.parse_arg("river_zigzag_min_path_len", 80_usize).max(1);
    let river_zigzag_extra_iterations: usize = cfg.args.parse_arg("river_zigzag_extra_iterations", 1_usize).min(3);
    let river_zigzag_extra_jitter: f32 = cfg.args.parse_arg("river_zigzag_extra_jitter", 2.5_f32).max(0.0);

    let mut source_reject_stats: HashMap<RiverBuildRejectReason, u32> = HashMap::default();
    let mut built_river = false;
    'source_loop: for main_source in sources.iter().copied() {
        let Some(&main_component_i) = component_of.get(&main_source) else {
            continue;
        };
        let (half_width_start, half_width_end) = if let Some(dist) = &main_width_dist {
            let seed = main_source.hash_value(settings, dimension_ref.0, 1_643);
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let sampled_half_width = dist.sample(&mut rng).round().clamp(3.0, 8.0) as i32;
            (sampled_half_width.max(1), sampled_half_width.max(1))
        } else {
            (default_half_width_start, default_half_width_end)
        };
        let Some((mouth_target, _mouth_ocean_target)) = pick_river_mouth(
            main_source,
            &land_points,
            coast_points,
            &component_of,
            main_component_i,
            source_mouth_min_distance,
            region_debug,
            source_region_pos,
            spacing,
        ) else {
            *source_reject_stats.entry(RiverBuildRejectReason::NoMainMouth).or_insert(0) += 1;
            continue;
        };

        let candidate_main_path = trace_downhill_path(
            main_source,
            &land_points,
            &component_of,
            main_component_i,
            spacing,
            max_steps,
            &trace_params,
            settings,
            dimension_ref.0,
            None,
            Some(mouth_target),
        );
        if candidate_main_path.len() < 2 || !path_reaches_target(&candidate_main_path, mouth_target, spacing) {
            *source_reject_stats.entry(RiverBuildRejectReason::MainPathTooShort).or_insert(0) += 1;
            continue;
        }

        let zigzag_selected = candidate_main_path.len() >= river_zigzag_min_path_len && {
            let roll = (main_source.hash_value(settings, dimension_ref.0, 514_379) % 10_000) as f32 / 10_000.0;
            roll < river_zigzag_chance
        };
        let main_curve_iterations = curve_iterations.saturating_add(if zigzag_selected { river_zigzag_extra_iterations } else { 0 }).min(8);
        let main_curve_jitter = if zigzag_selected {
            curve_jitter_tiles + river_zigzag_extra_jitter
        } else {
            curve_jitter_tiles
        };

        let smoothed_main_path = smooth_river_path(
            &candidate_main_path,
            main_curve_iterations,
            main_curve_jitter,
            settings,
            dimension_ref.0,
            main_source,
        );
        if path_has_nonconsecutive_overlap(&smoothed_main_path) {
            *source_reject_stats.entry(RiverBuildRejectReason::MainPathOverlap).or_insert(0) += 1;
            continue;
        }
        if path_touches_forbidden_border_chunks(
            &smoothed_main_path,
            half_width_start,
            half_width_end,
            source_region_pos,
        ) {
            *source_reject_stats.entry(RiverBuildRejectReason::MainPathForbiddenBorder).or_insert(0) += 1;
            continue;
        }

        plan_river_path_tiles(smoothed_main_path.as_slice(), half_width_start, half_width_end, source_region_pos, plan);
        maybe_add_river_delta(mouth_target, &smoothed_main_path, half_width_end, delta_chance, delta_spread, source_region_pos, plan, settings, dimension_ref.0);
        region_debug.river_source_points.insert(main_source);
        region_debug.river_mouth_points.insert(mouth_target);

        for source in sources.iter().skip(1).copied() {
            let Some(&source_component_i) = component_of.get(&source) else {
                continue;
            };
            let merge_target = closest_point_on_path(source, &candidate_main_path).unwrap_or(mouth_target);
            let path = trace_downhill_path(
                source,
                &land_points,
                &component_of,
                source_component_i,
                spacing,
                max_steps,
                &trace_params,
                settings,
                dimension_ref.0,
                None,
                Some(merge_target),
            );
            if path.len() < 2 {
                *source_reject_stats.entry(RiverBuildRejectReason::TributaryTooShort).or_insert(0) += 1;
                continue;
            }
            let smoothed_path = smooth_river_path(&path, curve_iterations, curve_jitter_tiles, settings, dimension_ref.0, source);
            let tributary_half_start = half_width_start.saturating_sub(1).max(1);
            let tributary_half_end = half_width_end.saturating_sub(1).max(1);
            if path_has_nonconsecutive_overlap(&smoothed_path) {
                *source_reject_stats.entry(RiverBuildRejectReason::TributaryOverlap).or_insert(0) += 1;
                continue;
            }
            region_debug.river_source_points.insert(source);
            region_debug.river_mouth_points.insert(*path.last().unwrap_or(&merge_target));
            plan_river_path_tiles(smoothed_path.as_slice(), tributary_half_start, tributary_half_end, source_region_pos, plan);
        }

        maybe_add_river_island_gap(&smoothed_main_path, half_width_start, half_width_end, source_region_pos, plan, settings, dimension_ref.0, main_source);
        maybe_add_river_gravel_deposits(
            &smoothed_main_path,
            half_width_end,
            source_region_pos,
            plan,
            settings,
            dimension_ref.0,
            main_source,
            river_gravel_deposit_max,
            river_gravel_deposit_spacing,
        );
        rebuild_claimed_chunks_from_masks(plan);

        built_river = true;
        break 'source_loop;
    }

    if !built_river {
        let mouth_fail_summary = format_top_mouth_reject_causes(&region_debug.mouth_reject_stats);
        let build_fail_summary = format_river_build_reject_causes(&source_reject_stats);
        error!(target: RIVER_SYSTEM, "river generation failed for region {:?} in dimension {:?}: no river was built; mouth rejection top causes: {}; build rejection stats: {}", source_region_pos, dimension_ref, mouth_fail_summary, build_fail_summary);
        return false;
    }

    if plan.river_tiles.is_empty() {
        error!(target: RIVER_SYSTEM, "river generation failed for region {:?} in dimension {:?}: river plan generation finished without producing any river tiles", source_region_pos, dimension_ref);
        return false;
    }

    true
}

fn estimate_sample_spacing(sampled_points: &HashMap<GlobalTilePos, f32>) -> i32 {
    let mut xs: Vec<i32> = sampled_points.keys().map(|p| p.0.x).collect();
    let mut ys: Vec<i32> = sampled_points.keys().map(|p| p.0.y).collect();
    xs.sort_unstable();
    ys.sort_unstable();
    xs.dedup();
    ys.dedup();
    let min_dx = xs.windows(2).map(|w| w[1] - w[0]).filter(|d| *d > 0).min().unwrap_or(64);
    let min_dy = ys.windows(2).map(|w| w[1] - w[0]).filter(|d| *d > 0).min().unwrap_or(64);
    min_dx.min(min_dy).max(1)
}

fn build_land_components(
    sampled_points: &HashMap<GlobalTilePos, f32>,
    land_threshold: f32,
    spacing: i32,
) -> (HashMap<GlobalTilePos, usize>, Vec<usize>) {
    let mut component_of: HashMap<GlobalTilePos, usize> = HashMap::default();
    let mut component_sizes: Vec<usize> = Vec::new();
    for (&start, &val) in sampled_points.iter() {
        if val < land_threshold || component_of.contains_key(&start) {
            continue;
        }
        let component_i = component_sizes.len();
        component_sizes.push(0);
        let mut queue = VecDeque::new();
        queue.push_back(start);
        component_of.insert(start, component_i);
        while let Some(curr) = queue.pop_front() {
            component_sizes[component_i] = component_sizes[component_i].saturating_add(1);
            for oy in -1..=1 {
                for ox in -1..=1 {
                    if ox == 0 && oy == 0 {
                        continue;
                    }
                    let next = GlobalTilePos(curr.0 + IVec2::new(ox * spacing, oy * spacing));
                    if component_of.contains_key(&next) {
                        continue;
                    }
                    let Some(&next_val) = sampled_points.get(&next) else {
                        continue;
                    };
                    if next_val < land_threshold {
                        continue;
                    }
                    component_of.insert(next, component_i);
                    queue.push_back(next);
                }
            }
        }
    }
    (component_of, component_sizes)
}

fn is_local_maximum(pos: GlobalTilePos, val: f32, sampled_points: &HashMap<GlobalTilePos, f32>, spacing: i32) -> bool {
    for oy in -1..=1 {
        for ox in -1..=1 {
            if ox == 0 && oy == 0 {
                continue;
            }
            let next = GlobalTilePos(pos.0 + IVec2::new(ox * spacing, oy * spacing));
            if let Some(&next_val) = sampled_points.get(&next) && next_val > val {
                return false;
            }
        }
    }
    true
}

fn pick_river_sources(
    sampled_points: &HashMap<GlobalTilePos, f32>,
    component_of: &HashMap<GlobalTilePos, usize>,
    component_sizes: &[usize],
    min_component_samples: usize,
    source_min_inlandness: f32,
    spacing: i32,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    source_stride: u64,
    max_sources: usize,
    min_source_distance_tiles: i32,
    local_region_pos: Option<RegionPos>,
) -> Vec<GlobalTilePos> {
    let mut candidates: Vec<(GlobalTilePos, f32)> = Vec::new();
    for (&pos, &val) in sampled_points.iter() {
        if let Some(region_pos) = local_region_pos && !region_pos.contains_chunkpos(pos.to_chunkpos()) {
            continue;
        }
        if is_near_region_border(pos, local_region_pos, ) {
            continue;
        }
        if val < source_min_inlandness {
            continue;
        }
        let Some(&component_i) = component_of.get(&pos) else {
            continue;
        };
        if component_sizes.get(component_i).copied().unwrap_or_default() < min_component_samples {
            continue;
        }
        if !is_local_maximum(pos, val, sampled_points, spacing) {
            continue;
        }
        if source_stride > 1 && pos.hash_value(settings, dimension_hash, 91) % source_stride != 0 {
            continue;
        }
        candidates.push((pos, val));
    }
    candidates.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.0.y.cmp(&b.0.0.y))
            .then_with(|| a.0.0.x.cmp(&b.0.0.x))
    });

    if candidates.is_empty() {
        error!(target: RIVER_SYSTEM, "river source selection produced no valid river sources after applying the inlandness, island-size, stride, and border filters; region={:?}", local_region_pos);
        return Vec::new();
    }

    select_spread_sources(
        candidates.into_iter().map(|(p, _)| p).collect::<Vec<_>>(),
        max_sources.max(1),
        min_source_distance_tiles.max(1),
    )
}

#[derive(Clone, Copy)]
struct RiverTraceParams {
    neighbor_radius: i32,
    directional_inertia: f32,
    downhill_weight: f32,
    uphill_penalty: f32,
}

pub(super) fn is_near_region_border(pos: GlobalTilePos, region_pos: Option<RegionPos>, ) -> bool {
    let Some(region_pos) = region_pos else {
        return false;
    };

    let (min, max) = region_pos.chunk_bounds();
    let min_tile = min.to_tilepos();
    let max_tile_excl = max.to_tilepos();
    let border_margin_tiles = 1 * ChunkPos::CHUNK_SIZE.x as i32;
    pos.0.x < min_tile.0.x + border_margin_tiles
        || pos.0.y < min_tile.0.y + border_margin_tiles
        || pos.0.x >= max_tile_excl.0.x - border_margin_tiles
        || pos.0.y >= max_tile_excl.0.y - border_margin_tiles
}

fn trace_downhill_path(
    source: GlobalTilePos,
    sampled_points: &HashMap<GlobalTilePos, f32>,
    component_of: &HashMap<GlobalTilePos, usize>,
    source_component_i: usize,
    spacing: i32,
    max_steps: usize,
    trace_params: &RiverTraceParams,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
    blocked_chunks: Option<&HashSet<ChunkPos>>,
    mouth_target: Option<GlobalTilePos>,
) -> Vec<GlobalTilePos> {
    let mut path = vec![source];
    let mut visited: HashSet<GlobalTilePos> = HashSet::default();
    visited.insert(source);
    let mut reroute_retry_count = 0_usize;
    let mut reroute_steps_used = 0_i32;
    let target_reach_tiles = spacing.saturating_mul(5) as i64;
    let target_reach_dist_sq = target_reach_tiles * target_reach_tiles;

    for step in 0..max_steps {
        let curr = *path.last().unwrap_or(&source);
        if let Some(target) = mouth_target {
            if curr == target {
                break;
            }
            if curr.distance_squared(&target) as i64 <= target_reach_dist_sq {
                path.push(target);
                break;
            }
        }
        let curr_val = sampled_points.get(&curr).copied().unwrap_or(0.0);
        let prev_dir = if path.len() >= 2 {
            let prev = path[path.len() - 2];
            (curr.0 - prev.0).as_vec2().normalize_or_zero()
        } else {
            Vec2::ZERO
        };

        let mut best_scored: Option<(GlobalTilePos, f32)> = None;
        for oy in -trace_params.neighbor_radius..=trace_params.neighbor_radius {
            for ox in -trace_params.neighbor_radius..=trace_params.neighbor_radius {
                if ox == 0 && oy == 0 {
                    continue;
                }
                let next = GlobalTilePos(curr.0 + IVec2::new(ox * spacing, oy * spacing));
                if visited.contains(&next) {
                    continue;
                }
                if blocked_chunks.is_some_and(|blocked| blocked.contains(&next.to_chunkpos())) {
                    continue;
                }
                if component_of.get(&next).copied() != Some(source_component_i) {
                    continue;
                }
                let Some(&next_val) = sampled_points.get(&next) else {
                    continue;
                };

                let delta = curr_val - next_val;
                let move_dir = IVec2::new(ox, oy).as_vec2().normalize_or_zero();
                let mut score = delta * trace_params.downhill_weight;
                if delta < 0.0 {
                    score -= (-delta) * trace_params.uphill_penalty;
                }
                if prev_dir.length_squared() > 0.0 {
                    score += prev_dir.dot(move_dir) * trace_params.directional_inertia;
                }
                score += river_noise_signed(source, curr, next, step, settings, dimension_hash) * 0.35;
                if let Some(target) = mouth_target {
                    let goal_dir = (target.0 - next.0).as_vec2().normalize_or_zero();
                    score += move_dir.dot(goal_dir) * 1.5;
                }
                score -= (ox.abs().max(oy.abs()) as f32 - 1.0).max(0.0) * 0.08;

                if best_scored.as_ref().map_or(true, |(_, best_score)| score > *best_score) {
                    best_scored = Some((next, score));
                }
            }
        }
        let Some((next, used_reroute, detour_steps)) = best_scored
            .map(|(next, _)| (next, false, 0_i32))
            .or_else(|| {
                let fallback = find_fallback_flow_target(
                    curr,
                    curr_val,
                    sampled_points,
                    component_of,
                    source_component_i,
                    &visited,
                    spacing,
                    4,
                    blocked_chunks,
                    mouth_target,
                )?;
                Some((fallback.0, true, fallback.2))
            })
        else {
            break;
        };
        if used_reroute && segment_reenters_visited_path(curr, next, &visited) {
            error!(target: RIVER_SYSTEM, "river trace for source {:?} would detour back through its own path at {:?} -> {:?}; cancelling", source, curr, next);
            break;
        }
        if used_reroute {
            if reroute_retry_count >= 1 || reroute_steps_used.saturating_add(detour_steps) > 4 {
                break;
            }
            reroute_retry_count = reroute_retry_count.saturating_add(1);
            reroute_steps_used = reroute_steps_used.saturating_add(detour_steps);
        }
        visited.insert(next);
        path.push(next);
    }

    path
}

fn path_reaches_target(path: &[GlobalTilePos], target: GlobalTilePos, spacing: i32) -> bool {
    let Some(last) = path.last().copied() else {
        return false;
    };
    let reach_tiles = spacing.saturating_mul(5) as u64;
    let reach_dist_sq = reach_tiles * reach_tiles;
    last == target || last.distance_squared(&target) <= reach_dist_sq
}

fn pick_river_mouth(
    source: GlobalTilePos,
    sampled_points: &HashMap<GlobalTilePos, f32>,
    coast_points: &HashSet<GlobalTilePos>,
    component_of: &HashMap<GlobalTilePos, usize>,
    source_component_i: usize,
    min_distance_tiles: usize,
    region_debug: &mut RiverRegionDebugInfo,
    local_region_pos: RegionPos,
    spacing: i32,
) -> Option<(GlobalTilePos, GlobalTilePos)> {
    let min_distance_tiles = min_distance_tiles as i32;
    let mut candidates: Vec<(GlobalTilePos, GlobalTilePos, f32, i32)> = Vec::new();
    let mut lowest_val: Option<f32> = None;

    let mut record_reject = |reason: RiverMouthRejectReason| {
        region_debug.mouth_reject_stats.total_rejections = region_debug.mouth_reject_stats.total_rejections.saturating_add(1);
        *region_debug.mouth_reject_stats.counts.entry(reason).or_insert(0) = region_debug
            .mouth_reject_stats
            .counts
            .get(&reason)
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
    };

    for (&pos, &val) in sampled_points {
        lowest_val = Some(lowest_val.map_or(val, |curr_lowest| curr_lowest.min(val)));
        if !local_region_pos.contains_chunkpos(pos.to_chunkpos()) {
            record_reject(RiverMouthRejectReason::OutsideRegion);
            continue;
        }
        if is_near_region_border(pos, Some(local_region_pos), ) {
            record_reject(RiverMouthRejectReason::NearBorder);
            continue;
        }
        if component_of.get(&pos).copied() != Some(source_component_i) {
            record_reject(RiverMouthRejectReason::WrongLandComponent);
            continue;
        }
        let Some(ocean_target) = find_adjacent_point_in_set(pos, coast_points, spacing, Some(local_region_pos)) else {
            record_reject(RiverMouthRejectReason::TooInland);
            continue;
        };
        let dist = source.euclidean_tile_distance(pos);
        if dist < min_distance_tiles as f32 {
            record_reject(RiverMouthRejectReason::TooCloseToSource);
            continue;
        }
        let dist = dist as i32;
        candidates.push((pos, ocean_target, val, dist));
    }
    candidates.sort_by(|a, b| {
        a.2.partial_cmp(&b.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.3.cmp(&b.3))
            .then_with(|| a.0.0.y.cmp(&b.0.0.y))
            .then_with(|| a.0.0.x.cmp(&b.0.0.x))
    });
    let result = candidates.first().copied().map(|(pos, ocean_target, val, _)| {
        info!(target: RIVER_SYSTEM, "pick_river_mouth winner_val={:.3}", val);
        let mouth_target = aim_river_mouth_towards_water_body(pos, ocean_target, coast_points, spacing);
        (mouth_target, ocean_target)
    });
    if result.is_none() {
        error!(target: RIVER_SYSTEM, "river mouth selection produced no valid mouth candidates after applying the coast-adjacency and region filters; source={:?} region={:?}; lowest_val={:?}; top causes: {}", source, local_region_pos, lowest_val, format_top_mouth_reject_causes(&region_debug.mouth_reject_stats));
    }
    result
}

fn aim_river_mouth_towards_water_body(
    land_mouth: GlobalTilePos,
    ocean_target: GlobalTilePos,
    coast_points: &HashSet<GlobalTilePos>,
    spacing: i32,
) -> GlobalTilePos {
    let Some((water_body_point, sampled_count)) = find_water_body_center_target(ocean_target, coast_points, spacing) else {
        let mouth_offset = (ocean_target.0 - land_mouth.0).signum() * spacing.max(1);
        return GlobalTilePos(ocean_target.0 + mouth_offset);
    };
    let _ = sampled_count;
    let extra_push_tiles = 8_i32;
    let mouth_direction = (water_body_point.0 - land_mouth.0).signum();
    GlobalTilePos(water_body_point.0 + mouth_direction * extra_push_tiles)
}

fn find_water_body_center_target(
    start: GlobalTilePos,
    points: &HashSet<GlobalTilePos>,
    spacing: i32,
) -> Option<(GlobalTilePos, usize)> {
    if !points.contains(&start) {
        return None;
    }

    let mut visited: HashSet<GlobalTilePos> = HashSet::default();
    let mut queue = VecDeque::new();
    let mut component_points: Vec<GlobalTilePos> = Vec::new();
    let mut sum = IVec2::ZERO;

    visited.insert(start);
    queue.push_back(start);

    while let Some(curr) = queue.pop_front() {
        component_points.push(curr);
        sum += curr.0;

        for oy in -1..=1 {
            for ox in -1..=1 {
                if ox == 0 && oy == 0 {
                    continue;
                }
                let next = GlobalTilePos(curr.0 + IVec2::new(ox * spacing, oy * spacing));
                if visited.contains(&next) || !points.contains(&next) {
                    continue;
                }
                visited.insert(next);
                queue.push_back(next);
            }
        }
    }

    if component_points.is_empty() {
        return None;
    }

    let count = component_points.len();
    let centroid = Vec2::new(sum.x as f32 / count as f32, sum.y as f32 / count as f32);
    let target = component_points
        .into_iter()
        .min_by_key(|point| point.distance_squared(&GlobalTilePos(centroid.round().as_ivec2())))
        .unwrap_or(start);

    Some((target, count))
}

fn format_top_mouth_reject_causes(stats: &super::river_components::RiverMouthRejectStats) -> String {
    if stats.total_rejections == 0 || stats.counts.is_empty() {
        return "none".to_string();
    }

    let total = stats.total_rejections.max(1) as f32;
    let mut entries = stats
        .counts
        .iter()
        .map(|(reason, count)| (*reason, *count))
        .collect::<Vec<_>>();
    entries.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0))));
    entries
        .into_iter()
        .take(2)
        .map(|(reason, count)| format!("{:?}: {} ({:.1}%)", reason, count, (count as f32 / total) * 100.0))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_river_build_reject_causes(stats: &HashMap<RiverBuildRejectReason, u32>) -> String {
    if stats.is_empty() {
        return "none".to_string();
    }

    let mut entries = stats.iter().collect::<Vec<_>>();
    entries.sort_unstable_by(|a, b| b.1.cmp(a.1).then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0))));
    entries
        .into_iter()
        .map(|(reason, count)| format!("{:?}: {}", reason, count))
        .collect::<Vec<_>>()
        .join(", ")
}

fn find_adjacent_point_in_set(
    pos: GlobalTilePos,
    points: &HashSet<GlobalTilePos>,
    spacing: i32,
    local_region_pos: Option<RegionPos>,
) -> Option<GlobalTilePos> {
    let mut best: Option<(GlobalTilePos, i64)> = None;
    let max_radius = 2;
    for radius in 1..=max_radius {
        for oy in -radius..=radius {
            for ox in -radius..=radius {
                if ox == 0 && oy == 0 {
                    continue;
                }
                let next = GlobalTilePos(pos.0 + IVec2::new(ox * spacing, oy * spacing));
                if let Some(region_pos) = local_region_pos
                    && !region_pos.contains_chunkpos(next.to_chunkpos())
                {
                    continue;
                }
                if !points.contains(&next) {
                    continue;
                }
                let dist_sq = (next.0.x - pos.0.x) as i64 * (next.0.x - pos.0.x) as i64
                    + (next.0.y - pos.0.y) as i64 * (next.0.y - pos.0.y) as i64;
                if best.as_ref().map_or(true, |(_, best_dist_sq)| dist_sq < *best_dist_sq) {
                    best = Some((next, dist_sq));
                }
            }
        }
        if best.is_some() {
            return best.map(|(pos, _)| pos);
        }
    }
    None
}

fn parse_capped_normal_dist_arg(args: &SgcArgsDict, key: &str) -> Option<CappedNormalDist> {
    let map = args.get_map(key)?;
    let std_dev = map_get_f32(map, "std_dev")?.max(0.0);
    let mean = map_get_f32(map, "mean")?;

    if let (Some(min), Some(max)) = (map_get_f32(map, "min"), map_get_f32(map, "max")) {
        let min_dev = (mean - min).max(0.0);
        let max_dev = (max - mean).max(0.0);
        return Some(CappedNormalDist::new(min_dev, max_dev, mean, std_dev));
    }

    let min_dev = map_get_f32(map, "min_dev")?;
    let max_dev = map_get_f32(map, "max_dev")?;
    Some(CappedNormalDist::new(min_dev, max_dev, mean, std_dev))
}

fn map_get_f32(map: &HashMap<String, SgcArgValue>, key: &str) -> Option<f32> {
    let value = map.get(key)?;
    match value {
        SgcArgValue::Float(raw) => Some(*raw as f32),
        SgcArgValue::Int(raw) => Some(*raw as f32),
        SgcArgValue::Str(raw) => raw.parse::<f32>().ok(),
        _ => None,
    }
}

fn find_fallback_flow_target(
    curr: GlobalTilePos,
    curr_val: f32,
    sampled_points: &HashMap<GlobalTilePos, f32>,
    component_of: &HashMap<GlobalTilePos, usize>,
    source_component_i: usize,
    visited: &HashSet<GlobalTilePos>,
    spacing: i32,
    max_radius_steps: i32,
    blocked_chunks: Option<&HashSet<ChunkPos>>,
    mouth_target: Option<GlobalTilePos>,
) -> Option<(GlobalTilePos, f32, i32)> {
    let mut best_choice: Option<(GlobalTilePos, f32, i32, f32)> = None;
    for oy in -max_radius_steps..=max_radius_steps {
        for ox in -max_radius_steps..=max_radius_steps {
            if ox == 0 && oy == 0 {
                continue;
            }
            let next = GlobalTilePos(curr.0 + IVec2::new(ox * spacing, oy * spacing));
            if visited.contains(&next) {
                continue;
            }
            if blocked_chunks.is_some_and(|blocked| blocked.contains(&next.to_chunkpos())) {
                continue;
            }
            if component_of.get(&next).copied() != Some(source_component_i) {
                continue;
            }
            let Some(&next_val) = sampled_points.get(&next) else {
                continue;
            };
            let d = ox.abs().max(oy.abs());
            let downhill_bonus = (curr_val - next_val).max(0.0) * 4.0;
            let uphill_penalty = (next_val - curr_val).max(0.0) * 6.0;
            let mut score = downhill_bonus - uphill_penalty - (d as f32 * 0.25);
            if let Some(target) = mouth_target {
                let curr_dist = curr.distance_squared(&target) as f32;
                let next_dist = next.distance_squared(&target) as f32;
                score += (curr_dist - next_dist).signum() * 2.0;
            }
            if segment_reenters_visited_path(curr, next, visited) {
                score -= 1000.0;
            }
            let replace = best_choice.as_ref().is_none_or(|(_, best_val, best_d, best_score)| {
                score > *best_score
                    || (score == *best_score && next_val < *best_val)
                    || (score == *best_score && next_val == *best_val && d < *best_d)
            });
            if replace {
                best_choice = Some((next, next_val, d, score));
            }
        }
    }
    best_choice.map(|(next, next_val, d, _)| (next, next_val, d))
}

fn select_spread_sources(
    ordered_sources: Vec<GlobalTilePos>,
    max_sources: usize,
    min_source_distance_tiles: i32,
) -> Vec<GlobalTilePos> {
    if ordered_sources.is_empty() {
        return Vec::new();
    }
    let max_sources = max_sources.max(1);
    let min_dist_sq = (min_source_distance_tiles.max(1) * min_source_distance_tiles.max(1)) as i64;
    let mut selected: Vec<GlobalTilePos> = Vec::with_capacity(max_sources);
    for &candidate in ordered_sources.iter() {
        let too_close = selected.iter().any(|&picked| picked.distance_squared(&candidate) as i64 <= min_dist_sq);
        if too_close {
            continue;
        }
        selected.push(candidate);
        if selected.len() >= max_sources {
            return selected;
        }
    }
    if selected.is_empty() {
        selected.push(ordered_sources[0]);
    }
    selected
}
