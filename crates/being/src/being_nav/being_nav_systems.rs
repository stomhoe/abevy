
use crate::being_messages::NavOrder;
use ::being_shared::*;
use crate::body::BodySums;
use bevy_northstar::{CardinalGrid, grid::GridSettingsBuilder, nav::Nav};
use ::tilemap_shared::*;
use bevy::{
    ecs::entity::EntityHashMap,
    ecs::system::SystemParam,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    tasks::{AsyncComputeTaskPool, futures_lite::future},
};
use common::log_targets::BEING_SYSTEM;
use param_sets::BlockingTileParamSet;
use ::being_shared::movement_shared_components::{InputMaxSpeed, InputSpeedThrottleMult};

use super::being_nav_resources::*;
use super::being_nav_structs::AiNavGridCache;

const NAV_GRID_CENTER_SHIFT_TILES: i32 = 12;

#[derive(Default)]
struct NavGridCenterAccum {
    sum_x: i64,
    sum_y: i64,
    weight: i64,
}
impl NavGridCenterAccum {
    fn push(
        &mut self,
        gpos: GlobalTilePos,
        weight: i64,
    ) {
        if weight <= 0 {
            return;
        }
        self.sum_x += gpos.0.x as i64 * weight;
        self.sum_y += gpos.0.y as i64 * weight;
        self.weight += weight;
    }

    fn center(&self) -> Option<IVec2> {
        if self.weight <= 0 {
            return None;
        }
        Some(IVec2::new(
            (self.sum_x / self.weight) as i32,
            (self.sum_y / self.weight) as i32,
        ))
    }
}

fn build_ai_nav_grid_cache(
    input: AiNavGridRebuildInput,
) -> AiNavGridRebuildResult {
    let mut grid = CardinalGrid::new(
        &GridSettingsBuilder::new_2d(input.width.max(3), input.height.max(3)).build(),
    );
    for blocked_tile in input.blocked_tiles.iter().copied() {
        grid.set_nav(
            UVec3::new(blocked_tile.x, blocked_tile.y, 0),
            Nav::Impassable,
        );
    }
    grid.build();

    AiNavGridRebuildResult {
        dim: input.dim,
        generation: input.generation,
        center: input.center,
        cache: AiNavGridCache {
            min: input.min_tile,
            grid,
            occupied: HashMap::default(),
        },
    }
}



#[allow(unused_parens, )]
pub fn ensure_loaded_beings_have_nav_state(
    mut cmd: Commands,
    beings: Query<(Entity, ), (With<Being>, Without<WanderState>, Without<Chasing>, Without<Fleeing>, )>,
) {
    for (being_ent, ) in beings { cmd.entity(being_ent).try_insert(WanderState::default()); }
}

#[allow(unused_parens, )]
pub fn update_goto_from_chasing(
    mut writer: MessageWriter<NavOrder>,
    mut messages: Local<Vec<NavOrder>>,
    chasing_query: Query<(Entity, &::tilemap_shared::DimensionRef, &Chasing, ), (With<Being>, LocalAiControlled, )>,
    targets_query: Query<(Entity, &GlobalTilePos, &::tilemap_shared::DimensionRef), >,
) {
    for (being_ent, &being_dim, chasing, ) in chasing_query.iter() {
        let order = chasing
            .chase_target_pos(being_ent, being_dim, &targets_query)
            .map(|target_pos| {
                NavOrder::new(
                    being_ent,
                    200,
                    NavOrderSource::Chasing,
                    Some(GoTo::new(target_pos, chasing.stop_distance)),
                )
            })
            .unwrap_or_else(|| {
                NavOrder::new(
                    being_ent,
                    200,
                    NavOrderSource::Chasing,
                    None,
                )
            });
        messages.push(order);
    }
    writer.write_batch(messages.drain(..));
}

