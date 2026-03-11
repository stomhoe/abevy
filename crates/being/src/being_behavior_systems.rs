use crate::being_components::{Being, ToChase};
use crate::being_inst_template::being_inst_template_resources::BitRef;
use crate::body::{Bodies, BodySums};
use crate::race::race_components::Race;
use crate::race::race_components::WanderConfig;
use crate::race::race_resources::RaceRef;
use ::being_shared::*;
use ::being_shared::{ControlledBy, Hunger, Predator, PredatorHuntThreshold};
use ::tilemap_shared::{ChunkPos, GlobalTilePos, LoadedChunks};
use ac_input::ac_input_actions::{BeingInputContext, BeingMoveAction};
#[allow(unused_imports)]
use bevy::{
    ecs::entity::EntityHashMap,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_enhanced_input::action::mock::MockEntityCommandsExt;
use bevy_enhanced_input::prelude::*;
use bevy_northstar::prelude::*;
use common::common_components::StrId;
use common::common_tag_components::TagSet;
use param_sets::BlockingTileParamSet;
use rand::Rng;
use std::time::Duration;
use tilemap::chunking::chunking_resources::AaChunkRangeSettings;

#[derive(Resource)]
pub struct AiNavGrids {
    pub by_dim: HashMap<Entity, AiNavGridCache>,
    pub center_by_dim: HashMap<Entity, IVec2>,
    pub rebuild_timer: Timer,
}
impl Default for AiNavGrids {
    fn default() -> Self {
        Self {
            by_dim: HashMap::default(),
            center_by_dim: HashMap::default(),
            rebuild_timer: Timer::from_seconds(0.35, TimerMode::Repeating),
        }
    }
}

pub struct AiNavGridCache {
    pub min: IVec2,
    pub grid: CardinalGrid,
    pub occupied: HashMap<UVec3, Entity>,
}

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

fn set_ai_movement_action(commands: &mut Commands, being_ent: Entity, input: Vec2) {
    let state = if input == Vec2::ZERO {
        TriggerState::None
    } else {
        TriggerState::Fired
    };
    commands
        .entity(being_ent)
        .try_mock::<BeingInputContext, BeingMoveAction>(state, input, MockSpan::once());
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
    beings: Query<
        (Entity, Option<&BitRef>, Option<&RaceRef>),
        (With<Being>, Or<(Changed<BitRef>, Changed<RaceRef>)>),
    >,
    bit_pred_cfg: Query<&Predator>,
    race_pred_cfg: Query<&Predator>,
    bit_cfg: Query<&PredatorHuntThreshold>,
    race_cfg: Query<&PredatorHuntThreshold>,
) {
    for (being_ent, bit_ref, race_ref) in beings.iter() {
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

pub fn tick_hunger(time: Res<Time>, mut query: Query<&mut Hunger>) {
    let delta = time.delta_secs();
    if delta <= 0.0 {
        return;
    }
    for mut hunger in query.iter_mut() {
        hunger.curr = (hunger.curr + hunger.increase_per_sec * delta).clamp(0.0, hunger.max);
    }
}

pub fn sync_ai_nav_grids(
    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_range: Res<AaChunkRangeSettings>,
    param_set: BlockingTileParamSet,
    chasers_query: Query<
        (
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&ControlledBy>,
            &ToChase,
        ),
        With<Being>,
    >,
    beings_query: Query<(Entity, &GlobalTilePos, &::tilemap_shared::DimensionRef), With<Being>>,
    mut grids: ResMut<AiNavGrids>,
    mut to_drain: Local<Vec<Entity>>,
) {
    let mut needed_dims: HashSet<Entity> = HashSet::default();
    let mut dim_centers: HashMap<Entity, IVec2> = HashMap::default();
    let mut dim_center_counts: HashMap<Entity, i32> = HashMap::default();

    for (gpos, dim_ref, controlled_by, _to_chase) in chasers_query.iter() {
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_input {
                continue;
            }
        }
        needed_dims.insert(dim_ref.0);
        let pos = gpos.0;
        let center = dim_centers.entry(dim_ref.0).or_insert(IVec2::ZERO);
        *center += pos;
        *dim_center_counts.entry(dim_ref.0).or_insert(0) += 1;
    }

    grids.by_dim.retain(|dim, _| needed_dims.contains(dim));
    grids
        .center_by_dim
        .retain(|dim, _| needed_dims.contains(dim));

    let max_side = (((chunk_range.discovery_range as i32 * 2) - 1).max(1) as u32)
        * ChunkPos::CHUNK_SIZE.x.max(1);
    let should_rebuild = grids.rebuild_timer.tick(time.delta()).just_finished();

    for dim in needed_dims.iter().copied() {
        let mut min_tile: Option<IVec2> = None;
        let mut max_tile: Option<IVec2> = None;

        for (&(dim_ref, chunk_pos), _) in loaded_chunks.0.iter() {
            if dim_ref.0 != dim {
                continue;
            }
            let cmin = chunk_pos.to_tilepos().0;
            let cmax = cmin + ChunkPos::CHUNK_SIZE.as_ivec2() - IVec2::ONE;
            min_tile = Some(min_tile.map(|m| m.min(cmin)).unwrap_or(cmin));
            max_tile = Some(max_tile.map(|m| m.max(cmax)).unwrap_or(cmax));
        }

        let Some(mut min_tile) = min_tile else {
            continue;
        };
        let Some(mut max_tile) = max_tile else {
            continue;
        };

        let center = dim_centers
            .get(&dim)
            .zip(dim_center_counts.get(&dim))
            .map(|(sum, count)| *sum / count.max(&1))
            .unwrap_or((min_tile + max_tile) / 2);

        let mut width = (max_tile.x - min_tile.x + 1).max(3) as u32;
        let mut height = (max_tile.y - min_tile.y + 1).max(3) as u32;
        if width > max_side {
            let half = (max_side as i32) / 2;
            min_tile.x = center.x - half;
            max_tile.x = min_tile.x + max_side as i32 - 1;
            width = max_side;
        }
        if height > max_side {
            let half = (max_side as i32) / 2;
            min_tile.y = center.y - half;
            max_tile.y = min_tile.y + max_side as i32 - 1;
            height = max_side;
        }

        let center_changed = grids
            .center_by_dim
            .get(&dim)
            .map(|prev| (*prev - center).abs().max_element() >= ChunkPos::CHUNK_SIZE.x as i32)
            .unwrap_or(true);
        let needs_new_grid = !grids.by_dim.contains_key(&dim);
        let rebuild_grid = needs_new_grid || should_rebuild || center_changed;

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
                        &mut to_drain,
                        ::tilemap_shared::DimensionRef(dim),
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
                },
            );
            grids.center_by_dim.insert(dim, center);
        }

        let Some(cache) = grids.by_dim.get_mut(&dim) else {
            continue;
        };
        cache.occupied.clear();
        for (being_ent, gpos, dim_ref) in beings_query.iter() {
            if dim_ref.0 != dim {
                continue;
            }
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
            Option<&ControlledBy>,
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
            if controlled_by.human_input {
                cmd.entity(pred_ent).try_remove::<ToChase>();
                continue;
            }
        }

        let hp = health_ratio(pred_ent, &bodies_query, &body_health_query);
        if hunger.curr < hunt_threshold.0 || hp <= 0.9 {
            cmd.entity(pred_ent).try_remove::<ToChase>();
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
            cmd.entity(pred_ent).try_remove::<ToChase>();
            continue;
        };
        cmd.entity(pred_ent).try_insert(ToChase::new(target, 1.5));
    }
}

