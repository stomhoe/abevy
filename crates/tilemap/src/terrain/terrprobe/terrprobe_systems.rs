use bevy::{ecs::entity::{EntityHashSet, EntityHashMap}, prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use common::common_components::StrId20B;
use game_common::game_common_components::EntityZeroRef;
use std::f32::consts::PI;
use std::collections::{HashMap, HashSet};
use crate::terrain::{
    terrprobe::opfilter::opfilter_components::OpFilter,
    terrprobe::terrprobe_components::TerrProbeTempl,
    terrprobe::terrprobe_messages::*,
    terrgen_components::FailedSearchOplistFilterHolder,
    terrgen_messages::PendingOp,
    terrgen_resources::{TerrGenAsyncTasks, TerrGenSearchTaskResult},
};
use crate::regioning::{
    regioning_components::{GridOfSgcs, Region, RegionState},
    regioning_resources::LoadedRegions,
};
use ::tilemap_shared::*;

#[derive(Clone)]
struct TerrGenSearchTaskInput {
    probe: TerrProbeJob,
    templ: Option<TerrProbeTempl>,
    opfilter: Option<OpFilter>,
    root_oplist: Option<DimensionRootOplist>,
    region_structures: HashMap<(Entity, RegionPos), GridOfSgcs>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct SearchRegioningParams<'w, 's> {
    pub loaded_regions: ResMut<'w, LoadedRegions>,
    pub region_query: Query<'w, 's, (&'static DimensionRef, &'static RegionPos, &'static GridOfSgcs, &'static RegionState), ()>,
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





#[allow(unused_parens)]
//input: PosSearch messages. output: SearchFailed or SuitablePosFound(emitted in produce_tiles)
pub fn search_suitable_positions(
    mut cmd: Commands,
    mut terrain_probe: ResMut<Messages<TerrProbeJob>>, mut mwriter_search_failed: MessageWriter<SearchFailed>,
    mut mwriter_pending_ops: MessageWriter<PendingOp>, mut mreader_suitable_pos_found: MessageReader<SuitablePosFound>,
    terrprobe_query: Query<&TerrProbeTempl>,
    studied_ops: Query<&OpFilter, ( )>,
    dimensions_query: Query<&DimensionRootOplist>,
    mut regioning: SearchRegioningParams,
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

    let mut region_structures: HashMap<(Entity, RegionPos), GridOfSgcs> = HashMap::with_capacity(regioning.loaded_regions.0.len());
    for (&(dim_ref, region_pos), &region_ent) in regioning.loaded_regions.0.iter() {
        let Ok((&region_dim_ref, &actual_region_pos, grid_of_sgcs, state)) = regioning.region_query.get(region_ent) else {
            continue;
        };
        if *state == RegionState::OfferingChunks {
            continue;
        }
        if region_dim_ref != dim_ref || actual_region_pos != region_pos {
            continue;
        }
        region_structures.insert((dim_ref.0, region_pos), grid_of_sgcs.clone());
    }

    let mut inputs = Vec::with_capacity(terrain_probe.len());
    for pos_search in terrain_probe.drain() {
        let templ = terrprobe_query.get(pos_search.templ_ent).ok().cloned();
        let Some(templ_ref) = templ.as_ref() else {
            inputs.push(TerrGenSearchTaskInput {
                probe: pos_search,
                templ,
                opfilter: None,
                root_oplist: None,
                region_structures: region_structures.clone(),
            });
            continue;
        };
        let mut requires_region_wait = false;
        for region_pos in crossed_regions_for_probe_batch(&pos_search, templ_ref) {
            let key = (pos_search.dimension_ref, region_pos);
            if regioning.loaded_regions.0.get(&key).is_none() {
                let region_ent = cmd.spawn((
                    region_pos,
                    Region,
                    StrId20B::trunc(format!("Region({}, {})", region_pos.0.x, region_pos.0.y)),
                    Transform::default(),
                    ChildOf(pos_search.dimension_ref.0),
                    pos_search.dimension_ref,
                )).id();
                regioning.loaded_regions.0.insert(key, region_ent);
                requires_region_wait = true;
                continue;
            }
            if !region_structures.contains_key(&(pos_search.dimension_ref.0, region_pos)) {
                requires_region_wait = true;
            }
        }
        if requires_region_wait {
            new_pos_searches.push(pos_search);
            continue;
        }
        let opfilter = templ_ref
            .opfilter_override
            .clone()
            .or_else(|| {
                let ent = templ_ref.opfilter_ent?;
                studied_ops.get(ent).ok().cloned()
            });
        let root_oplist = dimensions_query.get(pos_search.dimension_ref.0).ok().copied();
        inputs.push(TerrGenSearchTaskInput {
            probe: pos_search,
            templ,
            opfilter,
            root_oplist,
            region_structures: region_structures.clone(),
        });
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
        let (requester, step_size, curr_iteration_batch_i, iterations_per_batch, max_batches, dimension_ref) =
            (
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
                error!(target: "pos_search", "No OpFilter found in search_suitable_position, giving up");
                search_failed.push(requester);
            }
            continue;
        };
        let curr_iteration_batch_i = curr_iteration_batch_i.max(0);
        let filtered_op = templ.opfilter_ent.unwrap_or(Entity::PLACEHOLDER);
        let opfilter_override = templ.opfilter_override.clone().or(Some(opfilter.clone()));

        match templ.probe_pattern.clone() {
            ProbePattern::Radial(_) => {
                let is_candidate_allowed = |candidate_pos: GlobalTilePos| -> bool {
                    let chunk_pos = candidate_pos.to_chunkpos();
                    let region_pos = chunk_pos.to_region_pos();
                    let structure_here = input
                        .region_structures
                        .get(&(dimension_ref.0, region_pos))
                        .and_then(|grid| grid.sampled_structure_at_gpos(candidate_pos, region_pos));
                    if !pos_search.structuregen_whitelist.is_empty()
                        && structure_here.is_some_and(|sgc| !pos_search.structuregen_whitelist.contains(&sgc))
                    {
                        return false;
                    }
                    if pos_search.structuregen_blacklist.iter().any(|forbidden| structure_here.is_some_and(|sgc| *forbidden == sgc)) {
                        return false;
                    }
                    if !templ.sgc_required_tile_tags.is_empty() && templ.sgc_admitted_tiles_as_found_pos.is_empty() {
                        return false;
                    }
                    true
                };
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
                        let candidate_pos = calculate_pos(i_within_batch, angle);
                        if !is_candidate_allowed(candidate_pos) {
                            continue;
                        }
                        new_pending_ops.push(PendingOp {
                            oplist: root_oplist,
                            dimension_ref,
                            gpos: candidate_pos,
                            filtered_op,
                            opfilter_override: opfilter_override.clone(),
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
                let is_candidate_allowed = |candidate_pos: GlobalTilePos| -> bool {
                    let chunk_pos = candidate_pos.to_chunkpos();
                    let region_pos = chunk_pos.to_region_pos();
                    let structure_here = input
                        .region_structures
                        .get(&(dimension_ref.0, region_pos))
                        .and_then(|grid| grid.sampled_structure_at_gpos(candidate_pos, region_pos));
                    if !pos_search.structuregen_whitelist.is_empty()
                        && structure_here.is_some_and(|sgc| !pos_search.structuregen_whitelist.contains(&sgc))
                    {
                        return false;
                    }
                    if pos_search.structuregen_blacklist.iter().any(|forbidden| structure_here.is_some_and(|sgc| *forbidden == sgc)) {
                        return false;
                    }
                    if !templ.sgc_required_tile_tags.is_empty() && templ.sgc_admitted_tiles_as_found_pos.is_empty() {
                        return false;
                    }
                    true
                };
                trace!(target: "pos_search", "Spiral search started at pos {:?}, dir_vec {:?}, curr_length_in_dir {}, turns {}",
                    pos, dir_vec, curr_length_in_dir, turn_parity);

                for _ in 0..iterations_per_batch {
                    pos = pos + GlobalTilePos(dir_vec.saturating_mul(IVec2::splat(step_size as i32)));
                    if !is_candidate_allowed(pos) {
                        steps_taken += 1;
                        if steps_taken >= curr_length_in_dir {
                            steps_taken = 0;
                            dir_vec = dir_vec.perp();
                            curr_length_in_dir = curr_length_in_dir.saturating_add(turn_parity as u64);
                            turn_parity = !turn_parity;
                        }
                        continue;
                    }
                    new_pending_ops.push(PendingOp {
                        dimension_ref,
                        oplist: root_oplist,
                        gpos: pos,
                        filtered_op,
                        opfilter_override: opfilter_override.clone(),
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

fn crossed_regions_for_probe_batch(pos_search: &TerrProbeJob, templ: &TerrProbeTempl) -> HashSet<RegionPos> {
    let mut crossed_regions = HashSet::default();
    let curr_iteration_batch_i = pos_search.curr_iteration_batch_i.max(0);
    match templ.probe_pattern.clone() {
        ProbePattern::Radial(_) => {
            if curr_iteration_batch_i as u16 >= templ.max_batches {
                return crossed_regions;
            }
            let start_i_within_batch = (curr_iteration_batch_i == 0) as u16;
            for i in 0..8 {
                let angle = 2.0 * PI * (i as f32) / 8.0;
                for i_within_batch in start_i_within_batch..templ.iterations_per_batch {
                    let global_i = (curr_iteration_batch_i as u16 * templ.iterations_per_batch + i_within_batch) as f32 * templ.step_size as f32;
                    let candidate_pos = pos_search.search_start_pos + GlobalTilePos::from(IVec2::new(
                        (global_i * angle.cos()) as i32,
                        (global_i * angle.sin()) as i32,
                    ));
                    crossed_regions.insert(candidate_pos.to_chunkpos().to_region_pos());
                }
            }
        }
        ProbePattern::Spiral(_, _, _, _, _) => {
            let (mut curr_length_in_dir, mut steps_taken, mut dir_vec, mut pos, mut turn_parity) = spiral_state_at_batch_start(
                pos_search.search_start_pos,
                templ.step_size,
                curr_iteration_batch_i as u16,
                templ.iterations_per_batch,
            );
            for _ in 0..templ.iterations_per_batch {
                pos = pos + GlobalTilePos(dir_vec.saturating_mul(IVec2::splat(templ.step_size as i32)));
                crossed_regions.insert(pos.to_chunkpos().to_region_pos());
                steps_taken += 1;
                if steps_taken >= curr_length_in_dir {
                    steps_taken = 0;
                    dir_vec = dir_vec.perp();
                    curr_length_in_dir = curr_length_in_dir.saturating_add(turn_parity as u64);
                    turn_parity = !turn_parity;
                }
            }
        }
    }
    crossed_regions
}
