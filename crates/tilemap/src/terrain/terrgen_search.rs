use bevy::{ecs::entity::{EntityHashSet, EntityHashMap}, prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use game_common::game_common_components::EntityZeroRef;
use std::f32::consts::PI;
use crate::terrain::{
    opfilter::opfilter_components::OpFilter,
    terrprobe::terrprobe_components::TerrProbeTempl,
    terrprobe::terrprobe_messages::*,
    terrgen_components::FailedSearchOplistFilterHolder,
    terrgen_messages::PendingOp,
    terrgen_resources::{TerrGenAsyncTasks, TerrGenSearchTaskResult},
};
use ::tilemap_shared::*;

#[derive(Clone)]
struct TerrGenSearchTaskInput {
    probe: TerrProbeJob,
    templ: Option<TerrProbeTempl>,
    opfilter: Option<OpFilter>,
    root_oplist: Option<DimensionRootOplist>,
}

#[derive(bevy::ecs::system::SystemParam)]
#[allow(unused_parens, )]
pub struct SearchParams<'w, 's>
{
    pub ew_pos_search: MessageWriter<'w, TerrProbeJob>,
    pub reader_search_successful: MessageReader<'w, 's, SuitablePosFound>,
    pub mreader_search_failed: MessageReader<'w, 's, SearchFailed>,
    pub pending_by_requester: Local<'s, EntityHashMap<Vec<(Entity, GlobalTilePos, DimensionRef, EntityZeroRef)>>>,
    pub min_result_distance_by_requester: Local<'s, EntityHashMap<u64>>,
    pub pos_searches_msgs_to_write: Local<'s, Vec<TerrProbeJob>>,
}

impl<'w, 's> SearchParams<'w, 's> {
    pub fn read_successful(&mut self) -> impl Iterator<Item = &SuitablePosFound> + '_ {
        self.reader_search_successful.read()
    }

    pub fn read_failed(&mut self) -> impl Iterator<Item = &SearchFailed> + '_ {
        self.mreader_search_failed.read()
    }

    pub fn write_probes<I>(&mut self, probes: I)
    where
        I: IntoIterator<Item = TerrProbeJob>,
    {
        self.ew_pos_search.write_batch(probes);
    }
    pub fn write_pos_searches(&mut self) {
        self.ew_pos_search.write_batch(self.pos_searches_msgs_to_write.drain(..));
    }

}

use serde::{Deserialize, Serialize};
#[derive(Component, Debug, Default, Copy, Clone, Deserialize, Serialize)]
pub struct AwaitingStartSearch;


