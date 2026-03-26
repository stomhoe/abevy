use crate::{
    being_inst_template::being_inst_template_resources::BitRef,
    being_interaction_zone_helper::resolve_being_interaction_zone,
    being_messages::MakeChunkSnapshotForChaser,
    race::race_resources::RaceRef,
};
use ::being_shared::*;
use faction_shared::BelongsToAPlayerFaction;
use ::tilemap_shared::*;
use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_northstar::prelude::*;
use common::log_targets::BEING_SYSTEM;
use movement::movement_components::*;
use param_sets::BlockingTileParamSet;
use tilemap::chunking::chunking_components::ActivatingChunks;
use std::time::Duration;

use super::being_nav_components::RetainedChasePathSnapshot;
use super::being_nav_resources::{AiNavGrids, ChaserNavPlans, SharedChaseFlowFields};
use super::being_nav_structs::{AiNavGridCache, ChaserNavPlan, SharedChaseFlowField};
use super::being_nav_helpers::{
    cardinal_step_toward,
    rebuild_connected_chase_chunk_path,
    rebuild_dynamic_blocking,
    rebuild_retained_chase_chunk_positions,
    extend_target_chunk_trail,
    retained_target_trail_stale_timeout,
    retained_target_chunk_trails_match_positions,
};

const BOXED_TARGET_RETRY_SECS: f32 = 0.4;
const PARTIAL_TARGET_RETRY_SECS: f32 = 1.0;
const TARGET_SHIFT_REBUILD_TILES: i32 = 1;
const BLOCKED_STEP_RETRY_SECS: f32 = 0.08;
const CHASE_PACK_COMPACT_DISTANCE_TILES: i32 = 3;
const BOXED_TARGET_FALLBACK_DELTAS: [IVec2; 12] = [
    IVec2::new(2, 0),
    IVec2::new(-2, 0),
    IVec2::new(0, 2),
    IVec2::new(0, -2),
    IVec2::new(1, 1),
    IVec2::new(1, -1),
    IVec2::new(-1, 1),
    IVec2::new(-1, -1),
    IVec2::new(3, 0),
    IVec2::new(-3, 0),
    IVec2::new(0, 3),
    IVec2::new(0, -3),
];

fn consider_chase_goal_candidate(
    best_path_tiles: &mut Vec<GlobalTilePos>,
    best_path_is_partial: &mut bool,
    best_path_remaining: &mut f32,
    best_path_cost: &mut u32,
    dynamic_blocking: &mut HashMap<UVec3, Entity>,
    cache: &AiNavGridCache,
    chaser_ent: Entity,
    target_ent: Entity,
    chaser_gpos: GlobalTilePos,
    start: UVec3,
    goal: UVec3,
    goal_occupant: Option<Entity>,
) {
    rebuild_dynamic_blocking(dynamic_blocking, cache, chaser_ent, target_ent, start, goal);
    if let Some(goal_occupant) = goal_occupant.filter(|&ent| ent != chaser_ent && ent != target_ent) {
        dynamic_blocking.insert(goal, goal_occupant);
    }
    let mut req = PathfindArgs::new(start, goal)
        .astar()
        .partial()
        .blocking(dynamic_blocking);
    let Some(path) = cache.grid.pathfind(&mut req) else {
        return;
    };
    let path_tiles: Vec<GlobalTilePos> = path.path()
        .iter()
        .map(|step| GlobalTilePos(step.xy().as_ivec2() + cache.min))
        .collect();
    let end_pos = path_tiles.last().copied().unwrap_or(chaser_gpos);
    let remaining = end_pos.taxicab_tile_distance(GlobalTilePos(goal.xy().as_ivec2() + cache.min));
    let is_better = best_path_tiles.is_empty()
        || (*best_path_is_partial && !path.is_partial())
        || (*best_path_is_partial == path.is_partial() && remaining < *best_path_remaining)
        || (*best_path_is_partial == path.is_partial()
            && (remaining - *best_path_remaining).abs() <= f32::EPSILON
            && path.cost() < *best_path_cost);
    if is_better {
        *best_path_tiles = path_tiles;
        *best_path_is_partial = path.is_partial();
        *best_path_remaining = remaining;
        *best_path_cost = path.cost();
    }
}

fn queue_released_chase_chunks_for_despawn(
    activating_chunks: Option<&ActivatingChunks>,
    loaded_chunks: &LoadedChunks,
    messages: &mut Vec<CheckIfChunkShouldDespawn>,
    dimension: DimensionRef,
) {
    let Some(activating_chunks) = activating_chunks else {
        return;
    };
    for &chunk_pos in activating_chunks.0.iter() {
        let Some(&chunk_ent) = loaded_chunks.0.get(&(dimension, chunk_pos)) else {
            continue;
        };
        messages.push(CheckIfChunkShouldDespawn(chunk_ent));
    }
}

fn request_fast_repath(
    plans: &mut ChaserNavPlans,
    chaser_ent: Entity,
    target_pos: GlobalTilePos,
) {
    let Some(plan) = plans.by_ent.get_mut(&chaser_ent) else {
        return;
    };
    let fast_retry = Duration::from_secs_f32(BLOCKED_STEP_RETRY_SECS);
    if plan.rebuild_timer.duration() > fast_retry {
        plan.clear_path_and_retry(fast_retry, target_pos);
    }
}

