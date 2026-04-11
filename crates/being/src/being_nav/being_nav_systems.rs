
use crate::being_messages::NavOrder;
use ::being_shared::*;
use crate::body::BodySums;
use camera::camera_components::CameraTarget;
use bevy_northstar::{CardinalGrid, grid::GridSettingsBuilder, nav::Nav};
use ::tilemap_shared::*;
use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    ecs::system::SystemParam,
    platform::collections::HashMap,
    prelude::*,
};
use common::common_components::HashId;
use common::log_targets::BEING_SYSTEM;
use param_sets::BlockingTileParamSet;
use ::being_shared::movement_shared_components::{InputMaxSpeed, InputSpeedThrottleMult};

use super::being_nav_resources::*;
use super::being_nav_structs::AiNavGridCache;

const AI_LOD_NEAR_TILES: i32 = 72;
const AI_LOD_MID_TILES: i32 = 144;
const AI_LOD_FAR_TILES: i32 = 288;
const AI_LOD_HYSTERESIS_TILES: i32 = 12;

#[derive(SystemParam)]
pub struct SyncAiNavGridsScratch<'s> {
    needed_dims: Local<'s, EntityHashSet>,
    dim_centers: Local<'s, EntityHashMap<IVec2>>,
    dim_center_counts: Local<'s, EntityHashMap<i32>>,
    dirty_dims: Local<'s, EntityHashSet>,
    loaded_dim_bounds: Local<'s, EntityHashMap<(IVec2, IVec2)>>,
    occupancy_initialized_dims: Local<'s, EntityHashSet>,
    prev_being_state: Local<'s, EntityHashMap<(Entity, GlobalTilePos)>>,
    seen_beings: Local<'s, EntityHashSet>,
    beings_by_dim: Local<'s, EntityHashMap<Vec<(Entity, GlobalTilePos)>>>,
}

#[allow(unused_parens, )]
pub fn ensure_loaded_beings_have_nav_state(
    mut cmd: Commands,
    beings: Query<(Entity, ), (With<Being>, Without<WanderState>, Without<Chasing>, Without<Fleeing>, )>,
) {
    for (being_ent, ) in beings { cmd.entity(being_ent).try_insert(WanderState::default()); }
}

fn lod_level_for_tile_distance(tile_distance: i32) -> u8 {
    if tile_distance <= AI_LOD_NEAR_TILES {
        0
    } else if tile_distance <= AI_LOD_MID_TILES {
        1
    } else if tile_distance <= AI_LOD_FAR_TILES {
        2
    } else {
        3
    }
}

fn lod_level_with_hysteresis(previous_level: u8, tile_distance: i32) -> u8 {
    let d = tile_distance.max(0);
    let near_in = (AI_LOD_NEAR_TILES - AI_LOD_HYSTERESIS_TILES).max(1);
    let near_out = AI_LOD_NEAR_TILES + AI_LOD_HYSTERESIS_TILES;
    let mid_in = (AI_LOD_MID_TILES - AI_LOD_HYSTERESIS_TILES).max(1);
    let mid_out = AI_LOD_MID_TILES + AI_LOD_HYSTERESIS_TILES;
    let far_in = (AI_LOD_FAR_TILES - AI_LOD_HYSTERESIS_TILES).max(1);
    let far_out = AI_LOD_FAR_TILES + AI_LOD_HYSTERESIS_TILES;
    match previous_level {
        0 => {
            if d > near_out {
                lod_level_for_tile_distance(d)
            } else {
                0
            }
        }
        1 => {
            if d <= near_in {
                0
            } else if d > mid_out {
                lod_level_for_tile_distance(d)
            } else {
                1
            }
        }
        2 => {
            if d <= mid_in {
                lod_level_for_tile_distance(d)
            } else if d > far_out {
                3
            } else {
                2
            }
        }
        _ => {
            if d <= far_in {
                lod_level_for_tile_distance(d)
            } else {
                3
            }
        }
    }
}

