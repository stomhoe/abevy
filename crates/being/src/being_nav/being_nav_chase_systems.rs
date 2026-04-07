use crate::{
    being_interaction_zone_helper::resolve_being_interaction_zone,
    being_messages::MakeChunkSnapshotForChaser,
};
use ::being_shared::*;
use faction_shared::BelongsToAPlayerFaction;
use ::tilemap_shared::*;
use bevy::{
    ecs::{entity::EntityHashSet, system::SystemParam},
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

#[derive(Clone, Copy)]
struct SharedChaseRebuildJob {
    chaser_ent: Entity,
    chaser_dim: DimensionRef,
    chaser_gpos: GlobalTilePos,
    target_ent: Entity,
    target_pos: GlobalTilePos,
    target_dist: u32,
    goto_interval: Duration,
}

#[derive(SystemParam)]
pub struct RebuildGotoNavPlansScratch<'s> {
    dynamic_blocking: Local<'s, HashMap<UVec3, Entity>>,
    active_chasers: Local<'s, EntityHashSet>,
    active_chase_targets: Local<'s, EntityHashSet>,
    shared_rebuild_jobs: Local<'s, Vec<SharedChaseRebuildJob>>,
    shared_goal_owners: Local<'s, HashMap<GlobalTilePos, Entity>>,
    chase_zone_tiles: Local<'s, Vec<GlobalTilePos>>,
    chase_goal_tiles: Local<'s, Vec<GlobalTilePos>>,
    chase_slot_tiles: Local<'s, Vec<GlobalTilePos>>,
    chase_seed_goal_tiles: Local<'s, Vec<(GlobalTilePos, u32)>>,
    chase_zone_hits: Local<'s, Vec<GlobalTilePos>>,
}