fn collect_chase_goal_tiles(
    cache: &AiNavGridCache,
    target_pos: GlobalTilePos,
    collision_zone: &InteractionZone,
    zone_tiles: &mut Vec<GlobalTilePos>,
    goal_tiles: &mut Vec<GlobalTilePos>,
    seed_goal_tiles: &mut Vec<(GlobalTilePos, u32)>,
    zone_hits: &mut Vec<GlobalTilePos>,
) {
    zone_tiles.clear();
    goal_tiles.clear();
    seed_goal_tiles.clear();
    zone_hits.clear();

    zone_tiles.reserve(collision_zone.perimeter_size().max(4) as usize);
    collision_zone.gather_zone_positions(CardinalDirection::South, target_pos.to_pixelpos(), zone_tiles);
    if zone_tiles.is_empty() {
        zone_tiles.push(target_pos);
    }
    zone_tiles.sort_unstable_by_key(|pos| (pos.0.x, pos.0.y));
    zone_tiles.dedup();

    goal_tiles.reserve(zone_tiles.len().saturating_mul(8));
    for zone_tile in zone_tiles.iter().copied() {
        for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
            let candidate = GlobalTilePos(zone_tile.0 + delta);
            if zone_tiles
                .binary_search_by_key(&(candidate.0.x, candidate.0.y), |pos| (pos.0.x, pos.0.y))
                .is_ok()
            {
                continue;
            }

            zone_hits.clear();
            collision_zone.gather_accessible_border_positions_for_checked_pos(
                target_pos,
                candidate,
                zone_hits,
            );
            if zone_hits.is_empty() {
                continue;
            }

            let Some(local) = cache.local_from_gpos(candidate) else {
                continue;
            };
            if !cache.grid.is_passable(local) {
                continue;
            }
            goal_tiles.push(candidate);
        }
    }

    let mut min_zone = zone_tiles.first().copied().unwrap_or(target_pos).0;
    let mut max_zone = min_zone;
    for zone_tile in zone_tiles.iter().copied() {
        min_zone = min_zone.min(zone_tile.0);
        max_zone = max_zone.max(zone_tile.0);
    }
    for y in (min_zone.y - CHASE_PACK_COMPACT_DISTANCE_TILES)..=(max_zone.y + CHASE_PACK_COMPACT_DISTANCE_TILES) {
        for x in (min_zone.x - CHASE_PACK_COMPACT_DISTANCE_TILES)..=(max_zone.x + CHASE_PACK_COMPACT_DISTANCE_TILES) {
            let candidate = GlobalTilePos::new(x, y);
            if zone_tiles
                .binary_search_by_key(&(candidate.0.x, candidate.0.y), |pos| (pos.0.x, pos.0.y))
                .is_ok()
            {
                continue;
            }
            let Some(min_zone_dist) = shared_chase_min_zone_distance(zone_tiles, candidate) else {
                continue;
            };
            if min_zone_dist == 0 || min_zone_dist > CHASE_PACK_COMPACT_DISTANCE_TILES as u32 {
                continue;
            }
            let Some(local) = cache.local_from_gpos(candidate) else {
                continue;
            };
            if !cache.grid.is_passable(local) {
                continue;
            }
            goal_tiles.push(candidate);
        }
    }

    if goal_tiles.is_empty() {
        for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
            let candidate = GlobalTilePos(target_pos.0 + delta);
            let Some(local) = cache.local_from_gpos(candidate) else {
                continue;
            };
            if !cache.grid.is_passable(local) {
                continue;
            }
            goal_tiles.push(candidate);
        }
    }

    goal_tiles.sort_unstable_by_key(|pos| (pos.0.x, pos.0.y));
    goal_tiles.dedup();

    seed_goal_tiles.reserve(goal_tiles.len());
    for goal_tile in goal_tiles.iter().copied() {
        let Some(seed_cost) = shared_chase_min_zone_distance(zone_tiles, goal_tile)
            .map(|dist| dist.saturating_sub(1)) else {
            continue;
        };
        let Some(local) = cache.local_from_gpos(goal_tile) else {
            continue;
        };
        if cache.occupied.contains_key(&local) {
            continue;
        }
        seed_goal_tiles.push((goal_tile, seed_cost, ));
    }
    if seed_goal_tiles.is_empty() {
        seed_goal_tiles.extend(goal_tiles.iter().copied().filter_map(|goal_tile| {
            shared_chase_min_zone_distance(zone_tiles, goal_tile)
                .map(|dist| (goal_tile, dist.saturating_sub(1), ))
        }));
    }
}

fn rebuild_shared_chase_flow_field(
    chase_fields: &mut SharedChaseFlowFields,
    cache: &AiNavGridCache,
    target_ent: Entity,
    target_dim: DimensionRef,
    target_pos: GlobalTilePos,
    target_bit_ref: Option<&BitRef>,
    target_race_ref: Option<&RaceRef>,
    interaction_zones_query: &Query<&InteractionZones, >,
    zone_tiles: &mut Vec<GlobalTilePos>,
    goal_tiles: &mut Vec<GlobalTilePos>,
    seed_goal_tiles: &mut Vec<(GlobalTilePos, u32)>,
    zone_hits: &mut Vec<GlobalTilePos>,
) -> Option<()> {
    let collision_zone = resolve_being_interaction_zone(
        interaction_zones_query.get(target_ent).ok(),
        target_bit_ref,
        target_race_ref,
        InteractionZones::COLLISION,
        interaction_zones_query,
    );
    collect_chase_goal_tiles(
        cache,
        target_pos,
        &collision_zone,
        zone_tiles,
        goal_tiles,
        seed_goal_tiles,
        zone_hits,
    );
    let flow_field = SharedChaseFlowField::build(
        cache,
        target_dim.0,
        target_pos,
        goal_tiles,
        seed_goal_tiles,
    )?;
    debug!(
        target: BEING_SYSTEM,
        "Built shared chase flow target={:?} dim={:?} anchor={:?} goals={} open_goals={} zone_tiles={}",
        target_ent,
        target_dim,
        target_pos,
        flow_field.goal_tiles.len(),
        flow_field.seed_goal_tiles.len(),
        zone_tiles.len(),
    );
    chase_fields.by_target.insert(target_ent, flow_field);
    Some(())
}

