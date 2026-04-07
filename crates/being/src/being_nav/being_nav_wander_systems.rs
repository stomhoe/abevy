

use crate::being_messages::NavOrder;
use ::being_shared::*;
use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    ecs::system::SystemParam,
    prelude::*,
};
use common::common_tag_components::TagSet;
use common::log_targets::WANDER_SYSTEM;
use movement::movement_components::FinalNormMoveDir;
use sprite_animation_shared::MirrorHolderStateForSprite;
use ::param_sets::*;
use std::time::Duration;
use ::tilemap_shared::*;

fn wander_lod_interval_for_level(level: u8) -> Duration {
    match level {
        0 => Duration::ZERO,
        1 => Duration::from_secs_f32(0.10),
        2 => Duration::from_secs_f32(0.25),
        _ => Duration::from_secs_f32(0.50),
    }
}

fn advance_wander_lod_tick(
    being_ent: Entity,
    dt: f32,
    lod_level: u8,
    lod_interval_secs: f32,
    lod_secs_left_by_ent: &mut EntityHashMap<f32>,
    lod_accum_secs_by_ent: &mut EntityHashMap<f32>,
) -> Option<f32> {
    let dt = dt.max(0.0);
    let accum_secs = lod_accum_secs_by_ent.entry(being_ent).or_insert(0.0);
    *accum_secs += dt;
    if lod_level == 0 {
        lod_secs_left_by_ent.insert(being_ent, 0.0);
        let elapsed = *accum_secs;
        *accum_secs = 0.0;
        return Some(elapsed);
    }
    let lod_secs_left = lod_secs_left_by_ent
        .entry(being_ent)
        .or_insert(0.0);
    if *lod_secs_left > 0.0 {
        *lod_secs_left = (*lod_secs_left - dt).max(0.0);
    }
    if *lod_secs_left > 0.0 {
        return None;
    }
    *lod_secs_left = lod_interval_secs.max(0.0);
    let elapsed = *accum_secs;
    *accum_secs = 0.0;
    Some(elapsed)
}

fn collect_entity_avoidance(
    blocking_tiles: &mut BlockingTileParamSet,
    self_ent: Entity,
    self_dim: &DimensionRef,
    cfg: &WanderSeri,
    move_speed: f32,
    beings_at_gpos: &BeingsAtGpos,
    threat_query: &Query<
        (
            Entity,
            &DimensionRef,
            Option<&SquadMemberOf>,
        ),
        (With<Being>, ),
    >,
    tag_query: &Query<&TagSet>,
    nearby_threats: &mut EntityHashSet,
) -> Vec2 {
    if cfg.avoid_race_tags.is_empty() && cfg.avoid_bit_tags.is_empty() && cfg.avoid_pack_tags.is_empty() {
        return Vec2::ZERO;
    }

    let max_radius = cfg.max_avoid_being_radius();
    if max_radius <= 0.0 {
        return Vec2::ZERO;
    }

    let mut avoidance = Vec2::ZERO;
    let Ok(&self_pos) = blocking_tiles.gpos_query.get(self_ent) else {
        return Vec2::ZERO;
    };
    let radius_tiles = max_radius.ceil().max(1.0) as i32;
    nearby_threats.clear();
    for dy in -radius_tiles..=radius_tiles {
        for dx in -radius_tiles..=radius_tiles {
            let scan_gpos = GlobalTilePos(self_pos.0 + IVec2::new(dx, dy));
            nearby_threats.extend(
                beings_at_gpos
                    .get_beings_at_pos(*self_dim, scan_gpos)
                    .iter()
                    .copied(),
            );
        }
    }
    for threat_ent in nearby_threats.iter().copied() {
        if threat_ent == self_ent {
            continue;
        }
        let Ok((_, &threat_dim, member_of, )) = threat_query.get(threat_ent) else {
            continue;
        };
        if threat_dim != *self_dim {
            continue;
        }
        let Ok(&threat_pos) = blocking_tiles.gpos_query.get(threat_ent) else {
            continue;
        };
        let delta = self_pos.0 - threat_pos.0;
        let delta_vec = delta.as_vec2();
        let distance = delta_vec.length();
        if distance <= 0.0 || distance > max_radius {
            continue;
        }

        if let Some(bit_ref) = blocking_tiles.get_being_bit_ref(threat_ent) {
            let Ok(bit_tags) = tag_query.get(bit_ref.0) else {
                continue;
            };
            let Some(spec) = AvoidBeingSpec::strongest_avoidance_spec(&cfg.avoid_bit_tags, bit_tags) else {
                continue;
            };
            avoidance += spec.strongest_entity_avoidance(delta_vec, distance, move_speed);
        }
        if let Some(race_ref) = blocking_tiles.get_being_race_ref(threat_ent) {
            let Ok(race_tags) = tag_query.get(race_ref.0) else {
                continue;
            };
            let Some(spec) = AvoidBeingSpec::strongest_avoidance_spec(&cfg.avoid_race_tags, race_tags) else {
                continue;
            };
            avoidance += spec.strongest_entity_avoidance(delta_vec, distance, move_speed);
        }
        if let Some(member_of) = member_of {
            let Ok(pack_tags) = tag_query.get(member_of.0) else {
                continue;
            };
            let Some(spec) = AvoidBeingSpec::strongest_avoidance_spec(&cfg.avoid_pack_tags, pack_tags) else {
                continue;
            };
            avoidance += spec.strongest_entity_avoidance(delta_vec, distance, move_speed);
        }
    }

    let max_avoidance = move_speed * 1.5;
    if avoidance.length_squared() > max_avoidance * max_avoidance {
        avoidance = avoidance.normalize_or_zero() * max_avoidance;
    }
    avoidance
}

