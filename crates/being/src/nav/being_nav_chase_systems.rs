use crate::being_components::{Being, Chasing};
use ::being_shared::*;
use ::tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos};
use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_northstar::prelude::*;
use common::log_targets::BEING_SYSTEM;
use movement::movement_components::FinalNormMoveDir;
use movement::movement_components::InputMoveDir;
use movement::movement_components::SpeedMagnitude;
use param_sets::BlockingTileParamSet;
use tilemap::chunking::chunking_components::{ActivateChunksAround, ActivatingChunks};
use std::time::Duration;

use super::being_nav_components::RetainedChasePathSnapshot;
use super::being_nav_resources::{AiNavGrids, ChaserNavPlans};
use super::being_nav_structs::{AiNavGridCache, ChaserNavPlan};
use super::being_nav_helpers::{
    cardinal_step_toward,
    rebuild_connected_chase_chunk_path,
    rebuild_dynamic_blocking,
    rebuild_retained_chase_chunk_positions,
    extend_target_chunk_trail,
    retained_target_trail_stale_timeout,
    retained_target_chunk_trails_match_positions,
};
use crate::prelude::MakeChunkSnapshotForChaser;
use faction::faction_components::BelongsToAPlayerFaction;
use tilemap::chunking::{BeingsWithinChunk, BeingChunkDespawned};

const BOXED_TARGET_RETRY_SECS: f32 = 0.4;
const PARTIAL_TARGET_RETRY_SECS: f32 = 1.0;

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