fn choose_shared_chase_step(
    blocking_tiles: &mut BlockingTileParamSet,
    cache: &AiNavGridCache,
    flow_field: &SharedChaseFlowField,
    chaser_ent: Entity,
    chaser_dim: DimensionRef,
    chaser_pos: GlobalTilePos,
    last_dir: Option<IVec2>,
) -> Option<Vec2> {
    const SHARED_CHASE_HOLD_DISTANCE: u32 = 1;
    const SHARED_CHASE_HOLD_MARGIN: i32 = 120;

    let curr_dist = flow_field.distance_at_gpos(cache, chaser_pos)?;
    if curr_dist == 0 {
        return Some(Vec2::ZERO);
    }

    let target_delta = chaser_pos.0 - flow_field.target_pos.0;
    let forward_axis = cardinal_step_toward(-target_delta);
    let lateral_axis = IVec2::new(-forward_axis.y, forward_axis.x);
    let desired_lane_offset = if lateral_axis == IVec2::ZERO {
        0
    } else {
        let max_lane_offset = ((curr_dist as i32) / 5).clamp(0, 3);
        if max_lane_offset == 0 {
            0
        } else {
            let lane_count = (max_lane_offset * 2) + 1;
            let ent_index = chaser_ent.index_u32() as i32;
            let mut lane = (ent_index % lane_count) - max_lane_offset;
            if lane == 0 {
                lane = if ent_index % 2 == 0 { 1 } else { -1 };
            }
            lane.clamp(-max_lane_offset, max_lane_offset)
        }
    };
    let current_lane_penalty = if lateral_axis == IVec2::ZERO {
        0
    } else {
        let lane_offset = (chaser_pos.0 - flow_field.target_pos.0).dot(lateral_axis);
        (lane_offset - desired_lane_offset).abs() * 35
    };
    let current_crowd_penalty = shared_chase_neighbor_crowd_penalty(cache, chaser_ent, chaser_pos);
    let hold_score = (curr_dist as i32 * 1000)
        + current_lane_penalty
        + current_crowd_penalty
        - 220;

    let mut best_step = None;
    let mut best_non_improving_step = None;
    let mut best_score = i32::MAX;
    let mut best_non_improving_score = i32::MAX;
    for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
        let next_pos = GlobalTilePos(chaser_pos.0 + delta);
        let Some(next_dist) = flow_field.distance_at_gpos(cache, next_pos) else {
            continue;
        };
        if blocking_tiles.is_blocked_at_tiles_only(chaser_dim, next_pos, chaser_ent) {
            continue;
        }
        if blocking_tiles.is_blocked_at(chaser_dim, next_pos, chaser_ent) {
            continue;
        }
        if next_dist > curr_dist.saturating_add(1) {
            continue;
        }

        let progress_penalty = if next_dist > curr_dist {
            900
        } else if next_dist == curr_dist {
            180
        } else {
            0
        };
        let lane_penalty = if lateral_axis == IVec2::ZERO {
            0
        } else {
            let lane_offset = (next_pos.0 - flow_field.target_pos.0).dot(lateral_axis);
            (lane_offset - desired_lane_offset).abs() * 35
        };
        let crowd_penalty = shared_chase_neighbor_crowd_penalty(cache, chaser_ent, next_pos);
        let goal_bonus = if flow_field.is_goal_tile(next_pos) { -120 } else { 0 };
        let direction_change_penalty = if let Some(last_dir) = last_dir {
            if delta == -last_dir {
                if next_dist < curr_dist { 120 } else { 700 }
            } else if delta != last_dir && next_dist >= curr_dist {
                180
            } else {
                0
            }
        } else {
            0
        };
        let score = (next_dist as i32 * 1000)
            + progress_penalty
            + lane_penalty
            + crowd_penalty
            + direction_change_penalty
            + goal_bonus;
        if next_dist < curr_dist {
            if score < best_score {
                best_score = score;
                best_step = Some(delta.as_vec2());
            }
            continue;
        }
        if score < best_non_improving_score {
            best_non_improving_score = score;
            best_non_improving_step = Some(delta.as_vec2());
        }
    }

    if best_step.is_some() {
        return best_step;
    }
    if curr_dist <= SHARED_CHASE_HOLD_DISTANCE {
        return Some(Vec2::ZERO);
    }
    if let Some(best_non_improving_step) = best_non_improving_step {
        if let Some(last_dir) = last_dir {
            if best_non_improving_step.as_ivec2() == -last_dir {
                return Some(Vec2::ZERO);
            }
        }
        if best_non_improving_score + SHARED_CHASE_HOLD_MARGIN < hold_score {
            return Some(best_non_improving_step);
        }
    }
    Some(Vec2::ZERO)
}

fn shared_chase_min_zone_distance(
    zone_tiles: &[GlobalTilePos],
    pos: GlobalTilePos,
) -> Option<u32> {
    let mut min_dist: Option<u32> = None;
    for zone_tile in zone_tiles.iter().copied() {
        let dist = zone_tile.taxicab_tile_distance(pos) as u32;
        min_dist = Some(min_dist.map(|curr| curr.min(dist)).unwrap_or(dist));
    }
    min_dist
}

fn shared_chase_neighbor_crowd_penalty(
    cache: &AiNavGridCache,
    chaser_ent: Entity,
    pos: GlobalTilePos,
) -> i32 {
    let mut crowd_penalty = 0;
    for neighbor_delta in [
        IVec2::X,
        -IVec2::X,
        IVec2::Y,
        -IVec2::Y,
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, 1),
        IVec2::new(-1, -1),
    ] {
        let Some(local) = cache.local_from_gpos(GlobalTilePos(pos.0 + neighbor_delta)) else {
            continue;
        };
        let Some(&occupant_ent) = cache.occupied.get(&local) else {
            continue;
        };
        if occupant_ent == chaser_ent {
            continue;
        }
        crowd_penalty += if neighbor_delta.x == 0 || neighbor_delta.y == 0 { 70 } else { 40 };
    }
    crowd_penalty
}

#[allow(unused_parens, )]
pub fn cleanup_player_chase_chunk_retention(
    mut commands: Commands,
    loaded_chunks: Res<LoadedChunks>,
    chasers: Query<
        (
            Entity,
            &GlobalTilePos,
            &DimensionRef,
            Option<&Chasing>,
            Option<&ActivatingChunks>,
            Has<RetainedChasePathSnapshot>,
        ),
        (With<Being>, LocalAiControlled, Without<LoadChunksAround>),
    >,
    targets_query: Query<(&GlobalTilePos, &DimensionRef, Has<BelongsToAPlayerFaction>), (With<Being>, )>,
    mut writer: MessageWriter<CheckIfChunkShouldDespawn>,
    mut messages: Local<Vec<CheckIfChunkShouldDespawn>>,
) {
    for (chaser_ent, chaser_gpos, &chaser_dim, chaser, activating_chunks, has_snapshot) in chasers.iter() {
        if activating_chunks.is_none() && !has_snapshot {
            continue;
        }

        let Some(chaser) = chaser else {
            queue_released_chase_chunks_for_despawn(activating_chunks, &loaded_chunks, &mut messages, chaser_dim);
            commands.entity(chaser_ent).try_remove::<(ActivatingChunks, RetainedChasePathSnapshot)>();
            debug!(target: BEING_SYSTEM, "Removing chase chunk retention from {:?}: no longer chasing at {:?}", chaser_ent, chaser_gpos);
            continue;
        };
        let Ok((_target_gpos, &target_dim, target_is_player_faction)) = targets_query.get(chaser.target) else {
            queue_released_chase_chunks_for_despawn(activating_chunks, &loaded_chunks, &mut messages, chaser_dim);
            commands.entity(chaser_ent).try_remove::<(ActivatingChunks, RetainedChasePathSnapshot)>();
            debug!(target: BEING_SYSTEM, "Removing chase chunk retention from {:?}: target {:?} missing", chaser_ent, chaser.target);
            continue;
        };
        if !target_is_player_faction || target_dim != chaser_dim {
            queue_released_chase_chunks_for_despawn(activating_chunks, &loaded_chunks, &mut messages, chaser_dim);
            commands.entity(chaser_ent).try_remove::<(ActivatingChunks, RetainedChasePathSnapshot)>();
            debug!(target: BEING_SYSTEM, "Removing chase chunk retention from {:?}: target {:?} not an in-dimension player prey", chaser_ent, chaser.target);
        }
    }
    writer.write_batch(messages.drain(..));
}

