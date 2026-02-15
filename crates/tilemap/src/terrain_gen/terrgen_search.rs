use bevy::{ecs::entity::EntityHashSet, prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use std::f32::consts::PI;
use crate::{terrain_gen::{terrgen_components::FailedSearchOplistFilterHolder, terrgen_messages::*, terrgen_resources::{TerrGenAsyncTasks, TerrGenSearchTaskResult}}};
use ::tilemap_shared::*;

#[derive(Clone)]
struct TerrGenSearchTaskInput {
    probe: TerrainProbe,
    opfilter: Option<OpFilter>,
}

#[derive(bevy::ecs::system::SystemParam)]
#[allow(unused_parens, )]
pub struct SearchParams<'w, 's>
{
    ew_pos_search: MessageWriter<'w, TerrainProbe>,
    reader_search_successful: MessageReader<'w, 's, SuitablePosFound>,
    mreader_search_failed: MessageReader<'w, 's, SearchFailed>,
}


#[derive(Component, Debug, Default, Copy, Clone)]
pub struct AwaitingStartSearch;


#[allow(unused_parens)]
//input: PosSearch messages. output: SearchFailed or SuitablePosFound(emitted in produce_tiles)
pub fn search_suitable_positions(
    mut cmd: Commands,
    mut terrain_probe: ResMut<Messages<TerrainProbe>>, mut mwriter_search_failed: MessageWriter<SearchFailed>,
    mut mwriter_pending_ops: MessageWriter<PendingOp>, mut mreader_suitable_pos_found: MessageReader<SuitablePosFound>,
    studied_ops: Query<&OpFilter, ( )>,
    failed_search_oplist_filter_holder: Query<Entity, (With<FailedSearchOplistFilterHolder>)>,
    mut terrgen_tasks: ResMut<TerrGenAsyncTasks>,
    mut found_suitable_positions: Local<EntityHashSet>,
    mut new_pending_ops: Local<Vec<PendingOp>>,
    mut new_pos_searches: Local<Vec<TerrainProbe>>,
    mut search_failed_evs: Local<Vec<SearchFailed>>,
    mut failed_entities: Local<Vec<Entity>>,
) {
    found_suitable_positions.clear();
    for found_ev in mreader_suitable_pos_found.read() {
        found_suitable_positions.insert(found_ev.op_filter_ent);
    }

    new_pending_ops.clear();
    new_pos_searches.clear();
    search_failed_evs.clear();
    failed_entities.clear();

    terrgen_tasks.search_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            new_pending_ops.extend(result.new_pending_ops);
            new_pos_searches.extend(result.new_pos_searches);
            search_failed_evs.extend(result.search_failed.iter().cloned().map(SearchFailed));
            failed_entities.extend(result.search_failed);
            false
        } else {
            true
        }
    });

    for failed in failed_entities.drain(..) {
        if let Ok(failed_search_oplist_filter_holder) = failed_search_oplist_filter_holder.single() {
            cmd.entity(failed).try_insert(ChildOf(failed_search_oplist_filter_holder));
        }
    }

    if !new_pending_ops.is_empty() {
        mwriter_pending_ops.write_batch(new_pending_ops.drain(..));
    }
    if !new_pos_searches.is_empty() {
        terrain_probe.write_batch(new_pos_searches.drain(..));
    }
    if !search_failed_evs.is_empty() {
        mwriter_search_failed.write_batch(search_failed_evs.drain(..));
    }

    if terrain_probe.is_empty() { return; }

    let mut inputs = Vec::with_capacity(terrain_probe.len());
    for pos_search in terrain_probe.drain() {
        let opfilter = studied_ops.get(pos_search.operation_filter).ok().cloned();
        inputs.push(TerrGenSearchTaskInput { probe: pos_search, opfilter });
    }

    let found = found_suitable_positions.drain().collect::<EntityHashSet>();
    let task_pool = AsyncComputeTaskPool::get();
    terrgen_tasks.search_tasks.push(task_pool.spawn(async move {
        process_search_batch(inputs, found)
    }));
}


