#[allow(unused_imports, )]
use bevy::{ecs::entity::EntityHashMap, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_northstar::prelude::*;
use common::log_targets;
use param_sets::BlockingTileParamSet;
use rand::Rng;
use std::time::Duration;
use tilemap::{chunking::chunking_resources::AaChunkRangeSettings};
use ::tilemap_shared::{GlobalTilePos, ChunkPos, LoadedChunks};
use movement::movement_components::InputDirection;
use crate::being_components::Being;
use crate::being_inst_template::being_inst_template_resources::BitRef;
use crate::body::{BodyHealth, Bodies};
use crate::race::race_resources::RaceRef;
use ::being_shared::{Predator, ControlledBy, Hunger, PredatorHuntThreshold};

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
pub(crate) struct WanderState {
    dir: Vec2,
    dir_timer: Timer,
    throttle: f32,
    throttle_timer: Timer,
    next_burst_timer: Timer,
    burst_timer: Timer,
    pulse_timer: Timer,
    pulse_move: bool,
}

impl WanderState {
    fn new(rng: &mut impl Rng) -> Self {
        Self {
            dir: pick_wander_dir(rng),
            dir_timer: Timer::from_seconds(rng.random_range(0.6..2.2), TimerMode::Once),
            throttle: rng.random_range(0.0..1.0),
            throttle_timer: Timer::from_seconds(rng.random_range(0.35..1.2), TimerMode::Once),
            next_burst_timer: Timer::from_seconds(rng.random_range(2.5..7.0), TimerMode::Once),
            burst_timer: Timer::from_seconds(0.01, TimerMode::Once),
            pulse_timer: Timer::from_seconds(0.11, TimerMode::Repeating),
            pulse_move: true,
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
    input_dir: &mut Mut<InputDirection>,
    dt: f32,
    rng: &mut impl Rng,
) {
    state.dir_timer.tick(Duration::from_secs_f32(dt));
    if state.dir_timer.just_finished() {
        state.dir = pick_wander_dir(rng);
        state.dir_timer = Timer::from_seconds(rng.random_range(0.5..2.0), TimerMode::Once);
    }

    state.throttle_timer.tick(Duration::from_secs_f32(dt));
    if state.throttle_timer.just_finished() {
        state.throttle = rng.random_range(0.0..1.0);
        state.throttle_timer = Timer::from_seconds(rng.random_range(0.3..1.0), TimerMode::Once);
    }

    state.next_burst_timer.tick(Duration::from_secs_f32(dt));
    if state.next_burst_timer.just_finished() {
        state.burst_timer = Timer::from_seconds(rng.random_range(0.25..0.7), TimerMode::Once);
        state.next_burst_timer = Timer::from_seconds(rng.random_range(2.8..8.0), TimerMode::Once);
    }

    if !state.burst_timer.is_finished() {
        state.burst_timer.tick(Duration::from_secs_f32(dt));
    }

    state.pulse_timer.tick(Duration::from_secs_f32(dt));
    if state.pulse_timer.just_finished() {
        let speed_factor = if state.burst_timer.is_finished() { state.throttle } else { 1.0 };
        state.pulse_move = rng.random::<f32>() <= speed_factor;
    }

    input_dir.0 = if state.pulse_move { state.dir } else { Vec2::ZERO };
}

pub fn add_predator_behavior_components(
    mut commands: Commands,
    query: Query<Entity, (With<Predator>, Without<Hunger>)>,
) {
    for being_ent in query.iter() {
        commands.entity(being_ent).try_insert((Hunger::default(), PredatorHuntThreshold::default()));
    }
}

pub fn sync_predator_config_from_sources(
    mut commands: Commands,
    beings: Query<(Entity, Option<&BitRef>, Option<&RaceRef>), (With<Being>, Or<(Changed<BitRef>, Changed<RaceRef>)>)>,
    bit_cfg: Query<&PredatorHuntThreshold>,
    race_cfg: Query<&PredatorHuntThreshold>,
) {
    for (being_ent, bit_ref, race_ref) in beings.iter() {
        let bit_threshold = bit_ref
            .and_then(|r| bit_cfg.get(r.0).ok())
            .copied();
        let race_threshold = race_ref
            .and_then(|r| race_cfg.get(r.0).ok())
            .copied();

        let Some(chosen) = bit_threshold.or(race_threshold) else {
            continue;
        };
        commands.entity(being_ent).try_insert((Predator, chosen));
    }
}

pub fn tick_hunger(
    time: Res<Time>,
    mut query: Query<&mut Hunger>,
) {
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
    predator_query: Query<
        (
            &Transform,
            &::tilemap_shared::DimensionRef,
            Option<&ControlledBy>,
            &Hunger,
            &PredatorHuntThreshold,
        ),
        With<Predator>,
    >,
    beings_query: Query<(Entity, &Transform, &::tilemap_shared::DimensionRef), With<Being>>,
    mut grids: ResMut<AiNavGrids>,
    mut to_drain: Local<Vec<Entity>>,
) {
    let mut needed_dims: HashSet<Entity> = HashSet::default();
    let mut dim_centers: HashMap<Entity, IVec2> = HashMap::default();
    let mut dim_center_counts: HashMap<Entity, i32> = HashMap::default();

    for (transform, dim_ref, controlled_by, hunger, threshold) in predator_query.iter() {
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_input {
                continue;
            }
        }
        if hunger.curr < threshold.0 {
            continue;
        }
        needed_dims.insert(dim_ref.0);
        let pos = GlobalTilePos::from(transform.translation.xy()).0;
        let center = dim_centers.entry(dim_ref.0).or_insert(IVec2::ZERO);
        *center += pos;
        *dim_center_counts.entry(dim_ref.0).or_insert(0) += 1;
    }

    grids.by_dim.retain(|dim, _| needed_dims.contains(dim));
    grids.center_by_dim.retain(|dim, _| needed_dims.contains(dim));

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
            let mut grid = CardinalGrid::new(&GridSettingsBuilder::new_2d(width, height).chunk_size(8).build());
            for y in 0..height {
                for x in 0..width {
                    let world = GlobalTilePos(min_tile + IVec2::new(x as i32, y as i32));
                    if param_set.is_blocked_at_terrain_only(&mut to_drain, ::tilemap_shared::DimensionRef(dim), world, Entity::PLACEHOLDER) {
                        grid.set_nav(UVec3::new(x, y, 0), Nav::Impassable);
                    }
                }
            }
            grid.build();
            grids.by_dim.insert(dim, AiNavGridCache {
                min: min_tile,
                grid,
                occupied: HashMap::default(),
            });
            grids.center_by_dim.insert(dim, center);
        }

        let Some(cache) = grids.by_dim.get_mut(&dim) else {
            continue;
        };
        cache.occupied.clear();
        for (being_ent, transform, dim_ref) in beings_query.iter() {
            if dim_ref.0 != dim {
                continue;
            }
            let gpos = GlobalTilePos::from(transform.translation.xy()).0;
            let max_grid = cache.min + IVec2::new(cache.grid.width() as i32 - 1, cache.grid.height() as i32 - 1);
            if gpos.x < cache.min.x || gpos.y < cache.min.y || gpos.x > max_grid.x || gpos.y > max_grid.y {
                continue;
            }
            let local = (gpos - cache.min).as_uvec2();
            cache.occupied.insert(UVec3::new(local.x, local.y, 0), being_ent);
        }
    }
}