#[allow(unused_parens, )]
pub fn rebuild_goto_nav_plans(
    time: Res<Time>,
    goto_beings: Query<
        (
            Entity,
            &GlobalTilePos,
            &DimensionRef,
            &GoTo,
            Option<&SpeedMagnitude>,
            Option<&GoToMeta>,
            Option<&Chasing>,
        ),
        (With<Being>, LocalAiControlled),
    >,
    grids: Res<AiNavGrids>,
    targets_query: Query<
        (
            &GlobalTilePos,
            &DimensionRef,
            Option<&BitRef>,
            Option<&RaceRef>,
        ),
        (),
    >,
    interaction_zones_query: Query<&InteractionZones, >,
    mut chase_fields: ResMut<SharedChaseFlowFields>,
    mut plans: ResMut<ChaserNavPlans>,
    mut dynamic_blocking: Local<HashMap<UVec3, Entity>>,
    mut active_chasers: Local<EntityHashSet>,
    mut active_chase_targets: Local<EntityHashSet>,
    mut chase_zone_tiles: Local<Vec<GlobalTilePos>>,
    mut chase_goal_tiles: Local<Vec<GlobalTilePos>>,
    mut chase_seed_goal_tiles: Local<Vec<(GlobalTilePos, u32)>>,
    mut chase_zone_hits: Local<Vec<GlobalTilePos>>,
) {
    active_chasers.clear();
    active_chase_targets.clear();

    for (chaser_ent, chaser_gpos, &chaser_dim, goto, chaser_speed, go_to_meta, chasing, ) in goto_beings.iter() {
        active_chasers.insert(chaser_ent);

        let target_pos = goto.pos;
        let goto_interval = ChaserNavPlan::rebuild_interval(
            chaser_speed.map_or(1.0, |speed| speed.0),
            0.0,
            chaser_gpos.taxicab_tile_distance(target_pos),
        );

        let plan = plans.by_ent.entry(chaser_ent).or_default();
        let timer_finished = plan.rebuild_timer.tick(time.delta()).just_finished();
        if chaser_gpos.taxicab_tile_distance(target_pos) <= goto.stop_distance.max(0.0) {
            plan.clear_path_and_retry(Duration::from_secs_f32(0.25), target_pos);
            continue;
        }

        let Some(cache) = grids.by_dim.get(&chaser_dim.0) else {
            plan.clear_path_and_retry(goto_interval, target_pos);
            continue;
        };

        let is_shared_chase = matches!(go_to_meta, Some(meta) if meta.source == NavOrderSource::Chasing);
        if is_shared_chase {
            let Some(chasing) = chasing else {
                plan.clear_path_and_retry(goto_interval, target_pos);
                continue;
            };
            let Ok((target_gpos, &target_dim, target_bit_ref, target_race_ref, )) = targets_query.get(chasing.target) else {
                plan.clear_path_and_retry(goto_interval, target_pos);
                continue;
            };
            if target_dim != chaser_dim {
                plan.clear_path_and_retry(goto_interval, *target_gpos);
                continue;
            }

            active_chase_targets.insert(chasing.target);
            let target_pos = *target_gpos;
            let target_shifted = plan
                .last_target_pos
                .map(|prev| (prev.0 - target_pos.0).abs().max_element() >= TARGET_SHIFT_REBUILD_TILES)
                .unwrap_or(true);
            let need_rebuild = plan.path_tiles.is_empty() || timer_finished || target_shifted;

            let collision_zone = resolve_being_interaction_zone(
                interaction_zones_query.get(chasing.target).ok(),
                target_bit_ref,
                target_race_ref,
                InteractionZones::COLLISION,
                &interaction_zones_query,
            );
            collect_chase_goal_tiles(
                cache,
                target_pos,
                &collision_zone,
                &mut chase_zone_tiles,
                &mut chase_goal_tiles,
                &mut chase_seed_goal_tiles,
                &mut chase_zone_hits,
            );
            let needs_flow_rebuild = chase_fields
                .by_target
                .get(&chasing.target)
                .map(|field| {
                    !field.matches_grid(cache, target_dim.0, target_pos)
                        || field.goal_tiles != *chase_goal_tiles
                        || field.seed_goal_tiles != *chase_seed_goal_tiles
                })
                .unwrap_or(true);
            if needs_flow_rebuild {
                let Some(()) = rebuild_shared_chase_flow_field(
                    &mut chase_fields,
                    cache,
                    chasing.target,
                    target_dim,
                    target_pos,
                    target_bit_ref,
                    target_race_ref,
                    &interaction_zones_query,
                    &mut chase_zone_tiles,
                    &mut chase_goal_tiles,
                    &mut chase_seed_goal_tiles,
                    &mut chase_zone_hits,
                ) else {
                    plan.clear_path_and_retry(
                        goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)),
                        target_pos,
                    );
                    trace!(target: BEING_SYSTEM, "Deferred chase flow rebuild for {:?}: prey {:?} has no valid goal tiles at {:?}", chaser_ent, chasing.target, target_pos);
                    continue;
                };
            }
            let Some(flow_field) = chase_fields.by_target.get(&chasing.target) else {
                plan.clear_path_and_retry(
                    goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)),
                    target_pos,
                );
                continue;
            };
            if !need_rebuild {
                continue;
            }
            if flow_field.is_goal_tile(*chaser_gpos) {
                plan.clear_path_and_retry(Duration::from_secs_f32(0.2), target_pos);
                trace!(target: BEING_SYSTEM, "Holding boxed chase slot for {:?}: prey {:?} anchor {:?}", chaser_ent, chasing.target, target_pos);
                continue;
            }
            if !flow_field.reconstruct_path_tiles(cache, *chaser_gpos, &mut plan.path_tiles) {
                plan.clear_path_and_retry(
                    goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)),
                    target_pos,
                );
                trace!(target: BEING_SYSTEM, "Deferred chase plan rebuild for {:?}: prey {:?} unreachable from {:?} on shared field", chaser_ent, chasing.target, chaser_gpos);
                continue;
            }

            plan.next_step_ix = 0;
            plan.last_target_pos = Some(target_pos);
            plan.holds_at_partial_endpoint = false;
            plan.rebuild_timer = Timer::new(goto_interval, TimerMode::Once);
            trace!(target: BEING_SYSTEM, "Rebuilt shared chase plan for {:?}: prey {:?}, anchor {:?}, steps {}", chaser_ent, chasing.target, target_pos, plan.path_tiles.len());
            continue;
        }

        let Some((start, goal)) = cache.local_path_points(*chaser_gpos, target_pos) else {
            plan.clear_path_and_retry(goto_interval, target_pos);
            continue;
        };

        let mut open_target_approach_count = 0usize;
        let mut blocked_target_approach_count = 0usize;
        let mut best_path_tiles: Vec<GlobalTilePos> = Vec::new();
        let mut best_path_is_partial = true;
        let mut best_path_remaining = f32::INFINITY;
        let mut best_path_cost = u32::MAX;
        for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
            let local = target_pos.0 + delta - cache.min;
            if local.x < 0
                || local.y < 0
                || local.x >= cache.grid.width() as i32
                || local.y >= cache.grid.height() as i32
            {
                continue;
            }
            let local = UVec3::new(local.x as u32, local.y as u32, 0);
            if !cache.grid.is_passable(local) {
                continue;
            }
            open_target_approach_count += 1;

            let Some(&occupant_ent) = cache.occupied.get(&local) else {
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut dynamic_blocking,
                    cache,
                    chaser_ent,
                    Entity::PLACEHOLDER,
                    *chaser_gpos,
                    start,
                    local,
                    None,
                );
                continue;
            };
            if occupant_ent == chaser_ent {
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut dynamic_blocking,
                    cache,
                    chaser_ent,
                    Entity::PLACEHOLDER,
                    *chaser_gpos,
                    start,
                    local,
                    None,
                );
                continue;
            }
            let Ok((_blocker_ent, _blocker_gpos, _blocker_dim, blocker_goto, _blocker_speed, _blocker_meta, _blocker_chasing, )) = goto_beings.get(occupant_ent) else {
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut dynamic_blocking,
                    cache,
                    chaser_ent,
                    Entity::PLACEHOLDER,
                    *chaser_gpos,
                    start,
                    local,
                    Some(occupant_ent),
                );
                continue;
            };
            if blocker_goto.pos == target_pos {
                blocked_target_approach_count += 1;
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut dynamic_blocking,
                    cache,
                    chaser_ent,
                    Entity::PLACEHOLDER,
                    *chaser_gpos,
                    start,
                    local,
                    Some(occupant_ent),
                );
                continue;
            }

            consider_chase_goal_candidate(
                &mut best_path_tiles,
                &mut best_path_is_partial,
                &mut best_path_remaining,
                &mut best_path_cost,
                &mut dynamic_blocking,
                cache,
                chaser_ent,
                Entity::PLACEHOLDER,
                *chaser_gpos,
                start,
                local,
                Some(occupant_ent),
            );
        }
        let target_boxed_by_same_target_chasers = open_target_approach_count > 0
            && blocked_target_approach_count >= open_target_approach_count;

        let target_shifted = plan
            .last_target_pos
            .map(|prev| (prev.0 - target_pos.0).abs().max_element() >= TARGET_SHIFT_REBUILD_TILES)
            .unwrap_or(true);
        let need_rebuild = timer_finished || target_shifted;
        if !need_rebuild {
            continue;
        }

        if target_boxed_by_same_target_chasers {
            for delta in BOXED_TARGET_FALLBACK_DELTAS {
                let local = target_pos.0 + delta - cache.min;
                if local.x < 0
                    || local.y < 0
                    || local.x >= cache.grid.width() as i32
                    || local.y >= cache.grid.height() as i32
                {
                    continue;
                }
                let local = UVec3::new(local.x as u32, local.y as u32, 0);
                if !cache.grid.is_passable(local) {
                    continue;
                }
                let Some(&occupant_ent) = cache.occupied.get(&local) else {
                    consider_chase_goal_candidate(
                        &mut best_path_tiles,
                        &mut best_path_is_partial,
                        &mut best_path_remaining,
                        &mut best_path_cost,
                        &mut dynamic_blocking,
                        cache,
                        chaser_ent,
                        Entity::PLACEHOLDER,
                        *chaser_gpos,
                        start,
                        local,
                        None,
                    );
                    continue;
                };
                if occupant_ent == chaser_ent {
                    consider_chase_goal_candidate(
                        &mut best_path_tiles,
                        &mut best_path_is_partial,
                        &mut best_path_remaining,
                        &mut best_path_cost,
                        &mut dynamic_blocking,
                        cache,
                        chaser_ent,
                        Entity::PLACEHOLDER,
                        *chaser_gpos,
                        start,
                        local,
                        None,
                    );
                    continue;
                }
                let Ok((_blocker_ent, _blocker_gpos, _blocker_dim, blocker_goto, _blocker_speed, _blocker_meta, _blocker_chasing, )) = goto_beings.get(occupant_ent) else {
                    consider_chase_goal_candidate(
                        &mut best_path_tiles,
                        &mut best_path_is_partial,
                        &mut best_path_remaining,
                        &mut best_path_cost,
                        &mut dynamic_blocking,
                        cache,
                        chaser_ent,
                        Entity::PLACEHOLDER,
                        *chaser_gpos,
                        start,
                        local,
                        Some(occupant_ent),
                    );
                    continue;
                };
                if blocker_goto.pos == target_pos {
                    consider_chase_goal_candidate(
                        &mut best_path_tiles,
                        &mut best_path_is_partial,
                        &mut best_path_remaining,
                        &mut best_path_cost,
                        &mut dynamic_blocking,
                        cache,
                        chaser_ent,
                        Entity::PLACEHOLDER,
                        *chaser_gpos,
                        start,
                        local,
                        Some(occupant_ent),
                    );
                    continue;
                }
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut dynamic_blocking,
                    cache,
                    chaser_ent,
                    Entity::PLACEHOLDER,
                    *chaser_gpos,
                    start,
                    local,
                    Some(occupant_ent),
                );
            }
            if best_path_tiles.is_empty() {
                plan.clear_path_and_retry(
                    goto_interval.max(Duration::from_secs_f32(BOXED_TARGET_RETRY_SECS)),
                    target_pos,
                );
                trace!(target: BEING_SYSTEM, "Delaying GoTo nav for {:?}: target {:?} boxed by same-target movers", chaser_ent, target_pos);
                continue;
            }
            trace!(target: BEING_SYSTEM, "Using boxed-target fallback approach for {:?}: target {:?}, steps {}", chaser_ent, target_pos, best_path_tiles.len());
        }

        if best_path_tiles.is_empty() {
            rebuild_dynamic_blocking(&mut dynamic_blocking, cache, chaser_ent, Entity::PLACEHOLDER, start, goal);
            let mut req = PathfindArgs::new(start, goal)
                .astar()
                .partial()
                .blocking(&dynamic_blocking);
            let Some(path) = cache.grid.pathfind(&mut req) else {
                plan.clear_path_and_retry(goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)), target_pos);
                trace!(target: BEING_SYSTEM, "Deferred GoTo nav retry for {:?}: target {:?}, dist {:.2}, interval {:.2}s", chaser_ent, target_pos, chaser_gpos.taxicab_tile_distance(target_pos), goto_interval.as_secs_f32());
                continue;
            };
            best_path_is_partial = path.is_partial();
            best_path_tiles.extend(
                path.path()
                    .iter()
                    .map(|step| GlobalTilePos(step.xy().as_ivec2() + cache.min)),
            );
        }

        plan.path_tiles.clear();
        plan.path_tiles.extend(best_path_tiles);
        plan.next_step_ix = 0;
        plan.last_target_pos = Some(target_pos);
        plan.holds_at_partial_endpoint = best_path_is_partial;
        plan.rebuild_timer = Timer::new(
            if best_path_is_partial {
                goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS))
            } else {
                goto_interval
            },
            TimerMode::Once,
        );
        trace!(target: BEING_SYSTEM, "Rebuilt GoTo nav for {:?}: target {:?}, dist {:.2}, interval {:.2}s, steps {}", chaser_ent, target_pos, chaser_gpos.taxicab_tile_distance(target_pos), goto_interval.as_secs_f32(), plan.path_tiles.len());
    }

    plans.by_ent.retain(|ent, _| active_chasers.contains(ent));
    chase_fields
        .by_target
        .retain(|target_ent, field| active_chase_targets.contains(target_ent) && grids.by_dim.contains_key(&field.dim));
}