const BOXED_TARGET_RETRY_SECS: f32 = 0.4;
const PARTIAL_TARGET_RETRY_SECS: f32 = 1.0;
const TARGET_SHIFT_REBUILD_TILES: i32 = 1;
const BLOCKED_STEP_RETRY_SECS: f32 = 0.08;
const SHARED_CHASE_SUPPORT_RING_DEPTH: u32 = 2;
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
    slot_tiles: &mut Vec<GlobalTilePos>,
    seed_goal_tiles: &mut Vec<(GlobalTilePos, u32)>,
    zone_hits: &mut Vec<GlobalTilePos>,
) {
    zone_tiles.clear();
    goal_tiles.clear();
    slot_tiles.clear();
    seed_goal_tiles.clear();
    zone_hits.clear();

    zone_tiles.reserve(collision_zone.perimeter_size().max(4) as usize);
    collision_zone.gather_zone_positions(CardinalDirection::South, target_pos.to_pixelpos(), zone_tiles);
    if zone_tiles.is_empty() {
        zone_tiles.push(target_pos);
    }
    zone_tiles.sort_unstable_by_key(|pos| (pos.0.x, pos.0.y));
    zone_tiles.dedup();

    let mut seen_goal_tiles = HashSet::with_capacity(zone_tiles.len().saturating_mul(8));
    goal_tiles.reserve(zone_tiles.len().saturating_mul(4));
    for zone_tile in zone_tiles.iter().copied() {
        for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
            let candidate = GlobalTilePos(zone_tile.0 + delta);
            if zone_tiles
                .binary_search_by_key(&(candidate.0.x, candidate.0.y), |pos| (pos.0.x, pos.0.y))
                .is_ok()
            {
                continue;
            }

            if !collision_zone.has_accessible_border_position_for_checked_pos(
                target_pos,
                candidate,
                zone_hits,
            ) {
                continue;
            }

            let Some(local) = cache.local_from_gpos(candidate) else {
                continue;
            };
            if !cache.grid.is_passable(local) {
                continue;
            }
            if seen_goal_tiles.insert(candidate) {
                goal_tiles.push(candidate);
            }
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

    goal_tiles.sort_unstable_by_key(|pos| {
        (pos.taxicab_tile_distance(target_pos) as u32, pos.0.x, pos.0.y)
    });
    goal_tiles.dedup();

    let mut seen_slot_tiles = HashSet::with_capacity(goal_tiles.len().saturating_mul(8));
    let mut frontier = Vec::with_capacity(goal_tiles.len());
    frontier.extend(goal_tiles.iter().copied());
    slot_tiles.reserve(goal_tiles.len().saturating_mul((SHARED_CHASE_SUPPORT_RING_DEPTH as usize).saturating_add(1)));
    for &goal_tile in goal_tiles.iter() {
        seen_slot_tiles.insert(goal_tile);
        slot_tiles.push(goal_tile);
    }
    for _ in 0..SHARED_CHASE_SUPPORT_RING_DEPTH {
        let mut next_frontier = Vec::with_capacity(frontier.len().saturating_mul(4));
        let mut ring_candidates: Vec<(GlobalTilePos, u32, u32)> = Vec::with_capacity(frontier.len().saturating_mul(4));
        for source_tile in frontier.iter().copied() {
            for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let candidate = GlobalTilePos(source_tile.0 + delta);
                if !seen_slot_tiles.insert(candidate) {
                    continue;
                }
                if zone_tiles
                    .binary_search_by_key(&(candidate.0.x, candidate.0.y), |pos| (pos.0.x, pos.0.y))
                    .is_ok()
                {
                    continue;
                }
                let Some(local) = cache.local_from_gpos(candidate) else {
                    continue;
                };
                if !cache.grid.is_passable(local) {
                    continue;
                }
                let adjacent_goal_count = [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y]
                    .into_iter()
                    .filter(|delta| goal_tiles
                        .binary_search_by_key(
                            &((candidate.0 + *delta).x, (candidate.0 + *delta).y),
                            |pos| (pos.0.x, pos.0.y),
                        )
                        .is_ok())
                    .count() as u32;
                let support_score = adjacent_goal_count.saturating_mul(4);
                ring_candidates.push((candidate, support_score, candidate.taxicab_tile_distance(target_pos) as u32));
                next_frontier.push(candidate);
            }
        }
        ring_candidates.sort_unstable_by_key(|(pos, support_score, target_dist, )| {
            (std::cmp::Reverse(*support_score), *target_dist, pos.0.x, pos.0.y)
        });
        slot_tiles.extend(ring_candidates.into_iter().map(|(candidate, _, _, )| candidate));
        frontier.clear();
        frontier.extend(next_frontier.drain(..));
        if frontier.is_empty() {
            break;
        }
    }

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
    slot_tiles: &mut Vec<GlobalTilePos>,
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
        slot_tiles,
        seed_goal_tiles,
        zone_hits,
    );
    let flow_field = SharedChaseFlowField::build(
        cache,
        target_dim.0,
        target_pos,
        goal_tiles,
        slot_tiles,
        seed_goal_tiles,
    )?;
    debug!(
        target: BEING_SYSTEM,
        "Built shared chase flow target={:?} dim={:?} anchor={:?} goals={} slots={} open_goals={} zone_tiles={}",
        target_ent,
        target_dim,
        target_pos,
        flow_field.goal_tiles.len(),
        flow_field.slot_tiles.len(),
        flow_field.seed_goal_tiles.len(),
        zone_tiles.len(),
    );
    chase_fields.by_target.insert(target_ent, flow_field);
    Some(())
}

fn shared_flow_target_reached(
    flow_field: &SharedChaseFlowField,
    chaser_pos: GlobalTilePos,
) -> bool {
    flow_field.is_goal_tile(chaser_pos)
}

fn choose_shared_chase_goal(
    flow_field: &SharedChaseFlowField,
    cache: &AiNavGridCache,
    chaser_ent: Entity,
    reserved_goal: Option<GlobalTilePos>,
    reserved_goals: &HashMap<GlobalTilePos, Entity>,
) -> Option<GlobalTilePos> {
    if let Some(reserved_goal) = reserved_goal.filter(|goal| flow_field.is_slot_tile(*goal)) {
        match reserved_goals.get(&reserved_goal) {
            Some(&owner) if owner != chaser_ent => {}
            _ => return Some(reserved_goal),
        }
    }
    if flow_field.slot_tiles.is_empty() {
        return None;
    }

    for goal_tile in flow_field.slot_tiles.iter().copied() {
        if reserved_goals.contains_key(&goal_tile) {
            continue;
        }
        let Some(local) = cache.local_from_gpos(goal_tile) else {
            continue;
        };
        if cache.occupied.contains_key(&local) {
            continue;
        }
        return Some(goal_tile);
    }

    None
}