fn health_ratio(
    being: Entity,
    bodies_query: &Query<&Bodies, With<Being>>,
    body_health_query: &Query<&BodyHealth>,
) -> f32 {
    let Ok(bodies) = bodies_query.get(being) else { return 0.0; };
    let Some(body_ent) = bodies.entities().first() else { return 0.0; };
    let Ok(health) = body_health_query.get(*body_ent) else { return 0.0; };
    if health.total_hp <= 0.0 {
        return 0.0;
    }
    (health.current_hp / health.total_hp).clamp(0.0, 1.0)
}

pub fn predator_hunt_behavior(
    time: Res<Time>,
    bodies_query: Query<&Bodies, With<Being>>,
    body_health_query: Query<&BodyHealth>,
    mut predators: Query<(
        Entity,
        &Transform,
        &::tilemap_shared::DimensionRef,
        Option<&RaceRef>,
        &mut InputDirection,
        &Hunger,
        &PredatorHuntThreshold,
        Option<&ControlledBy>,
    ), With<Predator>>,
    prey_query: Query<(Entity, &Transform, &::tilemap_shared::DimensionRef, Option<&RaceRef>), With<Being>>,
    grids: Res<AiNavGrids>,
    mut dynamic_blocking: Local<HashMap<UVec3, Entity>>,
    mut wander_states: Local<HashMap<Entity, WanderState>>,
) {
    let mut rng = rand::rng();
    let dt = time.delta_secs();

    for (pred_ent, pred_transf, &pred_dim, pred_race, mut input_dir, hunger, hunt_threshold, controlled_by) in predators.iter_mut() {
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_input {
                continue;
            }
        }

        let hp = health_ratio(pred_ent, &bodies_query, &body_health_query);
        if hunger.curr < hunt_threshold.0 || hp <= 0.9 {
            if hp <= 0.9 {
                error_once!(
                    target: log_targets::BEING_SYSTEM,
                    "Predator {:?} wandering: hp gate failed (hp_ratio={:.3}, hunger={:.3}, threshold={:.3})",
                    pred_ent, hp, hunger.curr, hunt_threshold.0
                );
            }
            let state = wander_states.entry(pred_ent).or_insert_with(|| WanderState::new(&mut rng));
            apply_wander_input(state, &mut input_dir, dt, &mut rng);
            continue;
        }

        let pred_pos = GlobalTilePos::from(pred_transf.translation.xy());

        let mut closest: Option<(Entity, GlobalTilePos, i32)> = None;
        for (prey_ent, prey_transf, &prey_dim, prey_race) in prey_query.iter() {
            if prey_ent == pred_ent || prey_dim != pred_dim {
                continue;
            }
            if let (Some(pred_race), Some(prey_race)) = (pred_race, prey_race) {
                if pred_race.0 == prey_race.0 {
                    continue;
                }
            }
            let prey_pos = GlobalTilePos::from(prey_transf.translation.xy());
            let delta = prey_pos.0 - pred_pos.0;
            let manhattan = delta.x.abs() + delta.y.abs();
            let Some((_, _, curr_best)) = closest else {
                closest = Some((prey_ent, prey_pos, manhattan));
                continue;
            };
            if manhattan < curr_best {
                closest = Some((prey_ent, prey_pos, manhattan));
            }
        }

        let Some((prey_ent_target, prey_pos, _)) = closest else {
            error!(
                target: log_targets::BEING_SYSTEM,
                "Predator {:?} wandering: no eligible prey found in dimension {:?}",
                pred_ent, pred_dim.0
            );
            let state = wander_states.entry(pred_ent).or_insert_with(|| WanderState::new(&mut rng));
            apply_wander_input(state, &mut input_dir, dt, &mut rng);
            continue;
        };

        let desired_to_prey = (prey_pos.0 - pred_pos.0).as_vec2();
        if desired_to_prey == Vec2::ZERO {
            input_dir.0 = Vec2::ZERO;
            continue;
        }
        let direct_chase_dir = desired_to_prey.normalize();

        let Some(cache) = grids.by_dim.get(&pred_dim.0) else {
            error!(
                target: log_targets::BEING_SYSTEM,
                "Predator {:?} chasing without nav grid in dimension {:?}",
                pred_ent, pred_dim.0
            );
            input_dir.0 = direct_chase_dir;
            continue;
        };

        let start_i = pred_pos.0 - cache.min;
        let goal_i = prey_pos.0 - cache.min;
        if start_i.x < 0 || start_i.y < 0 || goal_i.x < 0 || goal_i.y < 0 {
            error!(
                target: log_targets::BEING_SYSTEM,
                "Predator {:?} chasing outside nav grid bounds (start_i={:?}, goal_i={:?})",
                pred_ent, start_i, goal_i
            );
            input_dir.0 = direct_chase_dir;
            continue;
        }
        let start = UVec3::new(start_i.x as u32, start_i.y as u32, 0);
        let goal = UVec3::new(goal_i.x as u32, goal_i.y as u32, 0);
        if start.x >= cache.grid.width() || start.y >= cache.grid.height() || goal.x >= cache.grid.width() || goal.y >= cache.grid.height() {
            error!(
                target: log_targets::BEING_SYSTEM,
                "Predator {:?} chasing with start/goal out of nav grid bounds (start={:?}, goal={:?}, grid=({}, {}))",
                pred_ent, start, goal, cache.grid.width(), cache.grid.height()
            );
            input_dir.0 = direct_chase_dir;
            continue;
        }

        dynamic_blocking.clear();
        dynamic_blocking.reserve(cache.occupied.len());
        for (&pos, &ent) in cache.occupied.iter() {
            if ent == pred_ent || ent == prey_ent_target {
                continue;
            }
            dynamic_blocking.insert(pos, ent);
        }
        dynamic_blocking.remove(&goal);
        dynamic_blocking.remove(&start);

        let mut req = PathfindArgs::new(start, goal).blocking(&dynamic_blocking);
        let Some(path) = cache.grid.pathfind(&mut req) else {
            error!(
                target: log_targets::BEING_SYSTEM,
                "Predator {:?} chasing with direct fallback: pathfind failed (start={:?}, goal={:?}, blocked={})",
                pred_ent, start, goal, dynamic_blocking.len()
            );
            input_dir.0 = direct_chase_dir;
            continue;
        };

        let steps = path.path();
        let Some(next) = steps.first() else {
            error!(
                target: log_targets::BEING_SYSTEM,
                "Predator {:?} chasing with direct fallback: path contained no steps (start={:?}, goal={:?})",
                pred_ent, start, goal
            );
            input_dir.0 = direct_chase_dir;
            continue;
        };
        let next = next.xy().as_ivec2() + cache.min;
        let desired = (next - pred_pos.0).as_vec2();
        input_dir.0 = if desired == Vec2::ZERO { direct_chase_dir } else { desired.normalize() };
    }
}