pub fn chase_behavior(
    mut commands: Commands,
    mut chasers: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            &ToChase,
        ),
        (With<Being>, LocalAiControlled),
    >,
    beings_query: Query<(Entity, &GlobalTilePos, &::tilemap_shared::DimensionRef), With<Being>>,
    grids: Res<AiNavGrids>,
    mut dynamic_blocking: Local<HashMap<UVec3, Entity>>,
) {
    for (chaser_ent, chaser_gpos, &chaser_dim, to_chase) in chasers.iter_mut() {
        if to_chase.target == chaser_ent {
            set_ai_movement_action(&mut commands, chaser_ent, Vec2::ZERO);
            continue;
        }
        let Ok((_target_ent, target_gpos, &target_dim)) = beings_query.get(to_chase.target) else {
            set_ai_movement_action(&mut commands, chaser_ent, Vec2::ZERO);
            continue;
        };
        if target_dim != chaser_dim {
            set_ai_movement_action(&mut commands, chaser_ent, Vec2::ZERO);
            continue;
        }

        let chaser_pos = *chaser_gpos;
        let target_pos = *target_gpos;

        let stop_threshold = to_chase.stop_distance.max(0.0);
        if chaser_pos.0.as_vec2().distance(target_pos.0.as_vec2()) <= stop_threshold {
            set_ai_movement_action(&mut commands, chaser_ent, Vec2::ZERO);
            continue;
        }

        let desired_to_prey = (target_pos.0 - chaser_pos.0).as_vec2();
        if desired_to_prey == Vec2::ZERO {
            set_ai_movement_action(&mut commands, chaser_ent, Vec2::ZERO);
            continue;
        }
        let direct_chase_dir = desired_to_prey.normalize();

        let Some(cache) = grids.by_dim.get(&chaser_dim.0) else {
            set_ai_movement_action(&mut commands, chaser_ent, direct_chase_dir);
            continue;
        };

        let start_i = chaser_pos.0 - cache.min;
        let goal_i = target_pos.0 - cache.min;
        if start_i.x < 0 || start_i.y < 0 || goal_i.x < 0 || goal_i.y < 0 {
            set_ai_movement_action(&mut commands, chaser_ent, direct_chase_dir);
            continue;
        }
        let start = UVec3::new(start_i.x as u32, start_i.y as u32, 0);
        let goal = UVec3::new(goal_i.x as u32, goal_i.y as u32, 0);
        if start.x >= cache.grid.width()
            || start.y >= cache.grid.height()
            || goal.x >= cache.grid.width()
            || goal.y >= cache.grid.height()
        {
            set_ai_movement_action(&mut commands, chaser_ent, direct_chase_dir);
            continue;
        }

        dynamic_blocking.clear();
        dynamic_blocking.reserve(cache.occupied.len());
        for (&pos, &ent) in cache.occupied.iter() {
            if ent == chaser_ent || ent == to_chase.target {
                continue;
            }
            dynamic_blocking.insert(pos, ent);
        }
        dynamic_blocking.remove(&goal);
        dynamic_blocking.remove(&start);

        let mut req = PathfindArgs::new(start, goal).blocking(&dynamic_blocking);
        let Some(path) = cache.grid.pathfind(&mut req) else {
            set_ai_movement_action(&mut commands, chaser_ent, direct_chase_dir);
            continue;
        };

        let steps = path.path();
        let Some(next) = steps.first() else {
            set_ai_movement_action(&mut commands, chaser_ent, direct_chase_dir);
            continue;
        };
        let next = next.xy().as_ivec2() + cache.min;
        let desired = (next - chaser_pos.0).as_vec2();
        let move_input = if desired == Vec2::ZERO {
            direct_chase_dir
        } else {
            desired.normalize()
        };
        set_ai_movement_action(&mut commands, chaser_ent, move_input);
    }
}

pub fn wander_behavior(
    time: Res<Time>,
    blocking_tiles: BlockingTileParamSet,
    mut beings: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&RaceRef>,
        ),
        (With<Being>, Without<ToChase>, LocalAiControlled),
    >,
    race_wander_cfg_query: Query<&WanderConfig>,
    mut wander_states: Local<HashMap<Entity, WanderState>>,
    mut to_drain: Local<Vec<Entity>>,
    mut commands: Commands,
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
            if blocking_tiles.has_tagset_at(&mut to_drain, dim_ref, next, &cfg.avoid_tile_tags) {
                input_dir = Vec2::ZERO;
            }
        }
        set_ai_movement_action(&mut commands, pred_ent, input_dir);
    }
}