pub fn rebuild_chaser_nav_plans(
    time: Res<Time>,
    chasers: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            &Chasing,
            Option<&movement::movement_components::SpeedMagnitude>,
        ),
        (With<Being>, LocalAiControlled),
    >,
    beings_query: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&movement::movement_components::SpeedMagnitude>,
        ),
        With<Being>,
    >,
    grids: Res<AiNavGrids>,
    mut plans: ResMut<ChaserNavPlans>,
    mut dynamic_blocking: Local<HashMap<UVec3, Entity>>,
    mut active_chasers: Local<EntityHashSet>,
) {
    active_chasers.clear();

    for (chaser_ent, chaser_gpos, &chaser_dim, to_chase, chaser_speed) in chasers.iter() {
        active_chasers.insert(chaser_ent);

        let chase_interval = if let Ok((_target_ent, target_gpos, &target_dim, prey_speed)) = beings_query.get(to_chase.target) {
            if target_dim != chaser_dim || to_chase.target == chaser_ent {
                Duration::from_secs_f32(0.2)
            } else {
                ChaserNavPlan::rebuild_interval(
                    chaser_speed.map_or(1.0, |speed| speed.0),
                    prey_speed.map_or(1.0, |speed| speed.0),
                    chaser_gpos.taxicab_tile_distance(*target_gpos),
                )
            }
        } else {
            Duration::from_secs_f32(0.2)
        };

        let plan = plans.by_ent.entry(chaser_ent).or_default();
        let timer_finished = plan.rebuild_timer.tick(time.delta()).just_finished();

        let Ok((_target_ent, target_gpos, &target_dim, _prey_speed)) = beings_query.get(to_chase.target) else {
            plan.reset(chase_interval);
            continue;
        };
        if target_dim != chaser_dim || to_chase.target == chaser_ent {
            plan.reset(chase_interval);
            continue;
        };
        let target_pos = *target_gpos;

        if chaser_gpos.taxicab_tile_distance(target_pos) <= to_chase.stop_distance.max(0.0) {
            plan.clear_path_and_retry(Duration::from_secs_f32(0.25), target_pos);
            continue;
        }

        let Some(cache) = grids.by_dim.get(&chaser_dim.0) else {
            plan.clear_path_and_retry(chase_interval, target_pos);
            continue;
        };

        let Some((start, goal)) = cache.local_path_points(*chaser_gpos, target_pos) else {
            plan.clear_path_and_retry(chase_interval, target_pos);
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
                    to_chase.target,
                    *chaser_gpos,
                    start,
                    local,
                    None,
                );
                continue;
            };
            if occupant_ent == chaser_ent || occupant_ent == to_chase.target {
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut dynamic_blocking,
                    cache,
                    chaser_ent,
                    to_chase.target,
                    *chaser_gpos,
                    start,
                    local,
                    None,
                );
                continue;
            }
            let Ok((_blocker_ent, _blocker_gpos, _blocker_dim, blocker_chaser, _blocker_speed)) = chasers.get(occupant_ent) else {
                consider_chase_goal_candidate(
                    &mut best_path_tiles,
                    &mut best_path_is_partial,
                    &mut best_path_remaining,
                    &mut best_path_cost,
                    &mut dynamic_blocking,
                    cache,
                    chaser_ent,
                    to_chase.target,
                    *chaser_gpos,
                    start,
                    local,
                    Some(occupant_ent),
                );
                continue;
            };
            if blocker_chaser.target == to_chase.target {
                blocked_target_approach_count += 1;
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
                to_chase.target,
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
            .map(|prev| (prev.0 - target_pos.0).abs().max_element() >= 2)
            .unwrap_or(true);
        let need_rebuild = timer_finished || target_shifted;
        if !need_rebuild {
            continue;
        }

        if target_boxed_by_same_target_chasers {
            plan.clear_path_and_retry(
                chase_interval.max(Duration::from_secs_f32(BOXED_TARGET_RETRY_SECS)),
                target_pos,
            );
            trace!(target: BEING_SYSTEM, "Delaying chase nav for {:?}: prey {:?} boxed by same-target chasers", chaser_ent, to_chase.target);
            continue;
        }

        if best_path_tiles.is_empty() {
            rebuild_dynamic_blocking(&mut dynamic_blocking, cache, chaser_ent, to_chase.target, start, goal);
            let mut req = PathfindArgs::new(start, goal)
                .astar()
                .partial()
                .blocking(&dynamic_blocking);
            let Some(path) = cache.grid.pathfind(&mut req) else {
                plan.clear_path_and_retry(chase_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS)), target_pos);
                trace!(target: BEING_SYSTEM, "Deferred chase nav retry for {:?}: prey {:?}, dist {:.2}, interval {:.2}s", chaser_ent, to_chase.target, chaser_gpos.taxicab_tile_distance(target_pos), chase_interval.as_secs_f32());
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
                chase_interval.max(Duration::from_secs_f32(PARTIAL_TARGET_RETRY_SECS))
            } else {
                chase_interval
            },
            TimerMode::Once,
        );
        trace!(target: BEING_SYSTEM, "Rebuilt chase nav for {:?}: prey {:?}, dist {:.2}, interval {:.2}s, steps {}", chaser_ent, to_chase.target, chaser_gpos.taxicab_tile_distance(target_pos), chase_interval.as_secs_f32(), plan.path_tiles.len());
    }

    plans.by_ent.retain(|ent, _| active_chasers.contains(ent));
}

pub fn chase_behavior(
    mut blocking_tiles: BlockingTileParamSet,
    mut chasers: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            &Chasing,
        ),
        (With<Being>, LocalAiControlled),
    >,
    targets_query: Query<(Entity, &GlobalTilePos, &::tilemap_shared::DimensionRef)>,
    mut plans: ResMut<ChaserNavPlans>,
    mut input_dirs: Query<&mut InputMoveDir>,
) {
    for (chaser_ent, chaser_gpos, &chaser_dim, to_chase) in chasers.iter_mut() {
        let Ok(mut input_move_dir) = input_dirs.get_mut(chaser_ent) else {
            continue;
        };
        let chaser_pos = *chaser_gpos;
        let Some(target_pos) = to_chase.chase_target_pos(chaser_ent, chaser_dim, &targets_query) else {
            input_move_dir.0 = Vec2::ZERO;
            continue;
        };
        let Some(direct_chase_dir) = chaser_pos.direct_chase_dir(target_pos, to_chase.stop_distance) else {
            input_move_dir.0 = Vec2::ZERO;
            continue;
        };

        let move_input = if let Some(plan) = plans.by_ent.get_mut(&chaser_ent) {
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
        } else {
            direct_chase_dir
        };

        let move_axis = FinalNormMoveDir(move_input).normalize_to_axis_dir();
        if move_axis != IVec2::ZERO {
            let next_pos = GlobalTilePos(chaser_pos.0 + move_axis);
            if blocking_tiles.is_blocked_at_tiles_only(chaser_dim, next_pos, chaser_ent) {
                input_move_dir.0 = Vec2::ZERO;
                if let Some(plan) = plans.by_ent.get_mut(&chaser_ent) {
                    plan.clear_path_and_retry(Duration::from_secs_f32(0.08), target_pos);
                }
                trace!(target: BEING_SYSTEM, "Chase repath after static blocked step for {:?}: prey {:?}, dir {:?}, from {:?}", chaser_ent, to_chase.target, move_axis, chaser_pos);
                continue;
            }
            if blocking_tiles.is_blocked_at(chaser_dim, next_pos, chaser_ent) {
                input_move_dir.0 = Vec2::ZERO;
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
        (With<Being>, LocalAiControlled, Without<ActivateChunksAround>),
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
                if activating_chunks.chunk_positions != *desired_chunks {
                    activating_chunks.chunk_positions.clear();
                    activating_chunks.chunk_positions.extend(desired_chunks.iter().copied());
                }
            }
            None => {
                commands.entity(chaser_ent).try_insert(ActivatingChunks {
                    chunk_positions: desired_chunks.clone(),
                });
            }
        }
    }
}