fn orbit_target_allowed_for_pack_members(
    blocking_tiles: &mut BlockingTileParamSet,
    wander_cfg_query: &Query<&WanderSeri>,
    blacklisted_spawn_tile_tags_query: &Query<&BlacklistedSpawnTileTags>,
    squad_ent: Entity,
    pack_members: &[(Entity, DimensionRef, bool)],
    dim_ref: DimensionRef,
    target_gpos: GlobalTilePos,
) -> bool {
    let empty_whitelist = WhitelistedTags::default();
    let empty_whitelist = WhitelistedSpawnTileTagsRef(&empty_whitelist);
    let fallback_cfg = WanderSeri::default();
    for &(member_ent, member_dim, has_avoid_blacklisted_spawn_tiles) in pack_members {
        if member_dim != dim_ref {
            continue;
        }
        let bit_ref = blocking_tiles.get_being_bit_ref(member_ent);
        let race_ref = blocking_tiles.get_being_race_ref(member_ent);
        let cfg = common::query_fallback_get!(
            wander_cfg_query,
            Some(squad_ent),
            bit_ref.map(|bit_ref| bit_ref.0),
            race_ref.map(|race_ref| race_ref.0),
        )
        .unwrap_or(&fallback_cfg);
        let avoid_tile_tags = cfg.resolve_wander_avoid_tile_tags(
            has_avoid_blacklisted_spawn_tiles,
            bit_ref,
            race_ref,
            blacklisted_spawn_tile_tags_query,
        );
        let avoid_spawn_tile_tags = BlacklistedSpawnTileTagsRef(&avoid_tile_tags);
        if !blocking_tiles.allowed_at_refs(
            dim_ref,
            target_gpos,
            member_ent,
            &empty_whitelist,
            &avoid_spawn_tile_tags,
        ) {
            return false;
        }
    }
    true
}