fn choose_flee_target_pos(
    blocking_tiles: &mut BlockingTileParamSet,
    being_ent: Entity,
    being_dim: ::tilemap_shared::DimensionRef,
    being_gpos: GlobalTilePos,
    flee_from_gpos: GlobalTilePos,
    _desired_distance_tiles: f32,
    avoid_tile_tags: &BlacklistedTags,
) -> Option<(GlobalTilePos, i32, u8)> {
    let empty_whitelist = WhitelistedTags::default();
    let empty_whitelist = WhitelistedSpawnTileTagsRef(&empty_whitelist);
    let avoid_tile_tags = BlacklistedSpawnTileTagsRef(avoid_tile_tags);

    fn tile_is_valid_flee_step(
        blocking_tiles: &mut BlockingTileParamSet,
        being_ent: Entity,
        being_dim: ::tilemap_shared::DimensionRef,
        candidate: GlobalTilePos,
        avoid_tile_tags: &BlacklistedSpawnTileTagsRef<'_>,
        empty_whitelist: &WhitelistedSpawnTileTagsRef<'_>,
    ) -> bool {
        if blocking_tiles.is_blocked_at(being_dim, candidate, being_ent, ) {
            return false;
        }
        if avoid_tile_tags.0.is_empty() {
            return true;
        }
        blocking_tiles.allowed_at_refs(
            being_dim,
            candidate,
            being_ent,
            empty_whitelist,
            avoid_tile_tags,
        )
    }

    fn candidate_escape_score(
        blocking_tiles: &mut BlockingTileParamSet,
        being_ent: Entity,
        being_dim: ::tilemap_shared::DimensionRef,
        being_gpos: GlobalTilePos,
        flee_from_gpos: GlobalTilePos,
        candidate: GlobalTilePos,
        step_dir: IVec2,
        avoid_tile_tags: &BlacklistedSpawnTileTagsRef<'_>,
        empty_whitelist: &WhitelistedSpawnTileTagsRef<'_>,
    ) -> Option<(i32, u8)> {
        if !tile_is_valid_flee_step(
            blocking_tiles,
            being_ent,
            being_dim,
                candidate,
                avoid_tile_tags,
                empty_whitelist,
        ) {
            return None;
        }

        let base_dist = (being_gpos.0 - flee_from_gpos.0).abs().element_sum() as f32;
        let candidate_dist = (candidate.0 - flee_from_gpos.0).abs().element_sum() as f32;
        if candidate_dist <= base_dist {
            return None;
        }
        let dist_gain = candidate_dist - base_dist;

        let mut open_exits = 0u8;
        for neighbor_dir in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
            let neighbor = GlobalTilePos(candidate.0 + neighbor_dir);
            if tile_is_valid_flee_step(
                blocking_tiles,
                being_ent,
                being_dim,
                neighbor,
                avoid_tile_tags,
                empty_whitelist,
            ) {
                open_exits += 1;
            }
        }

        let mut forward_run = 0i32;
        if step_dir != IVec2::ZERO {
            for run_dist in 1..=3 {
                let forward = GlobalTilePos(candidate.0 + step_dir * run_dist);
                if tile_is_valid_flee_step(
                    blocking_tiles,
                    being_ent,
                    being_dim,
                    forward,
                    avoid_tile_tags,
                    empty_whitelist,
                ) {
                    forward_run += 1;
                } else {
                    break;
                }
            }
        }

        let mut score = candidate_dist as i32 * 28
            + dist_gain as i32 * 80
            + (open_exits as i32) * 26
            + forward_run * 18;
        if dist_gain < 0.0 {
            score -= 220;
        }
        if open_exits <= 1 {
            score -= 260;
        }
        if open_exits == 0 {
            score -= 400;
        }
        Some((score, open_exits))
    }

    let away = being_gpos.0 - flee_from_gpos.0;
    let primary_step = if away == IVec2::ZERO {
        IVec2::X
    } else if away.x.abs() >= away.y.abs() {
        IVec2::new(away.x.signum(), 0)
    } else {
        IVec2::new(0, away.y.signum())
    };
    let lateral_step = IVec2::new(-primary_step.y, primary_step.x);
    let mut best: Option<(GlobalTilePos, i32, u8)> = None;
    for step in [primary_step, lateral_step, -lateral_step, -primary_step] {
        if step == IVec2::ZERO {
            continue;
        }
        for dist in 1..=8 {
            let candidate = GlobalTilePos(being_gpos.0 + step * dist);
            if blocking_tiles.is_blocked_at(being_dim, candidate, being_ent, ) {
                break;
            }
            let Some((score, open_exits)) = candidate_escape_score(
                blocking_tiles,
                being_ent,
                being_dim,
                being_gpos,
                flee_from_gpos,
                candidate,
                step,
                &avoid_tile_tags,
                &empty_whitelist,
            ) else {
                continue;
            };
            if best
                .map(|(_, best_score, best_open_exits)| {
                    score > best_score || (score == best_score && open_exits > best_open_exits)
                })
                .unwrap_or(true)
            {
                best = Some((candidate, score, open_exits));
            }
        }
    }
    best
}