fn unload_being_for_chunk_despawn(commands: &mut Commands, being_ent: Entity) {
    let mut entity = commands.entity(being_ent);
    entity.try_insert(BackgroundSimulated);
    entity.try_remove::<(Name, ActivateChunksAround, ActivatingChunks, RetainedChasePathSnapshot)>();
}

pub fn on_chunk_with_beings_attempt_unload(
    mut commands: Commands,
    mut reader: MessageReader<BeingChunkDespawned>,
    chunks_query: Query<&BeingsWithinChunk>,
    beings: Query<&Chasing, With<Being>>,
    player_targets: Query<(), (With<Being>, PlayerBeing)>,
    mut writer: MessageWriter<MakeChunkSnapshotForChaser>,
    mut messages: Local<Vec<MakeChunkSnapshotForChaser>>,
) {
    for msg in reader.read() {
        let Ok(beings_within_chunk) = chunks_query.get(msg.chunk_ent) else {
            continue;
        };

        let should_cancel = beings_within_chunk.iter().any(|being_ent| {
            let Ok(chaser) = beings.get(being_ent) else {
                return false;
            };
            player_targets.get(chaser.target).is_ok()
        });

        if should_cancel {
            for being_ent in beings_within_chunk.iter() {
                let Ok(chaser) = beings.get(being_ent) else {
                    continue;
                };
                if player_targets.get(chaser.target).is_err() {
                    continue;
                }
                messages.push(MakeChunkSnapshotForChaser(being_ent));
            }
            debug!(target: BEING_SYSTEM, "Canceled despawn for chunk {:?} because at least one resident must stay loaded", msg.chunk_ent);
            continue;
        }

        for being_ent in beings_within_chunk.iter() {
            unload_being_for_chunk_despawn(&mut commands, being_ent);
        }

        commands.entity(msg.chunk_ent).try_despawn();
        writer.write_batch(messages.drain(..));
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
        let needs_activating_chunks = activating_chunks.map(|chunks| chunks.chunk_positions.as_slice()) != Some(desired_chunks.as_slice());

        let mut entity = commands.entity(*chaser_ent);
        if needs_snapshot || needs_activating_chunks {
            entity.try_insert((
                RetainedChasePathSnapshot {
                    chunk_positions: desired_chunks.clone(),
                    target_chunk_trail,
                    last_target_chunk_pos: *target_chunk_pos,
                },
                ActivatingChunks {
                    chunk_positions: desired_chunks.clone(),
                },
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

        if activating_chunks.chunk_positions != snapshot.chunk_positions {
            activating_chunks.chunk_positions.clear();
            activating_chunks
                .chunk_positions
                .extend(snapshot.chunk_positions.iter().copied());
        }

        debug!(target: BEING_SYSTEM, "Updated retained chase snapshot for {:?} to prey chunk {:?}, retained {}, trail {}", chaser_ent, target_chunk_pos, snapshot.chunk_positions.len(), snapshot.target_chunk_trail.len());
    }
}