fn pick_clear_cardinal_dir(
    rng: &mut impl rand::Rng,
    blocking_tiles: &mut BlockingTileParamSet,
    dim_ref: DimensionRef,
    being_ent: Entity,
    gpos: GlobalTilePos,
    skip_dir: Option<CardinalDirection>,
) -> Option<CardinalDirection> {
    const CANDIDATE_DIRS: [CardinalDirection; 4] = [
        CardinalDirection::South,
        CardinalDirection::West,
        CardinalDirection::North,
        CardinalDirection::East,
    ];
    let start = rng.random_range(0..CANDIDATE_DIRS.len());
    for offset in 0..CANDIDATE_DIRS.len() {
        let candidate_dir = CANDIDATE_DIRS[(start + offset) % CANDIDATE_DIRS.len()];
        if skip_dir.is_some_and(|skip_dir| skip_dir == candidate_dir) {
            continue;
        }
        let candidate_gpos = GlobalTilePos(gpos.0 + candidate_dir.to_dir_vec());
        if !blocking_tiles.is_blocked_at_tiles_only(dim_ref, candidate_gpos, being_ent) {
            return Some(candidate_dir);
        }
    }
    None
}

fn pick_detour_axis_around_blocker(
    rng: &mut impl rand::Rng,
    blocking_tiles: &mut BlockingTileParamSet,
    dim_ref: DimensionRef,
    being_ent: Entity,
    gpos: GlobalTilePos,
    forward_axis: IVec2,
) -> Option<IVec2> {
    if forward_axis == IVec2::ZERO {
        return None;
    }
    let left_axis = IVec2::new(-forward_axis.y, forward_axis.x);
    let right_axis = IVec2::new(forward_axis.y, -forward_axis.x);
    let left_first = rng.random_bool(0.5);
    let side_axes = if left_first {
        [left_axis, right_axis]
    } else {
        [right_axis, left_axis]
    };
    let mut best_axis = None;
    let mut best_score = i32::MIN;
    for side_axis in side_axes {
        let side_gpos = GlobalTilePos(gpos.0 + side_axis);
        if blocking_tiles.is_blocked_at_tiles_only(dim_ref, side_gpos, being_ent) {
            continue;
        }
        let side_then_forward_gpos = GlobalTilePos(side_gpos.0 + forward_axis);
        let side_then_forward_clear = !blocking_tiles.is_blocked_at_tiles_only(
            dim_ref,
            side_then_forward_gpos,
            being_ent,
        );
        let score = if side_then_forward_clear { 2 } else { 1 };
        if score > best_score {
            best_score = score;
            best_axis = Some(side_axis);
        }
    }
    best_axis
}

#[allow(unused_parens, )]
#[derive(SystemParam)]
pub struct WanderBehaviorQueryParams<'w, 's> {
    blocking_tiles: BlockingTileParamSet<'w, 's>,
    beings: Query<
        'w,
        's,
        (
            Entity,
            &'static DimensionRef,
            &'static mut WanderState,
            Option<&'static LodLevel>,
            Option<&'static SquadMemberOf>,
            Has<DoAvoidBlacklistedSpawnTilesForWander>,
        ),
        (
            With<Being>,
            Without<Fleeing>,
            LocalAiControlled,
        ),
    >,
    wander_cfg_query: Query<'w, 's, &'static WanderSeri>,
    blacklisted_spawn_tile_tags_query: Query<'w, 's, &'static BlacklistedSpawnTileTags>,
    threat_query: Query<
        'w,
        's,
        (
            Entity,
            &'static DimensionRef,
            Option<&'static SquadMemberOf>,
        ),
        (With<Being>, ),
    >,
    beings_at_gpos: Res<'w, BeingsAtGpos>,
    tag_query: Query<'w, 's, &'static TagSet>,
    pack_center_query: Query<'w, 's, &'static SquadAvgCenterPerDim>,
}