fn resolve_flee_wander_cfg(
    member_of: Option<&SquadMemberOf>,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    bit_map: &BeingInstTemplateEntityMap,
    race_map: &RaceEntityMap,
    wander_cfg_query: &Query<&WanderSeri>,
) -> WanderSeri {
    if let Some(member_of) = member_of {
        if let Ok(cfg) = wander_cfg_query.get(member_of.0) {
            return cfg.clone();
        }
    }
    if let Some(bit_ref) = bit_ref {
        if let Ok(bit_ent) = bit_map.0.get_cloned(bit_ref.0)
            && let Ok(cfg) = wander_cfg_query.get(bit_ent)
        {
            return cfg.clone();
        }
    }
    if let Some(race_ref) = race_ref {
        if let Ok(race_ent) = race_map.0.get_cloned(race_ref.0)
            && let Ok(cfg) = wander_cfg_query.get(race_ent)
        {
            return cfg.clone();
        }
    }
    WanderSeri::default()
}

fn resolve_flee_avoid_tile_tags(
    cfg: &WanderSeri,
    has_avoid_blacklisted_spawn_tiles: bool,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    bit_map: &BeingInstTemplateEntityMap,
    race_map: &RaceEntityMap,
    blacklisted_spawn_tile_tags_query: &Query<&::tilemap_shared::BlacklistedSpawnTileTags>,
) -> BlacklistedTags {
    let mut avoid_tile_tags = BlacklistedTags::new(&cfg.avoid_tile_tags);
    if !has_avoid_blacklisted_spawn_tiles {
        return avoid_tile_tags;
    }
    if let Some(bit_ref) = bit_ref {
        if let Ok(bit_ent) = bit_map.0.get_cloned(bit_ref.0)
            && let Ok(bit_blacklisted_spawn_tile_tags) = blacklisted_spawn_tile_tags_query.get(bit_ent)
        {
            if !bit_blacklisted_spawn_tile_tags.0.is_empty() {
                avoid_tile_tags.extend_from(&bit_blacklisted_spawn_tile_tags.0);
                return avoid_tile_tags;
            }
        }
    }
    if let Some(race_ref) = race_ref {
        if let Ok(race_ent) = race_map.0.get_cloned(race_ref.0)
            && let Ok(race_blacklisted_spawn_tile_tags) = blacklisted_spawn_tile_tags_query.get(race_ent)
        {
            avoid_tile_tags.extend_from(&race_blacklisted_spawn_tile_tags.0);
        }
    }
    avoid_tile_tags
}

