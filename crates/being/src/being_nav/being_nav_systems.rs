
use crate::being_messages::NavOrder;
use ::being_shared::*;
use ::tilemap_shared::*;
use std::{f32, ops::{Deref, DerefMut}};
use bevy::{
    ecs::entity::EntityHashMap,
    ecs::system::SystemParam,
    platform::collections::HashMap,
    prelude::*,
    tasks::{AsyncComputeTaskPool, futures_lite::future},
};
use common::log_targets::BEING_SYSTEM;
use param_sets::BlockingTileParamSet;
use smallset::SmallSet;
use ::being_shared::movement_shared_components::{InputMaxSpeed, InputSpeedThrottleMult};

use super::being_nav_resources::*;

type NavDimensionSet = SmallSet<[DimensionRef; 6]>;

struct NavDimensionSetLocal(NavDimensionSet);
impl Deref for NavDimensionSetLocal {
    type Target = NavDimensionSet;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for NavDimensionSetLocal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Default for NavDimensionSetLocal {
    fn default() -> Self {
        Self(SmallSet::new())
    }
}

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
    chasing_query: Query<(Entity, &::tilemap_shared::DimensionRef, &Chasing, ), (LocalAiControlled, )>,
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
    mut query: Query<(Entity, ), (LocalAiControlled, Without<WanderState>, Without<Chasing>, Without<Fleeing>)>,
    mut input_speed_query: Query<(&mut InputSpeedThrottleMult, &mut InputMaxSpeed, ), (), >,
) {
    for (being_ent, ) in query.iter_mut() {
        let Ok((mut input_speed_throttle_mult, mut input_max_speed)) = input_speed_query.get_mut(being_ent) else {
            continue;
        };
        cmd.entity(being_ent).try_remove::<GoTo>();
        input_speed_throttle_mult.0 = 1.0;
        input_max_speed.0 = f32::INFINITY;
    }
}
#[derive(SystemParam)]
#[allow(non_camel_case_types, )]
pub struct sync_ai_nav_grids_Locals<'s> {
    active_dims: Local<'s, NavDimensionSetLocal>,
    dirty_dims: Local<'s, NavDimensionSetLocal>,
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
    beings_query: Query<
        (
            Entity,
            &DimensionRef,
            Option<&GoTo>,
            Option<&LodLevel>,
        ),
        (LocalAiControlled, ),
    >,
    mut nav_grid_dirty_msgs: MessageReader<AiNavGridDirtyDim>,
    mut grids: ResMut<AiNavGrids>,
    mut rebuild_tasks: ResMut<AiNavGridRebuildTasks>,
    mut scratch: sync_ai_nav_grids_Locals,
) {
    const NAV_GRID_CENTER_SHIFT_TILES: i32 = 12;

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

    let active_dims = &mut scratch.active_dims;
    active_dims.clear();
    scratch.center_accum_by_dim.clear();
    for (being_ent, &being_dim, go_to, lod_level, ) in beings_query.iter() {
        let Some(_dim_ent) = dim_map.0.get_opt(being_dim.0).copied() else {
            continue;
        };
        let Ok(being_gpos) = param_set.gpos_query.get(being_ent) else {
            continue;
        };
        active_dims.insert(being_dim);

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
        if !active_dims.contains(&dim) {
            scratch.stale_dims.push(dim);
        }
    }
    for stale_dim in scratch.stale_dims.drain(..) {
        grids.by_dim.remove(&stale_dim);
        grids.center_by_dim.remove(&stale_dim);
        rebuild_tasks.pending_dims.remove(&stale_dim);
        rebuild_tasks.pending_generation_by_dim.remove(&stale_dim);
    }

    let dirty_dims = &mut scratch.dirty_dims;
    dirty_dims.clear();
    for nav_grid_dirty_msg in nav_grid_dirty_msgs.read() {
        dirty_dims.insert(nav_grid_dirty_msg.dim);
    }

    let rebuild_timer_finished = grids.rebuild_timer.tick(time.delta()).just_finished();

    scratch.loaded_chunk_bounds_by_dim.clear();
    scratch.loaded_chunk_bounds_by_dim.reserve(active_dims.len());
    for (&(loaded_dim, chunk_pos), _) in loaded_chunks.0.iter() {
        if !active_dims.contains(&loaded_dim) {
            continue;
        }
        let entry = scratch
            .loaded_chunk_bounds_by_dim
            .entry(loaded_dim)
            .or_insert((chunk_pos.0, chunk_pos.0));
        entry.0 = entry.0.min(chunk_pos.0);
        entry.1 = entry.1.max(chunk_pos.0);
    }

    for &dim in active_dims.iter() {
        if scratch.loaded_chunk_bounds_by_dim.contains_key(&dim) {
            continue;
        }
        grids.by_dim.remove(&dim);
        grids.center_by_dim.remove(&dim);
        rebuild_tasks.pending_dims.remove(&dim);
        rebuild_tasks.pending_generation_by_dim.remove(&dim);
    }

    scratch.rebuild_inputs.clear();
    scratch.rebuild_inputs.reserve(active_dims.len());
    for &dim in active_dims.iter() {
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
        let dirty = dirty_dims.contains(&dim);
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
                .map(AiNavGridRebuildInput::build_ai_nav_grid_cache)
                .collect::<Vec<AiNavGridRebuildResult>>()
        });
        rebuild_tasks.tasks.push(task);
    }

    for (&dim, cache) in grids.by_dim.iter_mut() {
        if !active_dims.contains(&dim) {
            continue;
        }
        cache.occupied.clear();
    }

    for &dim in active_dims.iter() {
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