#[allow(unused_parens, )]
pub fn goto_behavior(
    mut blocking_tiles: BlockingTileParamSet,
    mut goto_beings: Query<
        (
            Entity,
            &DimensionRef,
            &GoTo,
            Option<&GoToMeta>,
            Option<&Chasing>,
        ),
        (With<Being>, LocalAiControlled),
    >,
    grids: Res<AiNavGrids>,
    chase_fields: Res<SharedChaseFlowFields>,
    mut plans: ResMut<ChaserNavPlans>,
    mut input_dirs: Query<&mut InputMoveDir>,
        mut last_shared_dirs: Local<HashMap<Entity, IVec2>>,
) {
    for (chaser_ent, &chaser_dim, goto, go_to_meta, chasing, ) in goto_beings.iter_mut() {
        let Ok(mut input_move_dir) = input_dirs.get_mut(chaser_ent) else {
            continue;
        };
        let Ok(&chaser_pos) = blocking_tiles.gpos_query.get(chaser_ent) else {
            continue;
        };
        let target_pos = goto.pos;

        let shared_flow_field = if matches!(go_to_meta, Some(meta) if meta.source == NavOrderSource::Chasing) {
            chasing.and_then(|chasing| {
                let cache = grids.by_dim.get(&chaser_dim.0)?;
                chase_fields
                    .by_target
                    .get(&chasing.target)
                    .filter(|flow_field| flow_field.matches_grid(cache, chaser_dim.0, target_pos))
                    .map(|flow_field| (cache, flow_field))
            })
        } else {
            None
        };
        if let Some((cache, flow_field, )) = shared_flow_field {
            if flow_field.is_goal_tile(chaser_pos) || flow_field.distance_at_gpos(cache, chaser_pos) == Some(0) {
                input_move_dir.0 = Vec2::ZERO;
                last_shared_dirs.remove(&chaser_ent);
                let face_delta = target_pos.0 - chaser_pos.0;
                if face_delta != IVec2::ZERO {
                    let facing = CardinalDirection::from_dir_vec(cardinal_step_toward(face_delta));
                    blocking_tiles.set_being_direction(chaser_ent, facing);
                }
                continue;
            }
        }

        let Some(direct_chase_dir) = chaser_pos.direct_chase_dir(target_pos, goto.stop_distance) else {
            input_move_dir.0 = Vec2::ZERO;
            let face_delta = target_pos.0 - chaser_pos.0;
            if face_delta != IVec2::ZERO {
                let facing = CardinalDirection::from_dir_vec(cardinal_step_toward(face_delta));
                blocking_tiles.set_being_direction(chaser_ent, facing);
            }
            continue;
        };

        let move_input = if let Some((cache, flow_field, )) = shared_flow_field {
            choose_shared_chase_step(
                &mut blocking_tiles,
                cache,
                flow_field,
                chaser_ent,
                chaser_dim,
                chaser_pos,
                last_shared_dirs.get(&chaser_ent).copied(),
            )
            .unwrap_or_else(|| {
                plans
                    .by_ent
                    .get_mut(&chaser_ent)
                    .map(|plan| {
                        match plan.next_step(chaser_pos) {
                            Some(next) => {
                                let desired = cardinal_step_toward(next.0 - chaser_pos.0);
                                if desired == IVec2::ZERO {
                                    direct_chase_dir
                                } else {
                                    desired.as_vec2()
                                }
                            }
                            None if plan.holds_at_partial_endpoint => Vec2::ZERO,
                            None => direct_chase_dir,
                        }
                    })
                    .unwrap_or(direct_chase_dir)
            })
        } else if let Some(plan) = plans.by_ent.get_mut(&chaser_ent) {
            match plan.next_step(chaser_pos) {
                Some(next) => {
                    let desired = cardinal_step_toward(next.0 - chaser_pos.0);
                    if desired == IVec2::ZERO {
                        direct_chase_dir
                    } else {
                        let desired_next_pos = GlobalTilePos(chaser_pos.0 + desired);
                        let desired_next_dist = desired_next_pos.taxicab_tile_distance(target_pos);
                        let current_dist = chaser_pos.taxicab_tile_distance(target_pos);
                        let direct_axis = FinalNormMoveDir(direct_chase_dir).normalize_to_axis_dir();
                        if direct_axis != IVec2::ZERO {
                            let direct_next_pos = GlobalTilePos(chaser_pos.0 + direct_axis);
                            let direct_next_dist = direct_next_pos.taxicab_tile_distance(target_pos);
                            if direct_next_dist < desired_next_dist {
                                trace!(target: BEING_SYSTEM, "GoTo choosing direct step over plan for {:?}: target {:?}, planned {:?} dist {:.1}, direct {:?} dist {:.1}", chaser_ent, target_pos, desired, desired_next_dist, direct_axis, direct_next_dist);
                                direct_chase_dir
                            } else if desired_next_dist > current_dist {
                                trace!(target: BEING_SYSTEM, "GoTo overriding stale plan step for {:?}: target {:?}, planned {:?} would increase dist {:.1}->{:.1}, using direct {:?}", chaser_ent, target_pos, desired, current_dist, desired_next_dist, direct_axis);
                                direct_chase_dir
                            } else {
                                desired.as_vec2()
                            }
                        } else if desired_next_dist > current_dist {
                            trace!(target: BEING_SYSTEM, "GoTo overriding stale plan step for {:?}: target {:?}, planned {:?} would increase dist {:.1}->{:.1}, using direct {:?}", chaser_ent, target_pos, desired, current_dist, desired_next_dist, direct_axis);
                            direct_chase_dir
                        } else {
                            desired.as_vec2()
                        }
                    }
                }
                None if plan.holds_at_partial_endpoint => Vec2::ZERO,
                None => direct_chase_dir,
            }
        } else {
            direct_chase_dir
        };

        let move_axis = FinalNormMoveDir(move_input).normalize_to_axis_dir();
        if shared_flow_field.is_some() {
            if move_axis != IVec2::ZERO {
                last_shared_dirs.insert(chaser_ent, move_axis);
            }
        } else {
            last_shared_dirs.remove(&chaser_ent);
        }
        if move_axis != IVec2::ZERO {
            let next_pos = GlobalTilePos(chaser_pos.0 + move_axis);
            if blocking_tiles.is_blocked_at_tiles_only(chaser_dim, next_pos, chaser_ent) {
                input_move_dir.0 = Vec2::ZERO;
                request_fast_repath(&mut plans, chaser_ent, target_pos);
                trace!(target: BEING_SYSTEM, "GoTo repath after static blocked step for {:?}: target {:?}, dir {:?}, from {:?}", chaser_ent, target_pos, move_axis, chaser_pos);
                continue;
            }
            if blocking_tiles.is_blocked_at(chaser_dim, next_pos, chaser_ent) {
                input_move_dir.0 = Vec2::ZERO;
                request_fast_repath(&mut plans, chaser_ent, target_pos);
                trace!(target: BEING_SYSTEM, "GoTo repath after dynamic blocked step for {:?}: target {:?}, dir {:?}, from {:?}", chaser_ent, target_pos, move_axis, chaser_pos);
                continue;
            }
        }

        input_move_dir.0 = move_input;
    }
}