#[allow(unused_parens, )]
pub fn update_being_lod_levels_from_camera(
    mut cmd: Commands,
    camera_query: Query<(&DimensionRef, &GlobalTransform), (With<CameraTarget>, )>,
    mut beings_query: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            Option<&mut LodLevel>,
        ),
        (With<Being>, LocalAiControlled),
    >,
    dim_map: Res<DimensionEntityMap>,
    mut cameras_by_dim: Local<EntityHashMap<Vec<GlobalTilePos>>>,
) {
    cameras_by_dim.clear();
    let camera_iter = camera_query.iter();
    let (lower, upper) = camera_iter.size_hint();
    cameras_by_dim.reserve(upper.unwrap_or(lower));
    for (&dim_ref, transform) in camera_iter {
        let Some(dim_ent) = dim_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        cameras_by_dim
            .entry(dim_ent)
            .or_default()
            .push(GlobalTilePos::from(transform.translation().xy()));
    }

    for (being_ent, &dim_ref, &being_gpos, lod_level) in beings_query.iter_mut() {
        let Some(dim_ent) = dim_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        let nearest_camera_tile_dist = cameras_by_dim
            .get(&dim_ent)
            .and_then(|camera_gpos| {
                camera_gpos
                    .iter()
                    .map(|&camera_gpos| {
                        let delta = camera_gpos.0 - being_gpos.0;
                        delta.abs().max_element()
                    })
                    .min()
            })
            .unwrap_or(i32::MAX / 4);
        let base_level = lod_level_for_tile_distance(nearest_camera_tile_dist);

        let Some(mut lod_level) = lod_level else {
            cmd.entity(being_ent).try_insert(LodLevel(base_level));
            continue;
        };
        let next_level = lod_level_with_hysteresis(lod_level.0, nearest_camera_tile_dist);
        if lod_level.0 != next_level {
            lod_level.0 = next_level;
        }
    }
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
    mut go_to_query: Query<Option<&mut GoTo>, >,
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
            if let Ok(Some(mut curr_go_to)) = go_to_query.get_mut(being_ent) {
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
        let speed_throttle_mult = order.speed_throttle_mult.clamp(0.0, 1.0);
        let Ok((mut input_speed_throttle_mult, mut input_max_speed)) = input_speed_query.get_mut(being_ent) else {
            continue;
        };
        input_speed_throttle_mult.0 = speed_throttle_mult;
        input_max_speed.0 = order.max_speed.max(0.0);
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

#[allow(unused_parens, )]
pub fn sync_ai_nav_grids(
    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_range: Res<LoadChunksAround>,
    dim_map: Res<DimensionEntityMap>,
    mut param_set: BlockingTileParamSet,
    chasers_query: Query<
        (
            Entity,
            &::tilemap_shared::DimensionRef,
            Option<&ComputedBy>,
            Option<&GoTo>,
            Option<&LodLevel>,
        ),
        With<Being>,
    >,
    beings_query: Query<(Entity, &::tilemap_shared::DimensionRef, Option<&GoTo>, Option<&LodLevel>), With<Being>>,
    mut removed_beings: RemovedComponents<Being>,
    mut grids: ResMut<AiNavGrids>,
    dimension_hash_query: Query<&HashId, With<Dimension>>,
    mut scratch: SyncAiNavGridsScratch,
) {
    let chaser_iter = chasers_query.iter();
    let chaser_count = chaser_iter.size_hint().1.unwrap_or(chaser_iter.size_hint().0);
    scratch.needed_dims.clear();
    scratch.needed_dims.reserve(chaser_count);
    scratch.dim_centers.clear();
    scratch.dim_centers.reserve(chaser_count);
    scratch.dim_center_counts.clear();
    scratch.dim_center_counts.reserve(chaser_count);

    for (being_ent, dim_ref, controlled_by, goto, lod_level) in chaser_iter {
        let Some(goto) = goto else {
            continue;
        };
        let Some(dim_ent) = dim_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_dc_input {
                continue;
            }
        }
        let lod_level = lod_level.map_or(0, |lod_level| lod_level.0);
        let is_critical_nav = matches!(
            goto.source,
            Some(NavOrderSource::Chasing | NavOrderSource::Fleeing)
        );
        if lod_level >= 2 && !is_critical_nav {
            continue;
        }
        scratch.needed_dims.insert(dim_ent);
        let Ok(&gpos) = param_set.gpos_query.get(being_ent) else {
            continue;
        };
        let center = scratch.dim_centers.entry(dim_ent).or_insert(IVec2::ZERO);
        *center += gpos.0;
        *scratch.dim_center_counts.entry(dim_ent).or_insert(0) += 1;
    }

    grids.by_dim.retain(|dim, _| scratch.needed_dims.contains(dim));
    grids
        .center_by_dim
        .retain(|dim, _| scratch.needed_dims.contains(dim));
    scratch.dirty_dims.clear();
    scratch.dirty_dims.reserve(scratch.needed_dims.len());
    scratch.seen_beings.clear();
    scratch.beings_by_dim.clear();
    scratch.beings_by_dim.reserve(scratch.needed_dims.len());
    for removed_being_ent in removed_beings.read() {
        let Some((removed_dim, _)) = scratch.prev_being_state.remove(&removed_being_ent) else {
            continue;
        };
        if scratch.needed_dims.contains(&removed_dim) {
            scratch.dirty_dims.insert(removed_dim);
        }
    }
    for (being_ent, dim_ref, go_to, lod_level) in beings_query.iter() {
        let Some(dim_ent) = dim_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        if !scratch.needed_dims.contains(&dim_ent) {
            continue;
        }
        let lod_level = lod_level.map_or(0, |lod_level| lod_level.0);
        let is_critical_nav = go_to
            .map(|go_to| matches!(go_to.source, Some(NavOrderSource::Chasing | NavOrderSource::Fleeing)))
            .unwrap_or(false);
        if lod_level >= 2 && !is_critical_nav {
            continue;
        }
        let Ok(&gpos) = param_set.gpos_query.get(being_ent) else {
            continue;
        };
        scratch.seen_beings.insert(being_ent);
        scratch
            .beings_by_dim
            .entry(dim_ent)
            .or_default()
            .push((being_ent, gpos));
        let previous = scratch.prev_being_state.insert(being_ent, (dim_ent, gpos));
        let Some((previous_dim, previous_gpos)) = previous else {
            scratch.dirty_dims.insert(dim_ent);
            continue;
        };
        if previous_dim != dim_ent {
            scratch.dirty_dims.insert(previous_dim);
            scratch.dirty_dims.insert(dim_ent);
            continue;
        }
        if previous_gpos != gpos {
            scratch.dirty_dims.insert(dim_ent);
        }
    }
    scratch.prev_being_state.retain(|being_ent, (prev_dim, _)| {
        let keep = scratch.seen_beings.contains(being_ent) && scratch.needed_dims.contains(prev_dim);
        if !keep && scratch.needed_dims.contains(prev_dim) {
            scratch.dirty_dims.insert(*prev_dim);
        }
        keep
    });
    scratch.occupancy_initialized_dims.retain(|dim| scratch.needed_dims.contains(dim));

    let max_side = (((chunk_range.discovery_range as i32 * 2) - 1).max(1) as u32)
        * ChunkPos::CHUNK_SIZE.x.max(1);
    let should_rebuild = grids.rebuild_timer.tick(time.delta()).just_finished();
    let mut rebuilt_grids = 0usize;
    let mut refreshed_occupancy = 0usize;

    scratch.loaded_dim_bounds.clear();
    scratch.loaded_dim_bounds.reserve(scratch.needed_dims.len());
    for (&(dim_ref, chunk_pos), _) in loaded_chunks.0.iter() {
        let Some(chunk_dim_ent) = dim_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        if !scratch.needed_dims.contains(&chunk_dim_ent) {
            continue;
        }
        let cmin = chunk_pos.to_tilepos().0;
        let cmax = cmin + ChunkPos::CHUNK_SIZE.as_ivec2() - IVec2::ONE;
        let bounds = scratch
            .loaded_dim_bounds
            .entry(chunk_dim_ent)
            .or_insert((cmin, cmax));
        bounds.0 = bounds.0.min(cmin);
        bounds.1 = bounds.1.max(cmax);
    }

    for dim in scratch.needed_dims.iter().copied() {
        let Ok(&dim_hash) = dimension_hash_query.get(dim) else {
            continue;
        };
        let Some(&(mut min_tile, max_tile)) = scratch.loaded_dim_bounds.get(&dim) else {
            continue;
        };

        let center = scratch
            .dim_centers
            .get(&dim)
            .zip(scratch.dim_center_counts.get(&dim))
            .map(|(sum, count)| *sum / count.max(&1))
            .unwrap_or((min_tile + max_tile) / 2);

        let mut width = (max_tile.x - min_tile.x + 1).max(3) as u32;
        let mut height = (max_tile.y - min_tile.y + 1).max(3) as u32;
        if width > max_side {
            let half = (max_side as i32) / 2;
            min_tile.x = center.x - half;
            width = max_side;
        }
        if height > max_side {
            let half = (max_side as i32) / 2;
            min_tile.y = center.y - half;
            height = max_side;
        }

        let center_changed = grids
            .center_by_dim
            .get(&dim)
            .map(|prev| (*prev - center).abs().max_element() >= ChunkPos::CHUNK_SIZE.x as i32)
            .unwrap_or(true);
        let needs_new_grid = !grids.by_dim.contains_key(&dim);
        let rebuild_grid = needs_new_grid || should_rebuild || center_changed;
        let mut rebuilt_grid = false;
        if rebuild_grid {
            let mut grid = CardinalGrid::new(
                &GridSettingsBuilder::new_2d(width, height)
                    .chunk_size(8)
                    .build(),
            );
            for y in 0..height {
                for x in 0..width {
                    let world = GlobalTilePos(min_tile + IVec2::new(x as i32, y as i32));
                    if param_set.is_blocked_at_tiles_only(
                        ::tilemap_shared::DimensionRef(dim_hash),
                        world,
                        Entity::PLACEHOLDER,
                    ) {
                        grid.set_nav(UVec3::new(x, y, 0), Nav::Impassable);
                    }
                }
            }
            grid.build();
            grids.by_dim.insert(
                dim,
                AiNavGridCache {
                    min: min_tile,
                    grid,
                    occupied: HashMap::default(),
                    occupied_initialized: true,
                },
            );
            grids.center_by_dim.insert(dim, center);
            rebuilt_grids += 1;
            rebuilt_grid = true;
        }
        let should_refresh_occupancy = rebuilt_grid
            || scratch.dirty_dims.contains(&dim)
            || !scratch.occupancy_initialized_dims.contains(&dim);
        if !should_refresh_occupancy {
            continue;
        }
        let Some(cache) = grids.by_dim.get_mut(&dim) else {
            continue;
        };
        cache.occupied.clear();
        let Some(dim_beings) = scratch.beings_by_dim.get(&dim) else {
            scratch.occupancy_initialized_dims.insert(dim);
            refreshed_occupancy += 1;
            continue;
        };
        cache.occupied.reserve(dim_beings.len());
        for &(being_ent, gpos) in dim_beings.iter() {
            let max_grid = cache.min
                + IVec2::new(
                    cache.grid.width() as i32 - 1,
                    cache.grid.height() as i32 - 1,
                );
            if gpos.0.x < cache.min.x
                || gpos.0.y < cache.min.y
                || gpos.0.x > max_grid.x
                || gpos.0.y > max_grid.y
            {
                continue;
            }
            let local = (gpos.0 - cache.min).as_uvec2();
            cache
                .occupied
                .insert(UVec3::new(local.x, local.y, 0), being_ent);
        }
        scratch.occupancy_initialized_dims.insert(dim);
        refreshed_occupancy += 1;
    }

    if rebuilt_grids > 0 || refreshed_occupancy > 0 {
        trace!(
            target: BEING_SYSTEM,
            "sync_ai_nav_grids: dims={} rebuilt_grids={} refreshed_occupancy={} dirty_dims={}",
            scratch.needed_dims.len(),
            rebuilt_grids,
            refreshed_occupancy,
            scratch.dirty_dims.len()
        );
    }
}
