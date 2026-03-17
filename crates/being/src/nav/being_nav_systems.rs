use crate::being_components::{Being, Chaser};
use ::being_shared::*;
use ::tilemap_shared::{ChunkPos, GlobalTilePos, LoadedChunks};
use bevy::{
    ecs::entity::{EntityHashMap, EntityHashSet},
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_northstar::prelude::*;
use common::log_targets::BEING_SYSTEM;
use faction::faction_components::BelongsToAPlayerFaction;
use movement::movement_components::{InputMoveDir, SpeedMagnitude};
use param_sets::BlockingTileParamSet;
use tilemap::chunking::chunking_components::{ActivateChunksAround, ActivatingChunks};
use tilemap::chunking::chunking_resources::AaChunkRangeSettings;
use std::time::Duration;

use super::being_nav_resources::{AiNavGrids, ChaserNavPlans};
use super::being_nav_structs::{AiNavGridCache, ChaserNavPlan};
use super::being_nav_helpers::{
    cardinal_step_toward,
    rebuild_dynamic_blocking, sync_chase_retained_chunks,
};

pub fn rebuild_chaser_nav_plans(
    time: Res<Time>,
    chasers: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            &Chaser,
            Option<&SpeedMagnitude>,
        ),
        (With<Being>, LocalAiControlled),
    >,
    beings_query: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&SpeedMagnitude>,
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

        let Some(cache) = grids.by_dim.get(&chaser_dim.0) else {
            plan.reset(chase_interval);
            continue;
        };

        let Some((start, goal)) = cache.local_path_points(*chaser_gpos, target_pos) else {
            plan.reset(chase_interval);
            continue;
        };

        let target_shifted = plan
            .last_target_pos
            .map(|prev| (prev.0 - target_pos.0).abs().max_element() >= 2)
            .unwrap_or(true);
        let need_rebuild = plan.path_tiles.is_empty() || timer_finished || target_shifted;
        if !need_rebuild {
            continue;
        }

        rebuild_dynamic_blocking(&mut dynamic_blocking, cache, chaser_ent, to_chase.target, start, goal);

        let mut req = PathfindArgs::new(start, goal).blocking(&dynamic_blocking);
        let Some(path) = cache.grid.pathfind(&mut req) else {
            plan.reset(chase_interval);
            continue;
        };

        plan.path_tiles.clear();
        plan.path_tiles.extend(
            path.path()
                .iter()
                .map(|step| GlobalTilePos(step.xy().as_ivec2() + cache.min)),
        );
        plan.next_step_ix = 0;
        plan.last_target_pos = Some(target_pos);
        plan.rebuild_timer = Timer::new(chase_interval, TimerMode::Once);
        trace!(target: BEING_SYSTEM, "Rebuilt chase nav for {:?}: prey {:?}, dist {:.2}, interval {:.2}s, steps {}", chaser_ent, to_chase.target, chaser_gpos.taxicab_tile_distance(target_pos), chase_interval.as_secs_f32(), plan.path_tiles.len());
    }

    plans.by_ent.retain(|ent, _| active_chasers.contains(ent));
}

pub fn sync_ai_nav_grids(
    time: Res<Time>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_range: Res<AaChunkRangeSettings>,
    mut param_set: BlockingTileParamSet,
    chasers_query: Query<
        (
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            Option<&ComputedBy>,
            &Chaser,
        ),
        With<Being>,
    >,
    beings_query: Query<(Entity, &GlobalTilePos, &::tilemap_shared::DimensionRef), With<Being>>,
    mut grids: ResMut<AiNavGrids>,
) {
    let mut needed_dims: HashSet<Entity> = HashSet::default();
    let mut dim_centers: HashMap<Entity, IVec2> = HashMap::default();
    let mut dim_center_counts: HashMap<Entity, i32> = HashMap::default();

    for (gpos, dim_ref, controlled_by, _to_chase) in chasers_query.iter() {
        if let Some(controlled_by) = controlled_by {
            if controlled_by.human_dc_input {
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
        let Some(max_tile) = max_tile else {
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

pub fn chase_behavior(
    mut chasers: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            &Chaser,
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

        let Some(plan) = plans.by_ent.get_mut(&chaser_ent) else {
            input_move_dir.0 = direct_chase_dir;
            continue;
        };

        let Some(next) = plan.next_step(chaser_pos) else {
            input_move_dir.0 = direct_chase_dir;
            continue;
        };
        let desired = cardinal_step_toward(next.0 - chaser_pos.0);
        let move_input = if desired == IVec2::ZERO {
            direct_chase_dir
        } else {
            desired.as_vec2()
        };
        input_move_dir.0 = move_input;
    }
}

pub fn retain_chunks_for_player_faction_chasers(
    mut commands: Commands,
    mut chasers: Query<
        (
            Entity,
            &GlobalTilePos,
            &::tilemap_shared::DimensionRef,
            &Chaser,
            Option<&mut ActivatingChunks>,
        ),
        (With<Being>, LocalAiControlled, Without<ActivateChunksAround>),
    >,
    targets_query: Query<(&GlobalTilePos, &::tilemap_shared::DimensionRef, Has<BelongsToAPlayerFaction>)>,
    plans: Res<ChaserNavPlans>,
    mut seen_chunks: Local<HashSet<ChunkPos>>,
    mut desired_chunks: Local<Vec<ChunkPos>>,
    mut to_insert: Local<Vec<(Entity, ActivatingChunks)>>,
    mut to_remove: Local<Vec<Entity>>,
) {
    for (chaser_ent, chaser_gpos, &chaser_dim, to_chase, activating_chunks) in chasers.iter_mut() {
        let Ok((_target_gpos, &target_dim, target_is_player_faction)) = targets_query.get(to_chase.target) else {
            if activating_chunks.is_some() {
                to_remove.push(chaser_ent);
                debug!(target: BEING_SYSTEM, "Removing chase chunk retention from {:?}: target {:?} missing", chaser_ent, to_chase.target);
            }
            continue;
        };

        if !target_is_player_faction {
            if activating_chunks.is_some() {
                to_remove.push(chaser_ent);
                debug!(target: BEING_SYSTEM, "Removing chase chunk retention from {:?}: target {:?} not player faction", chaser_ent, to_chase.target);
            }
            continue;
        }

        if target_dim != chaser_dim {
            if activating_chunks.is_some() {
                to_remove.push(chaser_ent);
            }
            continue;
        }

        let Some(plan) = plans.by_ent.get(&chaser_ent) else {
            continue;
        };

        seen_chunks.clear();
        desired_chunks.clear();

        let current_chunk_pos = ChunkPos::from(*chaser_gpos);
        if seen_chunks.insert(current_chunk_pos) {
            desired_chunks.push(current_chunk_pos);
        }

        for step in plan.path_tiles.iter() {
            let cpos = ChunkPos::from(*step);
            if seen_chunks.insert(cpos) {
                desired_chunks.push(cpos);
            }
        }

        sync_chase_retained_chunks(activating_chunks, &desired_chunks, chaser_ent, &mut to_insert);
    }

    for (ent, activating_chunks) in to_insert.drain(..) {
        commands.entity(ent).try_insert(activating_chunks);
    }
    for ent in to_remove.drain(..) {
        commands.entity(ent).try_remove::<ActivatingChunks>();
    }
}
