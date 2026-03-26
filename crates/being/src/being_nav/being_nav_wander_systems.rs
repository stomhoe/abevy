use crate::being_inst_template::being_inst_template_resources::BitRef;

use crate::race::race_resources::RaceRef;
use crate::pack::pack_components::PackCenterPos;
use crate::being_messages::NavOrder;
use ::being_shared::*;
use bevy::{
    ecs::entity::EntityHashMap,
    platform::collections::HashSet,
    prelude::*,
};
use common::common_tag_components::TagSet;
use movement::movement_components::FinalNormMoveDir;
use param_sets::BlockingTileParamSet;
use rand::Rng;
use std::time::Duration;
use ::tilemap_shared::*;

#[derive(Debug, Clone)]
pub struct WanderState {
    dir: Vec2,
    dir_timer: Timer,
    move_speed: f32,
    halting: bool,
    phase_timer: Timer,
    pack_orbit_timer: Timer,
    pack_orbit_target: Option<GlobalTilePos>,
}

impl WanderState {
    fn new(rng: &mut impl Rng, cfg: &WanderConfig) -> Self {
        Self {
            dir: pick_wander_dir(rng),
            dir_timer: Timer::from_seconds(
                sample_seconds(rng, cfg.dir_secs_min, cfg.dir_secs_max),
                TimerMode::Once,
            ),
            move_speed: sample_seconds(rng, cfg.speed_min, cfg.speed_max),
            halting: false,
            phase_timer: Timer::from_seconds(
                sample_seconds(rng, cfg.move_secs_min, cfg.move_secs_max),
                TimerMode::Once,
            ),
            pack_orbit_timer: sample_pack_orbit_timer(rng, cfg),
            pack_orbit_target: None,
        }
    }
}

fn pick_wander_dir(rng: &mut impl Rng) -> Vec2 {
    match rng.random_range(0..=5) {
        0 => Vec2::X,
        1 => -Vec2::X,
        2 => Vec2::Y,
        3 => -Vec2::Y,
        _ => Vec2::ZERO,
    }
}

fn sample_seconds(rng: &mut impl Rng, min: f32, max: f32) -> f32 {
    let min = min.max(0.01);
    let max = max.max(min);
    if max == min {
        min
    } else {
        rng.random_range(min..max)
    }
}

fn apply_wander_input(
    state: &mut WanderState,
    dt: f32,
    rng: &mut impl Rng,
    cfg: &WanderConfig,
) -> Vec2 {
    state.dir_timer.tick(Duration::from_secs_f32(dt));
    if state.dir_timer.just_finished() {
        state.dir = pick_wander_dir(rng);
        state.dir_timer = Timer::from_seconds(
            sample_seconds(rng, cfg.dir_secs_min, cfg.dir_secs_max),
            TimerMode::Once,
        );
    }

    state.phase_timer.tick(Duration::from_secs_f32(dt));
    if state.phase_timer.just_finished() {
        state.halting = !state.halting;
        if state.halting {
            state.phase_timer = Timer::from_seconds(
                sample_seconds(rng, cfg.halt_secs_min, cfg.halt_secs_max),
                TimerMode::Once,
            );
        } else {
            state.phase_timer = Timer::from_seconds(
                sample_seconds(rng, cfg.move_secs_min, cfg.move_secs_max),
                TimerMode::Once,
            );
            state.move_speed = sample_seconds(rng, cfg.speed_min, cfg.speed_max);
        }
    }

    if state.halting {
        Vec2::ZERO
    } else {
        state.dir * state.move_speed
    }
}

fn sample_pack_orbit_timer(rng: &mut impl Rng, cfg: &WanderConfig) -> Timer {
    if cfg.pack_orbit_retarget_secs_max <= 0.0 {
        return Timer::from_seconds(0.0, TimerMode::Once);
    }
    Timer::from_seconds(
        sample_seconds(rng, cfg.pack_orbit_retarget_secs_min, cfg.pack_orbit_retarget_secs_max),
        TimerMode::Once,
    )
}

fn sample_pack_orbit_target(
    rng: &mut impl Rng,
    center: GlobalTilePos,
    cfg: &WanderConfig,
) -> GlobalTilePos {
    let radius = cfg.pack_orbit_radius.round().max(0.0) as i32;
    if radius <= 0 {
        return center;
    }
    for _ in 0..8 {
        let offset = IVec2::new(
            rng.random_range(-radius..=radius),
            rng.random_range(-radius..=radius),
        );
        if offset != IVec2::ZERO {
            return GlobalTilePos(center.0 + offset);
        }
    }
    center
}

