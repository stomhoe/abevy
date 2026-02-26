use bevy::{ecs::entity::{EntityHashSet, EntityHashMap}, prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use game_common::game_common_components::EntityZeroRef;
use crate::terrain::{
    terrprobe::terrprobe_components::TerrProbeTempl,
    terrprobe::terrprobe_pattern_chunk::process_chunk_pattern,
    terrprobe::terrprobe_pattern_concentric::process_concentric_pattern,
    terrprobe::terrprobe_messages::*,
    terrprobe::terrprobe_pattern_region::process_region_pattern,
    terrprobe::terrprobe_pattern_spiral::process_spiral_pattern,
    terrgen_components::FailedSearchOplistFilterHolder,
    terrgen_messages::PendingOp,
    terrgen_resources::{TerrGenAsyncTasks, TerrGenSearchTaskResult},
};
use ::tilemap_shared::*;

#[derive(Clone)]
struct TerrGenSearchTaskInput {
    probe: TerrProbeJob,
    templ: TerrProbeTempl,
    root_oplist: DimensionRootOplist,
}

#[derive(bevy::ecs::system::SystemParam)]
#[allow(unused_parens, )]
pub struct SearchParams<'w, 's>
{
    pub ew_pos_search: MessageWriter<'w, TerrProbeJob>,
    pub reader_search_successful: MessageReader<'w, 's, SuitablePosFound>,
    pub reader_sampled_value_matrix: MessageReader<'w, 's, SampledValuesCollected>,
    pub mreader_search_failed: MessageReader<'w, 's, SearchFailed>,
    pub pending_by_requester: Local<'s, EntityHashMap<Vec<(Entity, GlobalTilePos, DimensionRef, EntityZeroRef)>>>,
    pub requester_collect_all: Local<'s, EntityHashMap<bool>>,
    pub requester_had_success: Local<'s, EntityHashMap<bool>>,
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

    pub fn read_sampled_matrices(&mut self) -> impl Iterator<Item = &SampledValuesCollected> + '_ {
        self.reader_sampled_value_matrix.read()
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
    mut mreader_sampled_value_matrix_found: MessageReader<SampledValuesCollected>,
    terrprobe_query: Query<&TerrProbeTempl>,
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
    new_pending_ops.clear();
    new_pos_searches.clear();
    search_failed_evs.clear();
    failed_entities.clear();
    for found_ev in mreader_suitable_pos_found.read() {
        found_suitable_positions.insert(found_ev.requester);
    }
    for sampled_matrix_ev in mreader_sampled_value_matrix_found.read() {
        found_suitable_positions.insert(sampled_matrix_ev.requester);
    }
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
        let Ok(templ) = terrprobe_query.get(pos_search.templ_ent).cloned() else {
            search_failed_evs.push(SearchFailed(pos_search.requester));
            continue;
        };
        let Ok(&root_oplist) = dimensions_query.get(pos_search.dimension_ref.0) else {
            error!(target: "pos_search", "No root oplist found for dimension {:?}", pos_search.dimension_ref);
            search_failed_evs.push(SearchFailed(pos_search.requester));
            continue;
        };
        inputs.push(TerrGenSearchTaskInput { probe: pos_search, templ, root_oplist });
    }
    let found = found_suitable_positions.drain().collect::<EntityHashSet>();
    let task_pool = AsyncComputeTaskPool::get();
    terrgen_tasks.search_tasks.push(task_pool.spawn(async move {
        process_search_batch(inputs, found)
    }));
}


fn process_search_batch(inputs: Vec<TerrGenSearchTaskInput>, successful_requesters: EntityHashSet) -> TerrGenSearchTaskResult {
    let pending_count = inputs.len();
    let mut new_pending_ops: Vec<PendingOp> = Vec::with_capacity(pending_count);
    let mut new_pos_searches: Vec<TerrProbeJob> = Vec::with_capacity(pending_count);
    let mut search_failed: Vec<Entity> = Vec::with_capacity(pending_count);

    for input in inputs {
        let pos_search = input.probe;

        if successful_requesters.contains(&pos_search.requester) {
            info!(target: "pos_search","Found suitable position for {:?}", pos_search.requester);
            continue;
        }
        let templ = input.templ;
        let curr_iteration_batch_i = pos_search.curr_iteration_batch_i;
        let root_oplist = input.root_oplist;
        let curr_iteration_batch_i = curr_iteration_batch_i.max(0);

        match templ.probe_pattern.clone() {
            ProbePattern::Spiral(_, _, _, _, _) => {
                process_spiral_pattern(
                    pos_search,
                    &templ,
                    root_oplist,
                    curr_iteration_batch_i,
                    &mut new_pending_ops,
                    &mut new_pos_searches,
                    &mut search_failed,
                );
            }
            ProbePattern::Concentric {
                radius_step,
                sample_spacing,
            } => {
                process_concentric_pattern(
                    pos_search,
                    &templ,
                    root_oplist,
                    radius_step,
                    sample_spacing,
                    curr_iteration_batch_i,
                    &mut new_pending_ops,
                    &mut new_pos_searches,
                    &mut search_failed,
                );
            }
            ProbePattern::Chunk(chunk_pos) => {
                process_chunk_pattern(
                    pos_search,
                    &templ,
                    root_oplist,
                    chunk_pos,
                    curr_iteration_batch_i,
                    &mut new_pending_ops,
                    &mut new_pos_searches,
                    &mut search_failed,
                );
            }
            ProbePattern::Region(spacing) => {
                process_region_pattern(
                    pos_search,
                    &templ,
                    root_oplist,
                    spacing,
                    curr_iteration_batch_i,
                    &mut new_pending_ops,
                    &mut new_pos_searches,
                    &mut search_failed,
                );
            }
        }
    }

    TerrGenSearchTaskResult {
        new_pending_ops,
        new_pos_searches,
        search_failed,
    }
}