pub fn retain_chunks_for_player_faction_chasers(
    mut commands: Commands,
    time: Res<Time>,
    mut chasers: Query<
        (
            Entity,
            &GlobalTilePos,
            &ChunkPos,
            &::tilemap_shared::DimensionRef,
            &Chasing,
            Option<&SpeedMagnitude>,
            Option<&mut ActivatingChunks>,
            Option<&mut RetainedChasePathSnapshot>,
        ),
        (With<Being>, LocalAiControlled, Without<LoadChunksAround>),
    >,
    targets_query: Query<(&ChunkPos, &::tilemap_shared::DimensionRef, Has<BelongsToAPlayerFaction>)>,
    plans: Res<ChaserNavPlans>,
    mut desired_chunks: Local<Vec<ChunkPos>>,
    mut corridor_chunks: Local<Vec<ChunkPos>>,
    mut seen_chunks: Local<HashSet<ChunkPos>>,
) {
    for (chaser_ent, chaser_gpos, &chaser_chunk_pos, &chaser_dim, to_chase, speed, activating_chunks, snapshot) in chasers.iter_mut() {
        let Ok((&target_chunk_pos, &target_dim, target_is_player_faction)) = targets_query.get(to_chase.target) else {
            if activating_chunks.is_some() {
                commands.entity(chaser_ent).try_remove::<ActivatingChunks>();
                debug!(target: BEING_SYSTEM, "Removing chase chunk retention from {:?}: target {:?} missing", chaser_ent, to_chase.target);
            }
            if snapshot.is_some() {
                commands.entity(chaser_ent).try_remove::<RetainedChasePathSnapshot>();
            }
            continue;
        };

        if !target_is_player_faction {
            if activating_chunks.is_some() {
                commands.entity(chaser_ent).try_remove::<ActivatingChunks>();
                debug!(target: BEING_SYSTEM, "Removing chase chunk retention from {:?}: target {:?} not player faction", chaser_ent, to_chase.target);
            }
            if snapshot.is_some() {
                commands.entity(chaser_ent).try_remove::<RetainedChasePathSnapshot>();
            }
            continue;
        }

        if target_dim != chaser_dim {
            if activating_chunks.is_some() {
                commands.entity(chaser_ent).try_remove::<ActivatingChunks>();
            }
            if snapshot.is_some() {
                commands.entity(chaser_ent).try_remove::<RetainedChasePathSnapshot>();
            }
            continue;
        }

        desired_chunks.clear();
        let stale_timeout = retained_target_trail_stale_timeout(speed.map_or(1.0, |speed| speed.0));

        let mut target_chunk_trail = snapshot
            .as_ref()
            .map(|snapshot| snapshot.target_chunk_trail.clone())
            .unwrap_or_default();
        if target_chunk_trail.is_empty() {
            extend_target_chunk_trail(&mut target_chunk_trail, target_chunk_pos, stale_timeout);
        } else if target_chunk_trail.last().map(|entry| entry.chunk_pos) != Some(target_chunk_pos) {
            extend_target_chunk_trail(&mut target_chunk_trail, target_chunk_pos, stale_timeout);
        }

        rebuild_connected_chase_chunk_path(
            &mut corridor_chunks,
            &mut seen_chunks,
            Some(*chaser_gpos),
            plans.by_ent.get(&chaser_ent).map(|plan| plan.path_tiles.as_slice()),
            chaser_chunk_pos,
            target_chunk_pos,
        );

        rebuild_retained_chase_chunk_positions(
            &mut desired_chunks,
            &corridor_chunks,
            &mut seen_chunks,
            &mut target_chunk_trail,
            stale_timeout,
            time.delta(),
        );

        let needs_snapshot = snapshot.as_ref().map(|snapshot| snapshot.chunk_positions.as_slice()) != Some(desired_chunks.as_slice())
            || snapshot
                .as_ref()
                .is_none_or(|snapshot| !retained_target_chunk_trails_match_positions(&snapshot.target_chunk_trail, &target_chunk_trail))
            || snapshot.as_ref().map(|snapshot| snapshot.last_target_chunk_pos) != Some(target_chunk_pos);
        if needs_snapshot {
            commands.entity(chaser_ent).try_insert(RetainedChasePathSnapshot {
                chunk_positions: desired_chunks.clone(),
                target_chunk_trail,
                last_target_chunk_pos: target_chunk_pos,
            });
        }

        trace!(target: BEING_SYSTEM, "Chaser {:?} retaining {} chunks", chaser_ent, desired_chunks.len());
        match activating_chunks {
            Some(mut activating_chunks) => {
                if activating_chunks.0 != *desired_chunks {
                    activating_chunks.0.clear();
                    activating_chunks.0.extend(desired_chunks.iter().copied());
                }
            }
            None => {
                commands.entity(chaser_ent).try_insert(ActivatingChunks(desired_chunks.clone()));
            }
        }
    }
}