fn process_search_batch(inputs: Vec<TerrGenSearchTaskInput>, successful_filters: EntityHashSet) -> TerrGenSearchTaskResult {
    let pending_count = inputs.len();
    let mut new_pending_ops = Vec::with_capacity(pending_count);
    let mut new_pos_searches = Vec::with_capacity(pending_count);
    let mut search_failed = Vec::with_capacity(pending_count);

    for input in inputs {
        let pos_search = input.probe;

        if successful_filters.contains(&pos_search.operation_filter) {
            info!(target: "pos_search","Found suitable position for {:?}", pos_search.operation_filter);
            continue;
        }

        let (filtered_op, step_size, curr_iteration_batch_i, iterations_per_batch, max_batches, dimension_ref) =
            (pos_search.operation_filter, pos_search.step_size, pos_search.curr_iteration_batch_i, pos_search.iterations_per_batch, pos_search.max_batches, pos_search.dimension_ref);

        let Some(opfilter) = input.opfilter else {
            if curr_iteration_batch_i == 0 {
                let mut new_search = pos_search;
                new_search.curr_iteration_batch_i -= 1;
                new_pos_searches.push(new_search);
            } else if curr_iteration_batch_i == -2 {
                error!(target: "pos_search", "StudiedOp entity {:?} not found in search_suitable_position, giving up", filtered_op);
                search_failed.push(filtered_op);
            }
            continue;
        };
        let curr_iteration_batch_i = curr_iteration_batch_i.max(0);

        match pos_search.probe_pattern {
            ProbePattern::Radial(explore_angle) => {
                let calculate_pos = |i_within_batch: u16, probe_direction: f32| -> GlobalTilePos {
                    let global_i = (curr_iteration_batch_i as u16 * iterations_per_batch as u16 + i_within_batch) as f32 * step_size as f32;
                    opfilter.search_start_pos + GlobalTilePos::from(IVec2::new(
                        (global_i * probe_direction.cos()) as i32, (global_i * probe_direction.sin()) as i32,
                    ))
                };

                if let Some(explore_angle) = explore_angle {
                    let start_i_within_batch = (curr_iteration_batch_i == 0) as u16;

                    for i_within_batch in start_i_within_batch..iterations_per_batch {
                        new_pending_ops.push(PendingOp {
                            oplist: opfilter.start_oplist,
                            dimension_ref,
                            gpos: calculate_pos(i_within_batch, explore_angle),
                            filtered_op,
                            max_emitted_results: pos_search.max_emitted_results,
                        });
                    }
                    if curr_iteration_batch_i as u16 + 1 < max_batches {
                        new_pos_searches.push(TerrainProbe {
                            curr_iteration_batch_i: curr_iteration_batch_i + 1,
                            probe_pattern: ProbePattern::Radial(Some(explore_angle)),
                            ..pos_search
                        });
                    } else {
                        error!(target: "pos_search", "No more batches to search for {:?}", opfilter);
                        search_failed.push(filtered_op);
                    }
                } else {
                    if curr_iteration_batch_i as u16 >= max_batches {
                        error!(target: "pos_search", "curr No more batches to search for {:?}", pos_search);
                        continue;
                    }
                    let divisions = 8;
                    for i in 0..divisions {
                        let angle = 2.0 * PI * (i as f32) / (divisions as f32);
                        new_pos_searches.push(TerrainProbe{
                            probe_pattern: ProbePattern::Radial(Some(angle)),
                            ..pos_search
                        });
                    }
                }
            }
            ProbePattern::Spiral(mut curr_length_in_dir, mut steps_taken, mut dir_vec, mut pos, mut turn_parity) => {
                trace!(target: "pos_search", "Spiral search started at pos {:?}, dir_vec {:?}, curr_length_in_dir {}, turns {}",
                    pos, dir_vec, curr_length_in_dir, turn_parity);

                for _ in 0..iterations_per_batch {
                    pos = pos + GlobalTilePos(dir_vec.saturating_mul(IVec2::splat(step_size as i32)));
                    new_pending_ops.push(PendingOp {
                        dimension_ref,
                        oplist: opfilter.start_oplist,
                        gpos: pos,
                        filtered_op,
                        max_emitted_results: pos_search.max_emitted_results,
                    });

                    steps_taken += 1;
                    if steps_taken >= curr_length_in_dir {
                        steps_taken = 0;

                        dir_vec = dir_vec.perp();
                        curr_length_in_dir = curr_length_in_dir.saturating_add(turn_parity as u64);
                        turn_parity = !turn_parity;
                    }
                }
                if curr_iteration_batch_i as u16 + 1 < max_batches {
                    new_pos_searches.push(TerrainProbe{
                        curr_iteration_batch_i: curr_iteration_batch_i + 1,
                        probe_pattern: ProbePattern::Spiral(curr_length_in_dir, steps_taken, dir_vec, pos, turn_parity),
                        ..pos_search
                    });
                } else {
                    error!(target: "pos_search", "No more batches to search for {:?}", opfilter);
                    search_failed.push(filtered_op);
                }
            },
        }
    }

    TerrGenSearchTaskResult {
        new_pending_ops,
        new_pos_searches,
        search_failed,
    }
}
