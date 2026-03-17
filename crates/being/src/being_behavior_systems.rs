use crate::being_inst_template::being_inst_template_resources::BitRef;
use crate::body::{Bodies, BodySums};
use crate::being_components::{Being, Chaser};
use crate::race::race_components::Race;
use crate::race::race_components::WanderConfig;
use crate::race::race_resources::RaceRef;
use ::being_shared::*;
use ::being_shared::{ComputedBy, Hunger, Predator, PredatorHuntThreshold};
#[allow(unused_imports)]
use bevy::{
    ecs::{entity::EntityHashMap, entity::EntityHashSet, entity_disabling::Disabled},
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use common::AnyDisabling;
use common::common_components::StrId;
use common::common_tag_components::TagSet;
use movement::movement_components::InputMoveDir;
use param_sets::BlockingTileParamSet;
use rand::Rng;
use std::time::Duration;
use tilemap_shared::GlobalTilePos;

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

pub fn add_predator_behavior_components(
    mut commands: Commands,
    query: Query<Entity, (With<Predator>, Without<Hunger>)>,
) {
    for being_ent in query.iter() {
        commands
            .entity(being_ent)
            .try_insert((Hunger::default(), PredatorHuntThreshold::default()));
    }
}

pub fn sync_predator_config_from_sources(
    mut commands: Commands,
    changed_beings: Query<Entity, (With<Being>, Or<(Changed<BitRef>, Changed<RaceRef>)>)>,
    beings: Query<
        (Option<&BitRef>, Option<&RaceRef>),
        (With<Being>, AnyDisabling),
    >,
    bit_pred_cfg: Query<&Predator>,
    race_pred_cfg: Query<&Predator>,
    bit_cfg: Query<&PredatorHuntThreshold>,
    race_cfg: Query<&PredatorHuntThreshold>,
    mut removed_disabled: RemovedComponents<Disabled>,
) {
    let reenabled_beings = collect_reenabled_entities(&mut removed_disabled);
    let mut beings_to_sync = reenabled_beings;
    beings_to_sync.extend(changed_beings.iter());

    for being_ent in beings_to_sync {
        let Ok((bit_ref, race_ref)) = beings.get(being_ent) else {
            continue;
        };
        let bit_predator = bit_ref.and_then(|r| bit_pred_cfg.get(r.0).ok()).cloned();
        let race_predator = race_ref.and_then(|r| race_pred_cfg.get(r.0).ok()).cloned();
        let bit_threshold = bit_ref.and_then(|r| bit_cfg.get(r.0).ok()).copied();
        let race_threshold = race_ref.and_then(|r| race_cfg.get(r.0).ok()).copied();

        let Some(chosen) = bit_threshold.or(race_threshold) else {
            continue;
        };
        let predator = bit_predator.or(race_predator).unwrap_or_default();
        commands.entity(being_ent).try_insert((predator, chosen));
    }
}

fn collect_reenabled_entities(removed_disabled: &mut RemovedComponents<Disabled>) -> EntityHashSet {
    let mut entities = EntityHashSet::default();
    entities.extend(removed_disabled.read());
    entities
}

pub fn tick_hunger(time: Res<Time>, mut query: Query<&mut Hunger>) {
    let delta = time.delta_secs();
    if delta <= 0.0 {
        return;
    }
    for mut hunger in query.iter_mut() {
        hunger.curr = (hunger.curr + hunger.increase_per_sec * delta).clamp(0.0, hunger.max);
    }
}

fn health_ratio(
    being: Entity,
    bodies_query: &Query<&Bodies, With<Being>>,
    body_health_query: &Query<&BodySums>,
) -> f32 {
    let Ok(bodies) = bodies_query.get(being) else {
        return 0.0;
    };
    let Some(body_ent) = bodies.entities().first() else {
        return 0.0;
    };
    let Ok(health) = body_health_query.get(*body_ent) else {
        return 0.0;
    };
    if health.total_hp <= 0.0 {
        return 0.0;
    }
    (health.current_hp / health.total_hp).clamp(0.0, 1.0)
}

pub fn update_predator_chase_targets(
    bodies_query: Query<&Bodies, With<Being>>,
    body_health_query: Query<&BodySums>,
    predators: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            &Predator,
            Option<&RaceRef>,
            Option<&BodyTreeWeightSum>,
            Option<&ComputedBy>,
            &Hunger,
            &PredatorHuntThreshold,
        ),
        With<Predator>,
    >,
    prey_query: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&RaceRef>,
            Option<&BodyTreeWeightSum>,
            Option<&TagSet>,
        ),
        With<Being>,
    >,
    race_str_id_query: Query<&StrId, With<Race>>,
    mut cmd: Commands,
) {
    for (
        pred_ent,
        pred_gpos,
        &pred_dim,
        predator_cfg,
        pred_race,
        pred_weight_sum,
        controlled_by,
        hunger,
        hunt_threshold,
    ) in predators.iter()
    {
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_dc_input {
                cmd.entity(pred_ent).try_remove::<Chaser>();
                continue;
            }
        }

        let hp = health_ratio(pred_ent, &bodies_query, &body_health_query);
        if hunger.curr < hunt_threshold.0 || hp <= 0.9 {
            cmd.entity(pred_ent).try_remove::<Chaser>();
            continue;
        }

        let pred_pos = *pred_gpos;
        let pred_weight_newtons = pred_weight_sum.map(|sum| sum.0).unwrap_or_default();
        let mut closest: Option<(Entity, i32)> = None;
        for (prey_ent, prey_gpos, &prey_dim, prey_race, prey_weight_sum, prey_tags) in
            prey_query.iter()
        {
            if prey_ent == pred_ent || prey_dim != pred_dim {
                continue;
            }
            if let Some(prey_tags) = prey_tags {
                if predator_cfg.do_not_hunt_tags.intersects(prey_tags) {
                    continue;
                }
            }
            let prey_weight_newtons = prey_weight_sum.map(|sum| sum.0).unwrap_or_default();
            if predator_cfg.prey_body_size_ratio_tolerance > 0.0
                && pred_weight_newtons > 0.0
                && prey_weight_newtons
                    > pred_weight_newtons * predator_cfg.prey_body_size_ratio_tolerance
            {
                continue;
            }
            if let Some(prey_race) = prey_race {
                if let Ok(prey_race_id) = race_str_id_query.get(prey_race.0) {
                    if predator_cfg.own_races.contains(prey_race_id) {
                        continue;
                    }
                }
            }
            if let (Some(pred_race), Some(prey_race)) = (pred_race, prey_race) {
                if pred_race.0 == prey_race.0 {
                    continue;
                }
            }
            let prey_pos = *prey_gpos;
            let delta = prey_pos.0 - pred_pos.0;
            let manhattan = delta.x.abs() + delta.y.abs();
            let Some((_, curr_best)) = closest else {
                closest = Some((prey_ent, manhattan));
                continue;
            };
            if manhattan < curr_best {
                closest = Some((prey_ent, manhattan));
            }
        }

        let Some((target, _)) = closest else {
            cmd.entity(pred_ent).try_remove::<Chaser>();
            continue;
        };
        cmd.entity(pred_ent).try_insert(Chaser::new(target, 1.5));
    }
}

pub fn wander_behavior(
    time: Res<Time>,
    mut blocking_tiles: BlockingTileParamSet,
    mut beings: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&RaceRef>,
        ),
        (With<Being>, Without<Chaser>, LocalAiControlled),
    >,
    race_wander_cfg_query: Query<&WanderConfig>,
    mut wander_states: Local<HashMap<Entity, WanderState>>,
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
        avoid_tile_tags: TagSet::default(),
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

        if !cfg.avoid_tile_tags.is_empty() && input_dir != Vec2::ZERO {
            let step = if input_dir.x.abs() >= input_dir.y.abs() {
                IVec2::new(input_dir.x.signum() as i32, 0)
            } else {
                IVec2::new(0, input_dir.y.signum() as i32)
            };
            let next = GlobalTilePos(gpos.0 + step);
            if blocking_tiles.has_tagset_at(dim_ref, next, &cfg.avoid_tile_tags) {
                input_dir = Vec2::ZERO;
            }
        }
        input_move_dir.0 = input_dir;
    }
}