#[allow(unused_parens, unused_imports, )]
pub fn make_chunk_snapshot_for_hunter(
    mut commands: Commands,
    mut reader: MessageReader<MakeChunkSnapshotForChaser>,
    plans: Res<ChaserNavPlans>,
    beings: Query<
        (
            &GlobalTilePos,
            &ChunkPos,
            &Chasing,
            Option<&SpeedMagnitude>,
            Option<&ActivatingChunks>,
            Option<&RetainedChasePathSnapshot>,
        ),
    >,
    player_targets: Query<&ChunkPos, (PlayerBeing)>,
    mut desired_chunks: Local<Vec<ChunkPos>>,
    mut corridor_chunks: Local<Vec<ChunkPos>>,
    mut seen_chunks: Local<HashSet<ChunkPos>>,
) {
    for MakeChunkSnapshotForChaser(chaser_ent) in reader.read() {
        let Ok((chaser_gpos, &chaser_chunk_pos, chaser, speed, activating_chunks, snapshot)) = beings.get(*chaser_ent) else {
            continue;
        };
        let Ok(target_chunk_pos) = player_targets.get(chaser.target) else {
            continue;
        };

        let stale_timeout = retained_target_trail_stale_timeout(speed.map_or(1.0, |speed| speed.0));
        let mut target_chunk_trail = snapshot
            .map(|snapshot| snapshot.target_chunk_trail.clone())
            .unwrap_or_default();

        if target_chunk_trail.is_empty() {
            extend_target_chunk_trail(&mut target_chunk_trail, *target_chunk_pos, stale_timeout);
        } else if target_chunk_trail.last().map(|entry| entry.chunk_pos) != Some(*target_chunk_pos) {
            extend_target_chunk_trail(&mut target_chunk_trail, *target_chunk_pos, stale_timeout);
        }

        rebuild_connected_chase_chunk_path(
            &mut corridor_chunks,
            &mut seen_chunks,
            Some(*chaser_gpos),
            plans.by_ent.get(chaser_ent).map(|plan| plan.path_tiles.as_slice()),
            chaser_chunk_pos,
            *target_chunk_pos,
        );

        rebuild_retained_chase_chunk_positions(
            &mut desired_chunks,
            &corridor_chunks,
            &mut seen_chunks,
            &mut target_chunk_trail,
            stale_timeout,
            Duration::ZERO,
        );

        let needs_snapshot = snapshot.map(|snapshot| snapshot.chunk_positions.as_slice()) != Some(desired_chunks.as_slice())
            || snapshot.map(|snapshot| retained_target_chunk_trails_match_positions(&snapshot.target_chunk_trail, &target_chunk_trail)) != Some(true)
            || snapshot.map(|snapshot| snapshot.last_target_chunk_pos) != Some(*target_chunk_pos);
        let needs_activating_chunks = activating_chunks.map(|chunks| chunks.0.as_slice()) != Some(desired_chunks.as_slice());

        let mut entity = commands.entity(*chaser_ent);
        if needs_snapshot || needs_activating_chunks {
            entity.try_insert((
                RetainedChasePathSnapshot {
                    chunk_positions: desired_chunks.clone(),
                    target_chunk_trail,
                    last_target_chunk_pos: *target_chunk_pos,
                },
                ActivatingChunks(desired_chunks.clone()),
            ));
        }
    }
}