fn rebuild_shared_chase_plan_for_job(
    job: SharedChaseRebuildJob,
    cache: &AiNavGridCache,
    chase_fields: &mut SharedChaseFlowFields,
    plans: &mut ChaserNavPlans,
    dynamic_blocking: &mut HashMap<UVec3, Entity>,
    reserved_goals: &mut HashMap<GlobalTilePos, Entity>,
) {
    let Some(flow_field) = chase_fields.by_target.get(&job.target_ent) else {
        return;
    };
    let Some(plan) = plans.by_ent.get_mut(&job.chaser_ent) else {
        return;
    };

    let Some(reserved_goal) = choose_shared_chase_goal(
        flow_field,
        cache,
        job.chaser_ent,
        plan.reserved_shared_goal,
        reserved_goals,
    ) else {
        plan.clear_shared_goal();
        plan.clear_path_and_retry(
            job.goto_interval.max(Duration::from_secs_f32(BOXED_TARGET_RETRY_SECS)),
            job.target_pos,
        );
        trace!(target: BEING_SYSTEM, "Deferred shared chase for {:?}: prey {:?} has no unblocked reserved slot near {:?}", job.chaser_ent, job.target_ent, job.target_pos);
        return;
    };
    plan.reserved_shared_goal = Some(reserved_goal);
    reserved_goals.insert(reserved_goal, job.chaser_ent);

    let Some((start, goal)) = cache.local_path_points(job.chaser_gpos, reserved_goal) else {
        if reserved_goals.get(&reserved_goal) == Some(&job.chaser_ent) {
            reserved_goals.remove(&reserved_goal);
        }
        plan.clear_path_and_retry(
            job.goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)),
            job.target_pos,
        );
        trace!(target: BEING_SYSTEM, "Deferred shared chase plan rebuild for {:?}: prey {:?} cannot path from {:?} to reserved slot {:?}", job.chaser_ent, job.target_ent, job.chaser_gpos, reserved_goal);
        return;
    };

    rebuild_dynamic_blocking(dynamic_blocking, cache, job.chaser_ent, Entity::PLACEHOLDER, start, goal);
    let mut req = PathfindArgs::new(start, goal)
        .astar()
        .partial()
        .blocking(dynamic_blocking);
    let Some(path) = cache.grid.pathfind(&mut req) else {
        if reserved_goals.get(&reserved_goal) == Some(&job.chaser_ent) {
            reserved_goals.remove(&reserved_goal);
        }
        plan.clear_path_and_retry(
            job.goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)),
            job.target_pos,
        );
        trace!(target: BEING_SYSTEM, "Deferred shared chase plan rebuild for {:?}: prey {:?} unreachable from {:?} to reserved slot {:?}", job.chaser_ent, job.target_ent, job.chaser_gpos, reserved_goal);
        return;
    };

    plan.path_tiles.clear();
    plan.path_tiles.extend(path.path().iter().map(|step| GlobalTilePos(step.xy().as_ivec2() + cache.min)));
    plan.next_step_ix = 0;
    plan.last_target_pos = Some(job.target_pos);
    plan.holds_at_partial_endpoint = path.is_partial();
    plan.rebuild_timer = Timer::new(
        if path.is_partial() {
            job.goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS))
        } else {
            job.goto_interval
        },
        TimerMode::Once,
    );
    trace!(target: BEING_SYSTEM, "Rebuilt shared chase plan for {:?}: prey {:?}, anchor {:?}, reserved {:?}, steps {}", job.chaser_ent, job.target_ent, job.target_pos, reserved_goal, plan.path_tiles.len());
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
    blocking_tiles: BlockingTileParamSet,
    goto_beings: Query<
        (
            Entity,
            &DimensionRef,
            Option<&GoTo>,
            Option<&SpeedMagnitude>,
            Option<&Chasing>,
            Option<&WanderState>,
            Option<&LodLevel>,
        ),
        (With<Being>, LocalAiControlled),
    >,
    grids: Res<AiNavGrids>,
    targets_query: Query<&DimensionRef, ()>,
    interaction_zones_query: Query<&InteractionZones, >,
    mut chase_fields: ResMut<SharedChaseFlowFields>,
    mut plans: ResMut<ChaserNavPlans>,
    mut scratch: RebuildGotoNavPlansScratch,
) {
    scratch.active_chasers.clear();
    scratch.active_chase_targets.clear();
    scratch.shared_rebuild_jobs.clear();
    scratch.shared_goal_owners.clear();

    for (chaser_ent, &chaser_dim, goto, chaser_speed, chasing, wander_state, lod_level, ) in goto_beings.iter() {
        let Ok(chaser_gpos) = blocking_tiles.gpos_query.get(chaser_ent) else {
            continue;
        };
        let chaser_gpos = *chaser_gpos;
        let chaser_gpos = &chaser_gpos;
        scratch.active_chasers.insert(chaser_ent);

        let Some(goto) = goto else {
            plans.by_ent.entry(chaser_ent).or_default().reset(Duration::from_secs_f32(0.25));
            continue;
        };
        let lod_level = lod_level.map_or(0, |lod_level| lod_level.0);
        let is_critical_nav = matches!(
            goto.source,
            Some(NavOrderSource::Chasing | NavOrderSource::Fleeing)
        );
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
        if lod_level >= 2 && !is_critical_nav {
            let target_shifted = plan
                .last_target_pos
                .map(|prev| (prev.0 - target_pos.0).abs().max_element() >= TARGET_SHIFT_REBUILD_TILES)
                .unwrap_or(true);
            let lazy_interval = if lod_level >= 3 {
                Duration::from_secs_f32(1.50)
            } else {
                Duration::from_secs_f32(0.90)
            };
            if plan.path_tiles.is_empty() || timer_finished || target_shifted {
                plan.rebuild_timer = Timer::new(lazy_interval, TimerMode::Once);
                plan.last_target_pos = Some(target_pos);
                plan.holds_at_partial_endpoint = false;
            }
            continue;
        }

        let Some(cache) = grids.by_dim.get(&chaser_dim.0) else {
            plan.clear_path_and_retry(goto_interval, target_pos);
            continue;
        };

        let shared_target_ent = if goto.source == Some(NavOrderSource::Chasing) {
            chasing.map(|chasing| chasing.target)
        } else if goto.source == Some(NavOrderSource::Wandering)
            && wander_state.is_some_and(WanderState::is_meeting)
        {
            wander_state.and_then(WanderState::meeting_anchor)
        } else {
            None
        };
        if let Some(shared_target_ent) = shared_target_ent {
            let Ok(target_gpos) = blocking_tiles.gpos_query.get(shared_target_ent) else {
                plan.clear_path_and_retry(goto_interval, target_pos);
                continue;
            };
            let target_pos = *target_gpos;
            let Ok(target_dim) = targets_query.get(shared_target_ent) else {
                plan.clear_path_and_retry(goto_interval, target_pos);
                continue;
            };
            let target_dim = *target_dim;
            if target_dim != chaser_dim {
                plan.clear_path_and_retry(goto_interval, target_pos);
                continue;
            }

            scratch.active_chase_targets.insert(shared_target_ent);
            let target_shifted = plan
                .last_target_pos
                .map(|prev| (prev.0 - target_pos.0).abs().max_element() >= TARGET_SHIFT_REBUILD_TILES)
                .unwrap_or(true);
            let need_rebuild = plan.path_tiles.is_empty() || timer_finished || target_shifted;
            let target_bit_ref = blocking_tiles.get_being_bit_ref(shared_target_ent);
            let target_race_ref = blocking_tiles.get_being_race_ref(shared_target_ent);

            let collision_zone = resolve_being_interaction_zone(
                interaction_zones_query.get(shared_target_ent).ok(),
                target_bit_ref,
                target_race_ref,
                InteractionZones::COLLISION,
                &interaction_zones_query,
            );
            collect_chase_goal_tiles(
                cache,
                target_pos,
                &collision_zone,
                &mut scratch.chase_zone_tiles,
                &mut scratch.chase_goal_tiles,
                &mut scratch.chase_slot_tiles,
                &mut scratch.chase_seed_goal_tiles,
                &mut scratch.chase_zone_hits,
            );
            let needs_flow_rebuild = chase_fields
                .by_target
                .get(&shared_target_ent)
                .map(|field| {
                    !field.matches_grid(cache, target_dim.0, target_pos)
                        || field.goal_tiles != *scratch.chase_goal_tiles
                        || field.slot_tiles != *scratch.chase_slot_tiles
                        || field.seed_goal_tiles != *scratch.chase_seed_goal_tiles
                })
                .unwrap_or(true);
            if needs_flow_rebuild {
                let Some(()) = rebuild_shared_chase_flow_field(
                    &mut chase_fields,
                    cache,
                    shared_target_ent,
                    target_dim,
                    target_pos,
                    target_bit_ref,
                    target_race_ref,
                    &interaction_zones_query,
                    &mut scratch.chase_zone_tiles,
                    &mut scratch.chase_goal_tiles,
                    &mut scratch.chase_slot_tiles,
                    &mut scratch.chase_seed_goal_tiles,
                    &mut scratch.chase_zone_hits,
                ) else {
                    plan.clear_path_and_retry(
                        goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)),
                        target_pos,
                    );
                    trace!(target: BEING_SYSTEM, "Deferred shared flow rebuild for {:?}: target {:?} has no valid goal tiles at {:?}", chaser_ent, shared_target_ent, target_pos);
                    continue;
                };
            }
            let Some(flow_field) = chase_fields.by_target.get(&shared_target_ent) else {
                plan.clear_path_and_retry(
                    goto_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)),
                    target_pos,
                );
                continue;
            };
            if let Some(reserved_goal) = plan.reserved_shared_goal.filter(|goal| flow_field.is_slot_tile(*goal)) {
                scratch.shared_goal_owners.insert(reserved_goal, chaser_ent);
            }
            if !need_rebuild {
                continue;
            }
            scratch.shared_rebuild_jobs.push(SharedChaseRebuildJob {
                chaser_ent,
                chaser_dim,
                chaser_gpos: *chaser_gpos,
                target_ent: shared_target_ent,
                target_pos,
                target_dist: chaser_gpos.taxicab_tile_distance(target_pos) as u32,
                goto_interval,
            });
            continue;
        }

        plan.clear_shared_goal();
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
                    &mut scratch.dynamic_blocking,
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
                    &mut scratch.dynamic_blocking,
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
            let Ok((_blocker_ent, _blocker_dim, blocker_goto, _blocker_speed, _blocker_chasing, _blocker_wander_state, _blocker_lod_level, )) = goto_beings.get(occupant_ent) else {
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut scratch.dynamic_blocking,
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
            if blocker_goto.is_some_and(|blocker_goto| blocker_goto.pos == target_pos) {
                blocked_target_approach_count += 1;
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut scratch.dynamic_blocking,
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
                &mut scratch.dynamic_blocking,
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
                        &mut scratch.dynamic_blocking,
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
                        &mut scratch.dynamic_blocking,
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
                let Ok((_blocker_ent, _blocker_dim, blocker_goto, _blocker_speed, _blocker_chasing, _blocker_wander_state, _blocker_lod_level, )) = goto_beings.get(occupant_ent) else {
                    consider_chase_goal_candidate(
                        &mut best_path_tiles,
                        &mut best_path_is_partial,
                        &mut best_path_remaining,
                        &mut best_path_cost,
                        &mut scratch.dynamic_blocking,
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
                if blocker_goto.is_some_and(|blocker_goto| blocker_goto.pos == target_pos) {
                    consider_chase_goal_candidate(
                        &mut best_path_tiles,
                        &mut best_path_is_partial,
                        &mut best_path_remaining,
                        &mut best_path_cost,
                        &mut scratch.dynamic_blocking,
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
                    &mut scratch.dynamic_blocking,
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
            rebuild_dynamic_blocking(&mut scratch.dynamic_blocking, cache, chaser_ent, Entity::PLACEHOLDER, start, goal);
            let mut req = PathfindArgs::new(start, goal)
                .astar()
                .partial()
                .blocking(&scratch.dynamic_blocking);
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

    scratch.shared_rebuild_jobs.sort_unstable_by_key(|job| (job.target_dist, job.chaser_ent.index_u32()));
    for job in scratch.shared_rebuild_jobs.iter().copied() {
        let Some(cache) = grids.by_dim.get(&job.chaser_dim.0) else {
            continue;
        };
        rebuild_shared_chase_plan_for_job(
            job,
            cache,
            &mut chase_fields,
            &mut plans,
            &mut scratch.dynamic_blocking,
            &mut scratch.shared_goal_owners,
        );
    }

    plans.by_ent.retain(|ent, _| scratch.active_chasers.contains(ent));
    chase_fields
        .by_target
        .retain(|target_ent, field| scratch.active_chase_targets.contains(target_ent) && grids.by_dim.contains_key(&field.dim));
}

#[allow(unused_parens, )]
pub fn goto_behavior(
    mut blocking_tiles: BlockingTileParamSet,
    mut goto_beings: Query<
        (
            Entity,
            &DimensionRef,
            Option<&GoTo>,
            Option<&Chasing>,
            Option<&WanderState>,
        ),
        (With<Being>, LocalAiControlled),
    >,
    grids: Res<AiNavGrids>,
    chase_fields: Res<SharedChaseFlowFields>,
    mut plans: ResMut<ChaserNavPlans>,
    mut input_dirs: Query<&mut InputMoveDir>,
        mut last_shared_dirs: Local<HashMap<Entity, IVec2>>,
) {
    for (chaser_ent, &chaser_dim, goto, chasing, wander_state, ) in goto_beings.iter_mut() {
        let Ok(mut input_move_dir) = input_dirs.get_mut(chaser_ent) else {
            continue;
        };
        let Ok(chaser_pos) = blocking_tiles.gpos_query.get(chaser_ent) else {
            continue;
        };
        let chaser_pos = *chaser_pos;
        let chaser_pos = &chaser_pos;
        let Some(goto) = goto else {
            if wander_state.is_some_and(WanderState::is_meeting) {
                last_shared_dirs.remove(&chaser_ent);
                continue;
            }
            input_move_dir.0 = Vec2::ZERO;
            last_shared_dirs.remove(&chaser_ent);
            continue;
        };
        let target_pos = goto.pos;

        let shared_target_ent = if goto.source == Some(NavOrderSource::Chasing) {
            chasing.map(|chasing| chasing.target)
        } else if goto.source == Some(NavOrderSource::Wandering)
            && wander_state.is_some_and(WanderState::is_meeting)
        {
            wander_state.and_then(WanderState::meeting_anchor)
        } else {
            None
        };
        let shared_flow_field = shared_target_ent.and_then(|shared_target_ent| {
                let cache = grids.by_dim.get(&chaser_dim.0)?;
                chase_fields
                    .by_target
                    .get(&shared_target_ent)
                    .filter(|flow_field| flow_field.matches_grid(cache, chaser_dim.0, target_pos))
                    .map(|flow_field| (cache, flow_field))
            });
        let shared_reserved_goal = plans.by_ent.get(&chaser_ent).and_then(|plan| plan.reserved_shared_goal);
        let shared_reserved_is_support = shared_flow_field
            .as_ref()
            .and_then(|(_, flow_field, )| shared_reserved_goal.map(|reserved_goal| !flow_field.is_goal_tile(reserved_goal)))
            .unwrap_or(false);
        if let Some((_cache, flow_field, )) = shared_flow_field {
            let shared_target_reached = shared_reserved_goal
                .map(|reserved_goal| reserved_goal == *chaser_pos)
                .unwrap_or_else(|| shared_flow_target_reached(flow_field, *chaser_pos));
            if shared_target_reached {
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

        let move_input = if let Some(plan) = plans.by_ent.get_mut(&chaser_ent) {
            match plan.next_step(*chaser_pos) {
                Some(next) => {
                    let desired = cardinal_step_toward(next.0 - chaser_pos.0);
                    if desired == IVec2::ZERO {
                        direct_chase_dir
                    } else if shared_reserved_is_support {
                        desired.as_vec2()
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
                None if plan.reserved_shared_goal.is_some() => Vec2::ZERO,
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
    player_targets: Query<&ChunkPos, (BeingOfPlayerFaction)>,
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