#[allow(unused_parens, )]
pub fn update_goto_from_fleeing(
    mut cmd: Commands,
    mut writer: MessageWriter<NavOrder>,
    mut blocking_tiles: BlockingTileParamSet,
    flee_query: Query<(Entity, &::tilemap_shared::DimensionRef, &Fleeing, Option<&SquadMemberOf>, Has<DoAvoidBlacklistedSpawnTilesForWander>, ), (With<Being>, LocalAiControlled, )>,
    body_weight_query: Query<&BodyWeightSum>,
    held_body_query: Query<&HeldBody>,
    body_sums_query: Query<&BodySums>,
    wander_cfg_query: Query<&WanderSeri>,
    blacklisted_spawn_tile_tags_query: Query<&::tilemap_shared::BlacklistedSpawnTileTags>,
    bit_map: Res<BeingInstTemplateEntityMap>,
    race_map: Res<RaceEntityMap>,
    mut messages: Local<Vec<NavOrder>>,
) {
    for (being_ent, &being_dim, fleeing, member_of, has_avoid_blacklisted_spawn_tiles, ) in flee_query.iter() {
        let Ok(&being_gpos) = blocking_tiles.gpos_query.get(being_ent) else {
            continue;
        };
        let mut primary_threat: Option<(Entity, GlobalTilePos, f32)> = None;
        for threat_ent in fleeing.threats.iter().copied() {
            let Ok(&flee_from_gpos) = blocking_tiles.gpos_query.get(threat_ent) else {
                continue;
            };
            let strength = body_weight_query.get(threat_ent).map_or(0.0, |weight_sum| weight_sum.0.max(0.0))
                + held_body_query
                    .get(threat_ent)
                    .ok()
                    .and_then(|held_body| body_sums_query.get(held_body.entity()).ok())
                    .map_or(0.0, |body_sums| body_sums.current_hp.max(0.0));
            let dist = (being_gpos.0 - flee_from_gpos.0).abs().element_sum().max(1) as f32;
            let threat_score = strength / dist;
            if primary_threat
                .map(|(_, _, best_score)| threat_score > best_score)
                .unwrap_or(true)
            {
                primary_threat = Some((threat_ent, flee_from_gpos, threat_score));
            }
        }
        let Some((primary_threat_ent, flee_from_gpos, _)) = primary_threat else {
            cmd.entity(being_ent).try_remove::<Fleeing>();
            continue;
        };
        let flee_dist = (being_gpos.0 - flee_from_gpos.0).abs().element_sum();
        if (flee_dist as f32) >= fleeing.desired_distance_tiles {
            cmd.entity(being_ent).try_remove::<Fleeing>();
            messages.push(NavOrder::new(
                being_ent,
                255,
                NavOrderSource::Fleeing,
                None,
            ));
            continue;
        }
        let bit_ref = blocking_tiles.get_being_bit_ref(being_ent);
        let race_ref = blocking_tiles.get_being_race_ref(being_ent);
        let cfg = resolve_flee_wander_cfg(member_of, bit_ref, race_ref, &bit_map, &race_map, &wander_cfg_query);
        let avoid_tile_tags = resolve_flee_avoid_tile_tags(
            &cfg,
            has_avoid_blacklisted_spawn_tiles,
            bit_ref,
            race_ref,
            &bit_map,
            &race_map,
            &blacklisted_spawn_tile_tags_query,
        );
        let Some((target_pos, score, open_exits)) = choose_flee_target_pos(
            &mut blocking_tiles,
            being_ent,
            being_dim,
            being_gpos,
            flee_from_gpos,
            fleeing.desired_distance_tiles,
            &avoid_tile_tags,
        ) else {
            cmd.entity(being_ent).try_remove::<Fleeing>();
            cmd.entity(being_ent).try_insert(Hunting::new(primary_threat_ent));
            messages.push(NavOrder::new(
                being_ent,
                255,
                NavOrderSource::Fleeing,
                None,
            ));
            continue;
        };
        trace!(
            target: BEING_SYSTEM,
            "Flee target selected for {:?}: from={:?} threat={:?} target={:?} score={} exits={}",
            being_ent,
            being_gpos,
            flee_from_gpos,
            target_pos,
            score,
            open_exits
        );
        messages.push(NavOrder::new(
            being_ent,
            255,
            NavOrderSource::Fleeing,
            // Fleeing is already gated by the threat distance check above; the
            // escape target itself should be pursued normally instead of being
            // treated as an additional stop-radius.
            Some(GoTo::new(target_pos, 0.0)),
        ));
    }
    writer.write_batch(messages.drain(..));
}