#[allow(unused_parens, unused_imports, )]
pub fn dynamically_extend_retained_chasepaths_due_to_moving_player_prey(
    time: Res<Time>,
    mut chasers: Query<
        (
            Entity,
            &Chasing,
            &GlobalTilePos,
            &ChunkPos,
            &DimensionRef,
            Option<&SpeedMagnitude>,
            &mut RetainedChasePathSnapshot,
            &mut ActivatingChunks,
        ),
        (LocalAiControlled),
    >,
    targets: Query<(&ChunkPos, &DimensionRef, Has<BelongsToAPlayerFaction>), (With<Being>, Changed<ChunkPos>)>,
    plans: Res<ChaserNavPlans>,
    mut corridor_chunks: Local<Vec<ChunkPos>>,
    mut seen_chunks: Local<HashSet<ChunkPos>>,
) {
    for (chaser_ent, chaser, chaser_gpos, &chaser_chunk_pos, &chaser_dim, speed, mut snapshot, mut activating_chunks) in chasers.iter_mut() {
        let Ok((&target_chunk_pos, &target_dim, target_is_player_faction)) = targets.get(chaser.target) else {
            continue;
        };
        if !target_is_player_faction || target_dim != chaser_dim {
            continue;
        }

        let stale_timeout = retained_target_trail_stale_timeout(speed.map_or(1.0, |speed| speed.0));

        if snapshot.target_chunk_trail.is_empty() {
            extend_target_chunk_trail(&mut snapshot.target_chunk_trail, target_chunk_pos, stale_timeout);
            snapshot.last_target_chunk_pos = target_chunk_pos;
        } else if target_chunk_pos != snapshot.last_target_chunk_pos {
            extend_target_chunk_trail(&mut snapshot.target_chunk_trail, target_chunk_pos, stale_timeout);
            snapshot.last_target_chunk_pos = target_chunk_pos;
        }

        rebuild_connected_chase_chunk_path(
            &mut corridor_chunks,
            &mut seen_chunks,
            Some(*chaser_gpos),
            plans.by_ent.get(&chaser_ent).map(|plan| plan.path_tiles.as_slice()),
            chaser_chunk_pos,
            target_chunk_pos,
        );

        let mut chunk_positions = std::mem::take(&mut snapshot.chunk_positions);
        rebuild_retained_chase_chunk_positions(
            &mut chunk_positions,
            &corridor_chunks,
            &mut seen_chunks,
            &mut snapshot.target_chunk_trail,
            stale_timeout,
            time.delta(),
        );
        snapshot.chunk_positions = chunk_positions;

        if activating_chunks.0 != snapshot.chunk_positions {
            activating_chunks.0.clear();
            activating_chunks.0.extend(snapshot.chunk_positions.iter().copied());
        }

        debug!(target: BEING_SYSTEM, "Updated retained chase snapshot for {:?} to prey chunk {:?}, retained {}, trail {}", chaser_ent, target_chunk_pos, snapshot.chunk_positions.len(), snapshot.target_chunk_trail.len());
    }
}