#[allow(unused_parens, )]
#[derive(SystemParam)]
pub struct WanderBehaviorLocals<'s> {
    nearby_threats: Local<'s, EntityHashSet>,
    lod_secs_left_by_ent: Local<'s, EntityHashMap<f32>>,
    lod_accum_secs_by_ent: Local<'s, EntityHashMap<f32>>,
    seen_wanderers: Local<'s, EntityHashSet>,
    pack_members_by_squad: Local<'s, EntityHashMap<Vec<(Entity, DimensionRef, bool)>>>,
    messages: Local<'s, Vec<NavOrder>>,
    facing_messages: Local<'s, Vec<MirrorHolderStateForSprite>>,
}

#[allow(unused_parens, )]
pub fn wander_behavior(
    mut writer: MessageWriter<NavOrder>,
    mut facing_writer: MessageWriter<MirrorHolderStateForSprite>,
    time: Res<Time>,
    queries: WanderBehaviorQueryParams,
    locals: WanderBehaviorLocals,
) {
    let WanderBehaviorQueryParams {
        mut blocking_tiles,
        mut beings,
        wander_cfg_query,
        blacklisted_spawn_tile_tags_query,
        threat_query,
        beings_at_gpos,
        tag_query,
        pack_center_query,
    } = queries;
    let WanderBehaviorLocals {
        mut nearby_threats,
        mut lod_secs_left_by_ent,
        mut lod_accum_secs_by_ent,
        mut seen_wanderers,
        mut pack_members_by_squad,
        mut messages,
        mut facing_messages,
    } = locals;

    let mut rng = rand::rng();
    let dt = time.delta_secs();
    let mut evaluated_wanderers = 0usize;
    let mut skipped_for_lod = 0usize;
    seen_wanderers.clear();
    pack_members_by_squad.clear();
    {
        let pack_member_iter = beings.iter_mut();
        pack_members_by_squad.reserve(pack_member_iter.size_hint().1.unwrap_or(pack_member_iter.size_hint().0));
    for (member_ent, &member_dim, _, _, member_of, has_avoid_blacklisted_spawn_tiles) in pack_member_iter {
            let Some(member_of) = member_of else {
                continue;
            };
            pack_members_by_squad
                .entry(member_of.0)
                .or_default()
                .push((member_ent, member_dim, has_avoid_blacklisted_spawn_tiles));
        }
    }
    for (pred_ent, &dim_ref, mut state, lod_level, member_of, has_avoid_blacklisted_spawn_tiles, ) in beings.iter_mut() {
        seen_wanderers.insert(pred_ent);
        evaluated_wanderers += 1;
        let Ok(&gpos) = blocking_tiles.gpos_query.get(pred_ent) else {
            continue;
        };
        let bit_ref = blocking_tiles.get_being_bit_ref(pred_ent);
        let race_ref = blocking_tiles.get_being_race_ref(pred_ent);
        let fallback_cfg = WanderSeri::default();
        let cfg = common::query_fallback_get!(
            wander_cfg_query,
            member_of.map(|member_of| member_of.0),
            bit_ref.map(|bit_ref| bit_ref.0),
            race_ref.map(|race_ref| race_ref.0),
        )
        .unwrap_or(&fallback_cfg);
        if state.is_uninitialized() {
            state.initialize(&mut rng, cfg);
        }
        let avoid_tile_tags = cfg.resolve_wander_avoid_tile_tags(
            has_avoid_blacklisted_spawn_tiles,
            bit_ref,
            race_ref,
            &blacklisted_spawn_tile_tags_query,
        );
        let empty_whitelist = WhitelistedTags::default();
        let empty_whitelist = WhitelistedSpawnTileTagsRef(&empty_whitelist);
        let avoid_spawn_tile_tags = BlacklistedSpawnTileTagsRef(&avoid_tile_tags);
        let lod_level = lod_level.map_or(0, |lod_level| lod_level.0);
        let Some(elapsed_secs) = advance_wander_lod_tick(
            pred_ent,
            dt,
            lod_level,
            wander_lod_interval_for_level(lod_level).as_secs_f32(),
            &mut lod_secs_left_by_ent,
            &mut lod_accum_secs_by_ent,
        ) else {
            skipped_for_lod += 1;
            continue;
        };
        let mut input_dir = state.advance_motion(elapsed_secs, &mut rng, cfg);

        if lod_level <= 1 {
            if let Some(member_of) = member_of {
                if let Some(pack_center) = pack_center_query
                    .get(member_of.0)
                    .ok()
                    .and_then(|pack_center| pack_center.0.get(&dim_ref).copied())
                {
                    let pack_delta = pack_center.0 - gpos.0;
                    let pack_distance = pack_delta.as_vec2().length();
                    if cfg.max_drift.is_finite() && pack_distance > cfg.max_drift {
                        if let Some(return_dir) = state.maybe_apply_pack_drift_return(
                            &mut rng,
                            cfg,
                            gpos,
                            pack_center,
                        ) {
                            trace!(
                                target: WANDER_SYSTEM,
                                "wander_behavior: max_drift return correction being={:?} dim={:?} gpos={:?} pack_center={:?} hard_flip={} pending_return={:?}",
                                pred_ent,
                                dim_ref.0,
                                gpos,
                                pack_center,
                                cfg.allow_hard_flips_to_return,
                                state.has_pack_return_dir(),
                            );
                            input_dir = return_dir;
                        }
                    } else if cfg.pack_orbit_radius > 0.0 {
                        let pack_members = pack_members_by_squad
                            .get(&member_of.0)
                            .map(Vec::as_slice)
                            .unwrap_or(&[]);
                        input_dir += state.pack_orbit_pull(
                            elapsed_secs,
                            &mut rng,
                            cfg,
                            pack_center,
                            gpos,
                            |target_gpos| {
                                orbit_target_allowed_for_pack_members(
                                    &mut blocking_tiles,
                                    &wander_cfg_query,
                                    &blacklisted_spawn_tile_tags_query,
                                    member_of.0,
                                    pack_members,
                                    dim_ref,
                                    target_gpos,
                                )
                            },
                        );
                    }
                }
            }

            input_dir += collect_entity_avoidance(
                &mut blocking_tiles,
                pred_ent,
                &dim_ref,
                &cfg,
                state.current_speed_mult_or_zero(),
                &beings_at_gpos,
                &threat_query,
                &tag_query,
                &mut nearby_threats,
            );

            // Check if currently in an undesirable tile, and if so, scan for nearby desirable tile
            if !avoid_tile_tags.is_empty() {
                if !blocking_tiles.allowed_at_refs(
                    dim_ref,
                    gpos,
                    pred_ent,
                    &empty_whitelist,
                    &avoid_spawn_tile_tags,
                ) {
                    if let Some(target_pos) = blocking_tiles.find_closest_allowed_gpos_refs(
                        dim_ref,
                        gpos,
                        pred_ent,
                        GposSearchConfig::wander(),
                        &empty_whitelist,
                        &avoid_spawn_tile_tags,
                    ) {
                        let delta = target_pos.0 - gpos.0;
                        input_dir = delta.as_vec2().normalize_or_zero() * (state.current_speed_mult_or_zero() * 2.0);
                    } else {
                        input_dir = Vec2::ZERO;
                    }
                } else if input_dir != Vec2::ZERO {
                    // Normal avoidance logic for movement
                    let step = if input_dir.x.abs() >= input_dir.y.abs() {
                        IVec2::new(input_dir.x.signum() as i32, 0)
                    } else {
                        IVec2::new(0, input_dir.y.signum() as i32)
                    };
                    let next = GlobalTilePos(gpos.0 + step);
                    if !blocking_tiles.allowed_at_refs(
                        dim_ref,
                        next,
                        pred_ent,
                        &empty_whitelist,
                        &avoid_spawn_tile_tags,
                    ) {
                        input_dir = Vec2::ZERO;
                    }
                }
            }
        }
        if lod_level == 0 && input_dir != Vec2::ZERO {
            let axis = FinalNormMoveDir(input_dir).normalize_to_axis_dir();
            if axis != IVec2::ZERO {
                let next = GlobalTilePos(gpos.0 + axis);
                if blocking_tiles.is_blocked_at_tiles_only(dim_ref, next, pred_ent) {
                    if let Some(detour_axis) = pick_detour_axis_around_blocker(
                        &mut rng,
                        &mut blocking_tiles,
                        dim_ref,
                        pred_ent,
                        gpos,
                        axis,
                    ) {
                        input_dir = detour_axis.as_vec2() * state.current_speed_mult_or_zero();
                    } else {
                        let current_dir = CardinalDirection::from_dir_vec(axis);
                        if let Some(next_dir) = pick_clear_cardinal_dir(
                            &mut rng,
                            &mut blocking_tiles,
                            dim_ref,
                            pred_ent,
                            gpos,
                            Some(current_dir),
                        ) {
                            state.set_motion_dir(next_dir, &mut rng, cfg);
                            input_dir = next_dir.to_dir_vec().as_vec2() * state.current_speed_mult_or_zero();
                        } else {
                            state.expire_motion_dir();
                            input_dir = Vec2::ZERO;
                        }
                    }
                }
            }
        }
        let throttle = state.current_speed_mult_or_zero().clamp(0.0, 1.0);
        let axis = FinalNormMoveDir(input_dir).normalize_to_axis_dir();
        if axis == IVec2::ZERO {
            if lod_level <= 1 && state.should_adjust_halt_facing_once() {
                if let Ok(facing_dir) = blocking_tiles.cardinal_direction_query().get_mut(pred_ent) {
                    let current_facing = *facing_dir;
                    let facing_gpos = GlobalTilePos(gpos.0 + current_facing.to_dir_vec());
                    if blocking_tiles.is_blocked_at_tiles_only(dim_ref, facing_gpos, pred_ent) {
                        let next_facing = pick_clear_cardinal_dir(
                            &mut rng,
                            &mut blocking_tiles,
                            dim_ref,
                            pred_ent,
                            gpos,
                            Some(current_facing),
                        )
                        .unwrap_or(current_facing);
                        if next_facing != current_facing {
                            if let Ok(mut facing_dir) = blocking_tiles.cardinal_direction_query().get_mut(pred_ent) {
                                *facing_dir = next_facing;
                                facing_messages.push(MirrorHolderStateForSprite(pred_ent));
                            }
                        }
                    }
                }
                state.mark_halt_facing_adjusted();
            }
            messages.push(NavOrder::with_speed_throttle(
                pred_ent,
                10,
                NavOrderSource::Wandering,
                None,
                0.0,
            ));
            continue;
        }

        let mut farthest_target = gpos;
        for dist in 1..=4 {
            let candidate = GlobalTilePos(gpos.0 + axis * dist);
            if blocking_tiles.is_blocked_at_tiles_only(dim_ref, candidate, pred_ent) {
                break;
            }
            farthest_target = candidate;
        }
        if farthest_target == gpos {
            messages.push(NavOrder::with_speed_throttle(
                pred_ent,
                10,
                NavOrderSource::Wandering,
                None,
                0.0,
            ));
            continue;
        }

        messages.push(NavOrder::with_speed_throttle(
            pred_ent,
            10,
            NavOrderSource::Wandering,
            Some(GoTo::new(farthest_target, 0.0)),
            throttle,
        ));
    }
    lod_secs_left_by_ent.retain(|being_ent, _| seen_wanderers.contains(being_ent));
    lod_accum_secs_by_ent.retain(|being_ent, _| seen_wanderers.contains(being_ent));
    if evaluated_wanderers > 0 {
        trace!(
            target: WANDER_SYSTEM,
            "wander_behavior: evaluated_wanderers={} skipped_for_lod={} emitted_nav_orders={}",
            evaluated_wanderers,
            skipped_for_lod,
            messages.len()
        );
    }
    writer.write_batch(messages.drain(..));
    facing_writer.write_batch(facing_messages.drain(..));
}