#[allow(unused_parens, )]
pub fn apply_nav_orders(
    mut cmd: Commands,
    time: Res<Time>,
    mut reader: MessageReader<NavOrder>,
    mut selected_by_ent: Local<EntityHashMap<NavOrder>>,
    mut go_to_query: Query<&mut GoTo>,
    mut input_speed_query: Query<(&mut InputSpeedThrottleMult, &mut InputMaxSpeed, ), (), >,
) {
    selected_by_ent.clear();
    for order in reader.read() {
        let should_replace = selected_by_ent
            .get(&order.being_ent)
            .map(|curr| {
                order.priority > curr.priority
                    || (order.priority == curr.priority
                        && order.source.tie_break_rank() > curr.source.tie_break_rank())
            })
            .unwrap_or(true);
        if should_replace {
            selected_by_ent.insert(order.being_ent, order.clone());
        }
    }
    let tick = time.elapsed().as_millis() as u32;
    for (being_ent, order) in selected_by_ent.drain() {
        if let Some(go_to) = order.go_to {
            if let Ok(mut curr_go_to) = go_to_query.get_mut(being_ent) {
                *curr_go_to = GoTo::with_source(
                    go_to.pos,
                    go_to.stop_distance,
                    order.source,
                    tick,
                );
            } else {
                cmd.entity(being_ent).try_insert(GoTo::with_source(
                    go_to.pos,
                    go_to.stop_distance,
                    order.source,
                    tick,
                ));
            }
        } else {
            cmd.entity(being_ent).try_remove::<GoTo>();
        }
        let speed_throttle_mult = order.speed_throttle_mult.0.clamp(0.0, 1.0);
        let Ok((mut input_speed_throttle_mult, mut input_max_speed)) = input_speed_query.get_mut(being_ent) else {
            continue;
        };
        input_speed_throttle_mult.0 = speed_throttle_mult;
        input_max_speed.0 = order.max_speed.0.max(0.0);
        trace!(target: BEING_SYSTEM, "NavOrder winner {:?}: source={:?} priority={} goto={:?} throttle={:.2}", being_ent, order.source, order.priority, order.go_to, speed_throttle_mult);
    }
}

