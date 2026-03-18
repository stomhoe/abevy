use crate::being_components::Being;
use crate::race::race_components::WanderConfig;
use crate::race::race_resources::RaceRef;
use bevy::{
    ecs::entity::EntityHashMap,
    prelude::*,
};
use movement::movement_components::InputMoveDir;
use param_sets::BlockingTileParamSet;
use rand::Rng;
use std::time::Duration;
use tilemap_shared::{BlacklistedTags, DimensionRef, GlobalTilePos, LoadedChunks};
use ::being_shared::LocalAiControlled;

#[derive(Debug, Clone)]
pub struct WanderState {
    dir: Vec2,
    dir_timer: Timer,
    move_speed: f32,
    halting: bool,
    phase_timer: Timer,
}

impl WanderState {
    fn new(rng: &mut impl Rng, cfg: &WanderConfig) -> Self {
        Self {
            dir: pick_wander_dir(rng),
            dir_timer: Timer::from_seconds(
                rng.random_range(cfg.dir_secs_min..cfg.dir_secs_max),
                TimerMode::Once,
            ),
            move_speed: rng.random_range(cfg.speed_min..cfg.speed_max),
            halting: false,
            phase_timer: Timer::from_seconds(
                rng.random_range(cfg.move_secs_min..cfg.move_secs_max),
                TimerMode::Once,
            ),
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
            rng.random_range(cfg.dir_secs_min..cfg.dir_secs_max),
            TimerMode::Once,
        );
    }

    state.phase_timer.tick(Duration::from_secs_f32(dt));
    if state.phase_timer.just_finished() {
        state.halting = !state.halting;
        if state.halting {
            state.phase_timer = Timer::from_seconds(
                rng.random_range(cfg.halt_secs_min..cfg.halt_secs_max),
                TimerMode::Once,
            );
        } else {
            state.phase_timer = Timer::from_seconds(
                rng.random_range(cfg.move_secs_min..cfg.move_secs_max),
                TimerMode::Once,
            );
            state.move_speed = rng.random_range(cfg.speed_min..cfg.speed_max);
        }
    }

    if state.halting {
        Vec2::ZERO
    } else {
        state.dir * state.move_speed
    }
}

pub fn wander_behavior(
    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
    mut blocking_tiles: BlockingTileParamSet,
    mut beings: Query<
        (
            Entity,
            &GlobalTilePos,
            &DimensionRef,
            Option<&RaceRef>,
        ),
        (With<Being>, Without<crate::being_components::Chaser>, LocalAiControlled),
    >,
    race_wander_cfg_query: Query<&WanderConfig>,
    mut wander_states: Local<EntityHashMap<WanderState>>,
    mut input_dirs: Query<&mut InputMoveDir>,
) {
    let mut rng = rand::rng();
    let dt = time.delta_secs();
    let default_cfg = WanderConfig {
        dir_secs_min: 0.8,
        dir_secs_max: 2.4,
        move_secs_min: 0.9,
        move_secs_max: 2.4,
        halt_secs_min: 0.25,
        halt_secs_max: 1.4,
        speed_min: 0.2,
        speed_max: 0.6,
        avoid_tile_tags: BlacklistedTags::default(),
    };
    for (pred_ent, gpos, &dim_ref, race_ref) in beings.iter_mut() {
        let Ok(mut input_move_dir) = input_dirs.get_mut(pred_ent) else {
            continue;
        };
        let cfg = race_ref
            .and_then(|r| race_wander_cfg_query.get(r.0).ok())
            .unwrap_or(&default_cfg);
        let state = wander_states
            .entry(pred_ent)
            .or_insert_with(|| WanderState::new(&mut rng, cfg));
        let mut input_dir = apply_wander_input(state, dt, &mut rng, cfg);

        // Check if currently in an undesirable tile, and if so, scan for nearby desirable tile
        if !cfg.avoid_tile_tags.is_empty() {
            if blocking_tiles.has_tagset_at(dim_ref, *gpos, &cfg.avoid_tile_tags.0) {
                if let Some(target_pos) = blocking_tiles.find_closest_allowed_gpos_across_loaded_chunks(
                    &loaded_chunks,
                    dim_ref,
                    *gpos,
                    pred_ent,
                    &cfg.avoid_tile_tags,
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
                if blocking_tiles.has_tagset_at(dim_ref, next, &cfg.avoid_tile_tags.0) {
                    input_dir = Vec2::ZERO;
                }
            }
        }
        input_move_dir.0 = input_dir;
    }
}