fn resolve_wander_cfg(
    member_of: Option<&SquadMemberOf>,
    bit_ref: Option<&BitRef>,
    race_ref: Option<&RaceRef>,
    wander_cfg_query: &Query<&WanderConfig>,
) -> WanderConfig {
    if let Some(member_of) = member_of {
        if let Ok(cfg) = wander_cfg_query.get(member_of.squad) {
            return cfg.clone();
        }
    }
    if let Some(bit_ref) = bit_ref {
        if let Ok(cfg) = wander_cfg_query.get(bit_ref.0) {
            return cfg.clone();
        }
    }
    if let Some(race_ref) = race_ref {
        if let Ok(cfg) = wander_cfg_query.get(race_ref.0) {
            return cfg.clone();
        }
    }
    WanderConfig::default()
}

fn strongest_entity_avoidance(
    cfg: &WanderConfig,
    avoid_tags: &HashSet<String>,
    threat_tags: &TagSet,
    delta: Vec2,
    distance: f32,
    move_speed: f32,
) -> Vec2 {
    let mut strongest: Option<(f32, f32)> = None;
    for tag in threat_tags.iter() {
        let tag_str = tag.to_string();
        if !avoid_tags.contains(&tag_str) {
            continue;
        }
        let radius = cfg.avoid_entity_radius_for(&tag_str);
        let strength = cfg.avoid_entity_strength_for(&tag_str);
        let Some((best_radius, best_strength)) = strongest else {
            strongest = Some((radius, strength));
            continue;
        };
        if strength > best_strength || (strength == best_strength && radius > best_radius) {
            strongest = Some((radius, strength));
        }
    }

    let Some((radius, strength)) = strongest else {
        return Vec2::ZERO;
    };
    if radius <= 0.0 || distance <= 0.0 || distance > radius {
        return Vec2::ZERO;
    }
    let pull = ((radius - distance) / radius).clamp(0.0, 1.0);
    delta.normalize_or_zero() * (move_speed * strength * pull * pull)
}

fn collect_entity_avoidance(
    blocking_tiles: &mut BlockingTileParamSet,
    self_ent: Entity,
    self_dim: &DimensionRef,
    cfg: &WanderConfig,
    move_speed: f32,
    threat_query: &Query<
        (
            Entity,
            &DimensionRef,
            Option<&SquadMemberOf>,
        ),
        (With<Being>, ),
    >,
    tag_query: &Query<&TagSet>,
) -> Vec2 {
    if cfg.avoid_race_tags.is_empty() && cfg.avoid_bit_tags.is_empty() && cfg.avoid_pack_tags.is_empty() {
        return Vec2::ZERO;
    }

    let max_radius = cfg.max_avoid_entity_radius();
    if max_radius <= 0.0 {
        return Vec2::ZERO;
    }

    let mut avoidance = Vec2::ZERO;
    let Ok(&self_pos) = blocking_tiles.gpos_query.get(self_ent) else {
        return Vec2::ZERO;
    };
    for (threat_ent, &threat_dim, member_of) in threat_query.iter() {
        if threat_ent == self_ent || threat_dim != *self_dim {
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
            if let Ok(bit_tags) = tag_query.get(bit_ref.0) {
                avoidance += strongest_entity_avoidance(
                    cfg,
                    &cfg.avoid_bit_tags,
                    bit_tags,
                    delta_vec,
                    distance,
                    move_speed,
                );
            }
        }
        if let Some(race_ref) = blocking_tiles.get_being_race_ref(threat_ent) {
            if let Ok(race_tags) = tag_query.get(race_ref.0) {
                avoidance += strongest_entity_avoidance(
                    cfg,
                    &cfg.avoid_race_tags,
                    race_tags,
                    delta_vec,
                    distance,
                    move_speed,
                );
            }
        }
        if let Some(member_of) = member_of {
            let Ok(pack_tags) = tag_query.get(member_of.squad) else {
                continue;
            };
            avoidance += strongest_entity_avoidance(
                cfg,
                &cfg.avoid_pack_tags,
                pack_tags,
                delta_vec,
                distance,
                move_speed,
            );
        }
    }

    let max_avoidance = move_speed * 1.5;
    if avoidance.length_squared() > max_avoidance * max_avoidance {
        avoidance = avoidance.normalize_or_zero() * max_avoidance;
    }
    avoidance
}