#[allow(unused_parens)]
//input: PosSearch messages. output: SearchFailed or SuitablePosFound(emitted in produce_tiles)
pub fn search_suitable_positions(
    mut cmd: Commands,
    mut terrain_probe: ResMut<Messages<TerrProbeJob>>, mut mwriter_search_failed: MessageWriter<SearchFailed>,
    mut mwriter_pending_ops: MessageWriter<PendingOp>, mut mreader_suitable_pos_found: MessageReader<SuitablePosFound>,
    terrprobe_query: Query<&TerrProbeTempl>,
    studied_ops: Query<&OpFilter, ( )>,
    dimensions_query: Query<&DimensionRootOplist>,
    failed_search_oplist_filter_holder: Query<Entity, (With<FailedSearchOplistFilterHolder>)>,
    mut terrgen_tasks: ResMut<TerrGenAsyncTasks>,
    mut found_suitable_positions: Local<EntityHashSet>,
    mut new_pending_ops: Local<Vec<PendingOp>>,
    mut new_pos_searches: Local<Vec<TerrProbeJob>>,
    mut search_failed_evs: Local<Vec<SearchFailed>>,
    mut failed_entities: Local<Vec<Entity>>,
) {
    found_suitable_positions.clear();
    for found_ev in mreader_suitable_pos_found.read() {
        found_suitable_positions.insert(found_ev.requester);
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
    mwriter_pending_ops.write_batch(new_pending_ops.drain(..));
    terrain_probe.write_batch(new_pos_searches.drain(..));
    mwriter_search_failed.write_batch(search_failed_evs.drain(..));

    if terrain_probe.is_empty() { return; }

    let mut inputs = Vec::with_capacity(terrain_probe.len());
    for pos_search in terrain_probe.drain() {
        let templ = terrprobe_query.get(pos_search.templ_ent).ok().cloned();
        let opfilter = templ
            .as_ref()
            .and_then(|t| studied_ops.get(t.opfilter_ent).ok().cloned());
        let root_oplist = dimensions_query.get(pos_search.dimension_ref.0).ok().copied();
        inputs.push(TerrGenSearchTaskInput { probe: pos_search, templ, opfilter, root_oplist });
    }
    let found = found_suitable_positions.drain().collect::<EntityHashSet>();
    let task_pool = AsyncComputeTaskPool::get();
    terrgen_tasks.search_tasks.push(task_pool.spawn(async move {
        process_search_batch(inputs, found)
    }));
}


fn process_search_batch(inputs: Vec<TerrGenSearchTaskInput>, successful_requesters: EntityHashSet) -> TerrGenSearchTaskResult {
    let pending_count = inputs.len();
    let mut new_pending_ops = Vec::with_capacity(pending_count);
    let mut new_pos_searches = Vec::with_capacity(pending_count);
    let mut search_failed = Vec::with_capacity(pending_count);

    for input in inputs {
        let pos_search = input.probe;

        if successful_requesters.contains(&pos_search.requester) {
            info!(target: "pos_search","Found suitable position for {:?}", pos_search.requester);
            continue;
        }
        let Some(templ) = input.templ else {
            search_failed.push(pos_search.requester);
            continue;
        };
        let (filtered_op, requester, step_size, curr_iteration_batch_i, iterations_per_batch, max_batches, dimension_ref) =
            (
                templ.opfilter_ent,
                pos_search.requester,
                templ.step_size,
                pos_search.curr_iteration_batch_i,
                templ.iterations_per_batch,
                templ.max_batches,
                pos_search.dimension_ref,
            );
        let Some(root_oplist) = input.root_oplist else {
            error!(target: "pos_search", "No root oplist found for dimension {:?}", dimension_ref);
            search_failed.push(requester);
            continue;
        };

        let Some(opfilter) = input.opfilter else {
            if curr_iteration_batch_i == 0 {
                let mut new_search = pos_search;
                new_search.curr_iteration_batch_i -= 1;
                new_pos_searches.push(new_search);
            } else if curr_iteration_batch_i == -2 {
                error!(target: "pos_search", "StudiedOp entity {:?} not found in search_suitable_position, giving up", filtered_op);
                search_failed.push(requester);
            }
            continue;
        };
        let curr_iteration_batch_i = curr_iteration_batch_i.max(0);

        match templ.probe_pattern.clone() {
            ProbePattern::Radial(_) => {
                let calculate_pos = |i_within_batch: u16, probe_direction: f32| -> GlobalTilePos {
                    let global_i = (curr_iteration_batch_i as u16 * iterations_per_batch as u16 + i_within_batch) as f32 * step_size as f32;
                    pos_search.search_start_pos + GlobalTilePos::from(IVec2::new(
                        (global_i * probe_direction.cos()) as i32, (global_i * probe_direction.sin()) as i32,
                    ))
                };
                if curr_iteration_batch_i as u16 >= max_batches {
                    error!(target: "pos_search", "No more batches to search for {:?}", pos_search);
                    continue;
                }
                let divisions = 8;
                let start_i_within_batch = (curr_iteration_batch_i == 0) as u16;
                for i in 0..divisions {
                    let angle = 2.0 * PI * (i as f32) / (divisions as f32);
                    for i_within_batch in start_i_within_batch..iterations_per_batch {
                        new_pending_ops.push(PendingOp {
                            oplist: root_oplist,
                            dimension_ref,
                            gpos: calculate_pos(i_within_batch, angle),
                            filtered_op,
                            requester,
                            max_emitted_results: templ.max_emitted_results,
                        });
                    }
                }
                if curr_iteration_batch_i as u16 + 1 < max_batches {
                    let mut next_search = pos_search.clone();
                    next_search.curr_iteration_batch_i = curr_iteration_batch_i + 1;
                    new_pos_searches.push(next_search);
                } else {
                    error!(target: "pos_search", "No more batches to search for {:?}", opfilter);
                    search_failed.push(requester);
                }
            }
            ProbePattern::Spiral(_, _, _, _, _) => {
                let (
                    mut curr_length_in_dir,
                    mut steps_taken,
                    mut dir_vec,
                    mut pos,
                    mut turn_parity,
                ) = spiral_state_at_batch_start(
                    pos_search.search_start_pos,
                    step_size,
                    curr_iteration_batch_i as u16,
                    iterations_per_batch,
                );
                trace!(target: "pos_search", "Spiral search started at pos {:?}, dir_vec {:?}, curr_length_in_dir {}, turns {}",
                    pos, dir_vec, curr_length_in_dir, turn_parity);

                for _ in 0..iterations_per_batch {
                    pos = pos + GlobalTilePos(dir_vec.saturating_mul(IVec2::splat(step_size as i32)));
                    new_pending_ops.push(PendingOp {
                        dimension_ref,
                        oplist: root_oplist,
                        gpos: pos,
                        filtered_op,
                        requester,
                        max_emitted_results: templ.max_emitted_results,
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
                    let mut next_search = pos_search.clone();
                    next_search.curr_iteration_batch_i = curr_iteration_batch_i + 1;
                    new_pos_searches.push(next_search);
                } else {
                    error!(target: "pos_search", "No more batches to search for {:?}", opfilter);
                    search_failed.push(requester);
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

fn spiral_state_at_batch_start(
    start_pos: GlobalTilePos,
    step_size: u16,
    batch_i: u16,
    iterations_per_batch: u16,
) -> (u64, u64, IVec2, GlobalTilePos, bool) {
    let mut curr_length_in_dir: u64 = 1;
    let mut steps_taken: u64 = 0;
    let mut dir_vec = IVec2::new(0, 1);
    let mut pos = start_pos;
    let mut turn_parity = false;
    let steps_to_advance = batch_i as u64 * iterations_per_batch as u64;
    for _ in 0..steps_to_advance {
        pos = pos + GlobalTilePos(dir_vec.saturating_mul(IVec2::splat(step_size as i32)));
        steps_taken += 1;
        if steps_taken >= curr_length_in_dir {
            steps_taken = 0;
            dir_vec = dir_vec.perp();
            curr_length_in_dir = curr_length_in_dir.saturating_add(turn_parity as u64);
            turn_parity = !turn_parity;
        }
    }
    (curr_length_in_dir, steps_taken, dir_vec, pos, turn_parity)
}

#[macro_export]
macro_rules! run_suitable_pos_search_logic {
    (
        target: $target:expr,
        searched_entity_label: $searched_entity_label:expr,
        cmd: $cmd:ident,
        searching_entities: $searching_entities:ident,
        search_params: $search_params:ident,
        make_search_request: $make_search_request:ident,
        handle_success_event: $handle_success_event:ident,
        handle_pending_failure: $handle_pending_failure:ident,
    ) => {{
        $search_params.pending_by_requester.clear();
        for (ent, &dim_ref, &my_pos, &ezero_ref, _, searching_for) in $searching_entities.iter() {
            if let Some(SearchingForSuitablePos { requester }) = searching_for {
                $search_params
                    .pending_by_requester
                    .entry(*requester)
                    .or_default()
                    .push((ent, my_pos, dim_ref, ezero_ref));
            }
        }

        $searching_entities
            .iter().for_each(|(search_ent, &dim_ref, &global_pos, ezero_ref, is_awaiting_start, ..)| {
                if !is_awaiting_start {
                    return;
                }
                $cmd.entity(search_ent).try_remove::<AwaitingStartSearch>();

                let Some(mut probe) =
                    $make_search_request(&mut $cmd, search_ent, global_pos, *ezero_ref)
                else {
                    return;
                };
                if probe.requester == Entity::PLACEHOLDER {
                    probe.requester = search_ent;
                }
                let requester = probe.requester;

                info!(
                    target: $target,
                    "Starting suitable-pos search for {} entity {:?} at position {:?}",
                    $searched_entity_label,
                    search_ent,
                    global_pos
                );

                $cmd.entity(search_ent)
                    .try_insert(SearchingForSuitablePos { requester });
                $search_params
                    .min_result_distance_by_requester
                    .insert(requester, probe.min_result_distance as u64);
                $search_params.pos_searches_msgs_to_write.push(probe);
                $search_params
                    .pending_by_requester
                    .entry(requester)
                    .or_default()
                    .push((search_ent, global_pos, dim_ref, *ezero_ref));
            });

        let mut accepted_results: Vec<(DimensionRef, GlobalTilePos)> = Vec::new();
        for suitable_pos in $search_params.reader_search_successful.read() {
            let requester = suitable_pos.requester;
            let Some(owners) = $search_params.pending_by_requester.get_mut(&requester) else {
                continue;
            };
            let Some(&(search_ent, my_pos, dim_ref, ezero_ref)) = owners.last() else {
                continue;
            };

            let min_result_distance = $search_params
                .min_result_distance_by_requester
                .get(&requester)
                .copied()
                .unwrap_or(0);
            let min_result_distance_sq = min_result_distance.saturating_mul(min_result_distance);
            let too_close = min_result_distance_sq > 0 && accepted_results.iter().any(|(taken_dim_ref, taken_pos)| {
                *taken_dim_ref == dim_ref
                    && suitable_pos.found_pos.distance_squared(taken_pos) <= min_result_distance_sq
            });
            if too_close {
                trace!(
                    target: $target,
                    "Skipping suitable-pos result for requester {:?} at {:?} due to min_result_distance {}",
                    requester,
                    suitable_pos.found_pos,
                    min_result_distance
                );
                continue;
            }

            if $handle_success_event(
                &mut $cmd,
                search_ent,
                my_pos,
                dim_ref,
                ezero_ref,
                suitable_pos.found_pos,
            ) {
                owners.pop();
                if owners.is_empty() {
                    $search_params.pending_by_requester.remove(&requester);
                    $search_params.min_result_distance_by_requester.remove(&requester);
                }
                accepted_results.push((dim_ref, suitable_pos.found_pos));
                $cmd.entity(search_ent).try_remove::<SearchingForSuitablePos>();
            }
        }

        for failed_search in $search_params.mreader_search_failed.read() {
            let Some(pending_searches) = $search_params.pending_by_requester.remove(&failed_search.0) else {
                continue;
            };
            $search_params.min_result_distance_by_requester.remove(&failed_search.0);
            for (search_ent, global_pos, dim_ref, ezero_ref) in pending_searches {
                error!(
                    target: $target,
                    "Failed to find suitable pos for a {} entity, {:?}",
                    $searched_entity_label,
                    failed_search.0
                );
                $cmd.entity(search_ent).try_remove::<SearchingForSuitablePos>();
                $handle_pending_failure(search_ent, global_pos, dim_ref, ezero_ref, failed_search.0);
            }
        }

        $search_params.write_pos_searches();
    }};
}

#[macro_export]
macro_rules! run_oneshot_suitable_pos_search_logic {
    (
        target: $target:expr,
        searched_label: $searched_label:expr,
        cmd: $cmd:ident,
        search_params: $search_params:ident,
        active_probe_ent: $active_probe_ent:ident,
        search_finished: $search_finished:ident,
        make_search_request: $make_search_request:ident,
        handle_success: $handle_success:ident,
        handle_failure: $handle_failure:ident,
    ) => {{
        if *$search_finished {
            $search_params.write_pos_searches();
        } else {
            if $active_probe_ent.is_none() {
                if let Some(mut probe) = $make_search_request(&mut $cmd) {
                    if probe.requester == Entity::PLACEHOLDER {
                        probe.requester = $cmd.spawn_empty().id();
                    }
                    let requester = probe.requester;
                    info!(
                        target: $target,
                        "Starting one-shot suitable-pos search for {} with requester {:?}",
                        $searched_label,
                        requester
                    );
                    *$active_probe_ent = Some(requester);
                    $search_params.pos_searches_msgs_to_write.push(probe);
                }
            }

            if let Some(active_probe_ent) = *$active_probe_ent {
                for suitable_pos in $search_params.reader_search_successful.read() {
                    if suitable_pos.requester != active_probe_ent {
                        continue;
                    }
                    if $handle_success(
                        &mut $cmd,
                        suitable_pos.found_pos,
                        active_probe_ent,
                        suitable_pos.val,
                    ) {
                        *$search_finished = true;
                        *$active_probe_ent = None;
                        break;
                    }
                }

                if !*$search_finished {
                    for failed_search in $search_params.mreader_search_failed.read() {
                        if failed_search.0 != active_probe_ent {
                            continue;
                        }
                        error!(
                            target: $target,
                            "Failed to find suitable pos for one-shot {} search, terrain probe {:?}",
                            $searched_label,
                            active_probe_ent
                        );
                        $handle_failure(&mut $cmd, active_probe_ent);
                        *$search_finished = true;
                        *$active_probe_ent = None;
                        break;
                    }
                }
            }

            $search_params.write_pos_searches();
        }
    }};
}