#[allow(unused_parens, )]
pub fn clear_nav_outputs_for_beings_without_nav_state(
    mut cmd: Commands,
    mut query: Query<(Entity, Has<WanderState>, Has<Chasing>, Has<Fleeing>, Option<&mut GoTo>, ), (With<Being>, LocalAiControlled, )>,
    mut input_speed_query: Query<(&mut InputSpeedThrottleMult, &mut InputMaxSpeed, ), (), >,
) {
    for (being_ent, has_wandering, has_chasing, has_fleeing, _, ) in query.iter_mut() {
        if has_wandering || has_chasing || has_fleeing {
            continue;
        }
        let Ok((mut input_speed_throttle_mult, mut input_max_speed)) = input_speed_query.get_mut(being_ent) else {
            continue;
        };
        cmd.entity(being_ent).try_remove::<GoTo>();
        input_speed_throttle_mult.0 = 1.0;
        input_max_speed.0 = f32::MAX;
    }
}
#[derive(SystemParam)]
pub struct NavGridsLocals<'s> {
    active_dims: Local<'s, HashSet<DimensionRef>>,
    dirty_dims: Local<'s, HashSet<DimensionRef>>,
    loaded_chunk_bounds_by_dim: Local<'s, HashMap<DimensionRef, (IVec2, IVec2)>>,
    center_accum_by_dim: Local<'s, HashMap<DimensionRef, NavGridCenterAccum>>,
    rebuild_inputs: Local<'s, Vec<AiNavGridRebuildInput>>,
    stale_dims: Local<'s, Vec<DimensionRef>>,
}
#[allow(unused_parens, )]
pub fn sync_ai_nav_grids(
    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
    dim_map: Res<DimensionEntityMap>,
    param_set: BlockingTileParamSet,
    beings_at_gpos: Res<BeingsAtGpos>,
    tile_blocked_gpos_counts: Res<AiNavTileBlockedGposCounts>,
    chasers_query: Query<
        (
            Entity,
            &DimensionRef,
            Option<&GoTo>,
            Option<&LodLevel>,
        ),
        (With<Being>, LocalAiControlled, ),
    >,
    mut nav_grid_dirty_msgs: MessageReader<AiNavGridDirtyDim>,
    mut grids: ResMut<AiNavGrids>,
    mut rebuild_tasks: ResMut<AiNavGridRebuildTasks>,
    mut scratch: NavGridsLocals,
) {
    let mut finished_task_ix = 0usize;
    while finished_task_ix < rebuild_tasks.tasks.len() {
        let Some(results) = future::block_on(future::poll_once(&mut rebuild_tasks.tasks[finished_task_ix])) else {
            finished_task_ix += 1;
            continue;
        };
        for result in results {
            let Some(&pending_generation) = rebuild_tasks.pending_generation_by_dim.get(&result.dim) else {
                continue;
            };
            if pending_generation != result.generation {
                continue;
            }

            let dim = result.dim;
            let generation = result.generation;
            let width = result.cache.grid.width();
            let height = result.cache.grid.height();
            grids.by_dim.insert(dim, result.cache);
            grids.center_by_dim.insert(dim, result.center);
            rebuild_tasks.pending_generation_by_dim.remove(&dim);
            rebuild_tasks.pending_dims.remove(&dim);
            trace!(target: BEING_SYSTEM, "Applied async AI nav grid rebuild: dim={:?} generation={} size={}x{}", dim, generation, width, height);
        }
        drop(rebuild_tasks.tasks.swap_remove(finished_task_ix));
    }

    scratch.active_dims.clear();
    scratch.center_accum_by_dim.clear();
    for (being_ent, &being_dim, go_to, lod_level, ) in chasers_query.iter() {
        let Some(_dim_ent) = dim_map.0.get_opt(being_dim.0).copied() else {
            continue;
        };
        let Ok(being_gpos) = param_set.gpos_query.get(being_ent) else {
            continue;
        };
        scratch.active_dims.insert(being_dim);

        let lod_level = lod_level.map_or(0, |lod_level| lod_level.0);
        let mut weight = 1i64;
        if go_to.is_some() {
            weight += 1;
        }
        if lod_level <= 1 {
            weight += 1;
        }
        scratch
            .center_accum_by_dim
            .entry(being_dim)
            .or_default()
            .push(*being_gpos, weight);
    }

    scratch.stale_dims.clear();
    scratch.stale_dims.reserve(grids.by_dim.len());
    for &dim in grids.by_dim.keys() {
        if !scratch.active_dims.contains(&dim) {
            scratch.stale_dims.push(dim);
        }
    }
    for stale_dim in scratch.stale_dims.drain(..) {
        grids.by_dim.remove(&stale_dim);
        grids.center_by_dim.remove(&stale_dim);
        rebuild_tasks.pending_dims.remove(&stale_dim);
        rebuild_tasks.pending_generation_by_dim.remove(&stale_dim);
    }

    scratch.dirty_dims.clear();
    for nav_grid_dirty_msg in nav_grid_dirty_msgs.read() {
        scratch.dirty_dims.insert(nav_grid_dirty_msg.dim);
    }

    let rebuild_timer_finished = grids.rebuild_timer.tick(time.delta()).just_finished();

    scratch.loaded_chunk_bounds_by_dim.clear();
    scratch.loaded_chunk_bounds_by_dim.reserve(scratch.active_dims.len());
    for (&(loaded_dim, chunk_pos), _) in loaded_chunks.0.iter() {
        if !scratch.active_dims.contains(&loaded_dim) {
            continue;
        }
        let entry = scratch
            .loaded_chunk_bounds_by_dim
            .entry(loaded_dim)
            .or_insert((chunk_pos.0, chunk_pos.0));
        entry.0 = entry.0.min(chunk_pos.0);
        entry.1 = entry.1.max(chunk_pos.0);
    }

    for &dim in scratch.active_dims.iter() {
        if scratch.loaded_chunk_bounds_by_dim.contains_key(&dim) {
            continue;
        }
        grids.by_dim.remove(&dim);
        grids.center_by_dim.remove(&dim);
        rebuild_tasks.pending_dims.remove(&dim);
        rebuild_tasks.pending_generation_by_dim.remove(&dim);
    }

    scratch.rebuild_inputs.clear();
    scratch.rebuild_inputs.reserve(scratch.active_dims.len());
    for &dim in scratch.active_dims.iter() {
        let Some(&(min_chunk, max_chunk)) = scratch.loaded_chunk_bounds_by_dim.get(&dim) else {
            continue;
        };
        let chunk_span = max_chunk - min_chunk + IVec2::ONE;
        if chunk_span.x <= 0 || chunk_span.y <= 0 {
            continue;
        }
        let min_tile = min_chunk * ChunkPos::CHUNK_SIZE.as_ivec2();
        let width = chunk_span.x as u32 * ChunkPos::CHUNK_SIZE.x;
        let height = chunk_span.y as u32 * ChunkPos::CHUNK_SIZE.y;
        let desired_center = scratch
            .center_accum_by_dim
            .get(&dim)
            .and_then(NavGridCenterAccum::center)
            .or_else(|| grids.center_by_dim.get(&dim).copied())
            .unwrap_or(min_tile + IVec2::new(width as i32 / 2, height as i32 / 2));
        let grid_shape_changed = if let Some(cache) = grids.by_dim.get(&dim) {
            cache.min != min_tile || cache.grid.width() != width || cache.grid.height() != height
        } else {
            true
        };
        let center_shift = grids
            .center_by_dim
            .get(&dim)
            .map(|center| (*center - desired_center).abs().max_element())
            .unwrap_or(i32::MAX);
        let dirty = scratch.dirty_dims.contains(&dim);
        let should_rebuild = grid_shape_changed
            || dirty
            || (rebuild_timer_finished && center_shift >= NAV_GRID_CENTER_SHIFT_TILES);
        if !should_rebuild {
            continue;
        }

        let has_pending = rebuild_tasks.pending_dims.contains(&dim);
        if has_pending && !dirty && !grid_shape_changed {
            continue;
        }

        let blocked_tiles =
            tile_blocked_gpos_counts.blocked_tiles_for_dim(dim, min_tile, width, height);
        let generation = rebuild_tasks.next_generation;
        rebuild_tasks.next_generation = rebuild_tasks.next_generation.wrapping_add(1);
        rebuild_tasks.pending_dims.insert(dim);
        rebuild_tasks.pending_generation_by_dim.insert(dim, generation);
        scratch.rebuild_inputs.push(AiNavGridRebuildInput {
            dim,
            generation,
            min_tile,
            width,
            height,
            center: desired_center,
            blocked_tiles,
        });
        trace!(target: BEING_SYSTEM, "Queued async AI nav grid rebuild: dim={:?} generation={} size={}x{} center={:?}", dim, generation, width, height, desired_center);
    }

    if !scratch.rebuild_inputs.is_empty() {
        let rebuild_inputs: Vec<AiNavGridRebuildInput> = scratch.rebuild_inputs.drain(..).collect();
        let task = AsyncComputeTaskPool::get().spawn(async move {
            rebuild_inputs
                .into_iter()
                .map(build_ai_nav_grid_cache)
                .collect::<Vec<AiNavGridRebuildResult>>()
        });
        rebuild_tasks.tasks.push(task);
    }

    for (&dim, cache) in grids.by_dim.iter_mut() {
        if !scratch.active_dims.contains(&dim) {
            continue;
        }
        cache.occupied.clear();
    }

    for &dim in scratch.active_dims.iter() {
        let Some(occupied_positions) = beings_at_gpos.occupied_positions_in_dim(dim) else {
            continue;
        };
        let Some(cache) = grids.by_dim.get_mut(&dim) else {
            continue;
        };
        for &gpos in occupied_positions.iter() {
            let Some(&being_ent) = beings_at_gpos.get_beings_at_pos(dim, gpos).first() else {
                continue;
            };
            let Some(local) = cache.local_from_gpos(gpos) else {
                continue;
            };
            cache.occupied.entry(local).or_insert(being_ent);
        }
    }
}