#[allow(unused_parens, )]
pub fn wander_behavior(
    mut writer: MessageWriter<NavOrder>,
    time: Res<Time>,
    mut blocking_tiles: BlockingTileParamSet,
    mut beings: Query<
        (
            Entity,
            &DimensionRef,
            Option<&SquadMemberOf>,
        ),
        (With<Being>, With<Wandering>, Without<Fleeing>, LocalAiControlled),
    >,
    wander_cfg_query: Query<&WanderConfig>,
    threat_query: Query<
        (
            Entity,
            &DimensionRef,
            Option<&SquadMemberOf>,
        ),
        (With<Being>, ),
    >,
    tag_query: Query<&TagSet>,
    pack_center_query: Query<&PackCenterPos>,
    mut wander_states: Local<EntityHashMap<WanderState>>,
    mut messages: Local<Vec<NavOrder>>,
) {
    let mut rng = rand::rng();
    let dt = time.delta_secs();
    for (pred_ent, &dim_ref, member_of, ) in beings.iter_mut() {
        let Ok(&gpos) = blocking_tiles.gpos_query.get(pred_ent) else {
            continue;
        };
        let bit_ref = blocking_tiles.get_being_bit_ref(pred_ent);
        let race_ref = blocking_tiles.get_being_race_ref(pred_ent);
        let cfg = resolve_wander_cfg(member_of, bit_ref, race_ref, &wander_cfg_query);
        let avoid_tile_tags = BlacklistedTags::new(&cfg.avoid_tile_tags);
        let state = wander_states
            .entry(pred_ent)
            .or_insert_with(|| WanderState::new(&mut rng, &cfg));
        let mut input_dir = apply_wander_input(state, dt, &mut rng, &cfg);

        if let Some(member_of) = member_of {
            if let Some(pack_center) = pack_center_query
                .get(member_of.squad)
                .ok()
                .and_then(|pack_center| pack_center.0.get(&dim_ref).copied())
            {
                let pack_distance = (pack_center.0 - gpos.0).as_vec2().length();
                let pack_delta = pack_center.0 - gpos.0;
                if cfg.wander_around_leader && cfg.pack_orbit_radius > 0.0 {
                    state.pack_orbit_timer.tick(Duration::from_secs_f32(dt));
                    let retarget = state.pack_orbit_target.is_none() || state.pack_orbit_timer.just_finished();
                    if retarget {
                        state.pack_orbit_target = Some(sample_pack_orbit_target(&mut rng, pack_center, &cfg));
                        state.pack_orbit_timer = sample_pack_orbit_timer(&mut rng, &cfg);
                    }
                    if let Some(orbit_target) = state.pack_orbit_target {
                        let orbit_delta = orbit_target.0 - gpos.0;
                        let orbit_distance = orbit_delta.as_vec2().length();
                        if orbit_distance > 0.5 {
                            let pull = ((orbit_distance - cfg.pack_orbit_radius).max(0.0)
                                / cfg.pack_orbit_radius.max(1.0))
                                .clamp(0.0, 1.0);
                            input_dir += orbit_delta.as_vec2().normalize_or_zero() * (state.move_speed * 0.75 * pull);
                        }
                    }
                }
                if cfg.max_drift.is_finite() && pack_distance > cfg.max_drift {
                    let pull = ((pack_distance - cfg.max_drift).max(0.0) / cfg.max_drift.max(1.0))
                        .clamp(0.0, 1.0);
                    input_dir += pack_delta.as_vec2().normalize_or_zero() * (state.move_speed * 1.5 * pull);
                }
            }
        }

        input_dir += collect_entity_avoidance(
            &mut blocking_tiles,
            pred_ent,
            &dim_ref,
            &cfg,
            state.move_speed,
            &threat_query,
            &tag_query,
        );

        // Check if currently in an undesirable tile, and if so, scan for nearby desirable tile
        if !avoid_tile_tags.is_empty() {
            if !blocking_tiles.allowed_at(
                dim_ref,
                gpos,
                pred_ent,
                &WhitelistedSpawnTileTags::default(),
                &BlacklistedSpawnTileTags(avoid_tile_tags.clone()),
            ) {
                if let Some(target_pos) = blocking_tiles.find_closest_allowe_gpos(
                    dim_ref,
                    gpos,
                    pred_ent,
                    &WhitelistedTags::default(),
                    &avoid_tile_tags,
                ) {
                    let delta = target_pos.0 - gpos.0;
                    input_dir = delta.as_vec2().normalize_or_zero() * (state.move_speed * 2.0);
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
                if !blocking_tiles.allowed_at(
                    dim_ref,
                    next,
                    pred_ent,
                    &WhitelistedSpawnTileTags::default(),
                    &BlacklistedSpawnTileTags(avoid_tile_tags.clone()),
                ) {
                    input_dir = Vec2::ZERO;
                }
            }
        }
        let throttle = state.move_speed.clamp(0.0, 1.0);
        let axis = FinalNormMoveDir(input_dir).normalize_to_axis_dir();
        if axis == IVec2::ZERO {
            messages.push(NavOrder::new(
                pred_ent,
                10,
                NavOrderSource::Wandering,
                None,
                Some(0.0),
                None,
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
            messages.push(NavOrder::new(
                pred_ent,
                10,
                NavOrderSource::Wandering,
                None,
                Some(0.0),
                None,
            ));
            continue;
        }

        messages.push(NavOrder::new(
            pred_ent,
            10,
            NavOrderSource::Wandering,
            Some(GoTo::new(farthest_target, 0.0)),
            Some(throttle),
            None,
        ));
    }
    writer.write_batch(messages.drain(..));
}
