use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap, prelude::*};
use ::common::*;
use std::sync::Arc;

use crate::{
    chunking::macro_chunk_components::BiomeTagWeightAtMacrochunk,
    terrain::{
        terrprobe::{opfilter::opfilter_components::OpFilter, terrprobe_messages::*},
        terrgen_async_resources::*,
        terrgen_messages::{ChunkTerrainBuilt, PendingOp, PendingOpInput, PendingOpPurpose},
        terrgen_resources::{TerrGenSharedTaskDataInner, *},
    },
};
use ::tilemap_shared::*;

pub(crate) type TerrGenTaskContext = Arc<TerrGenSharedTaskDataInner>;

#[derive(Clone)]
pub(crate) struct EvalFrame {
    pub(crate) oplist: HashId,
    pub(crate) gpos: GlobalTilePos,
    pub(crate) oplist_size: OplistSize,
    pub(crate) variables: HashIdMap<f32>,
}

impl EvalFrame {
    pub fn spawn_bifurcation_frames(
        &self,
        frames: &mut Vec<EvalFrame>,
        child_oplist: HashId,
        child_oplist_size: OplistSize,
    ) {
        for gpos in self.oplist_size.child_positions(self.gpos, child_oplist_size) {
            frames.push(EvalFrame {
                oplist: child_oplist,
                gpos,
                oplist_size: child_oplist_size,
                variables: self.variables.clone(),
            });
        }
    }
}

pub(crate) fn pending_root_gpos_count_for_chunk(work: &ChunkTerrGenWork) -> usize {
    for_each_root_gpos(work, |_| {})
}

pub(crate) fn for_each_root_gpos(work: &ChunkTerrGenWork, mut callback: impl FnMut(GlobalTilePos)) -> usize {
    let mut count = 0usize;
    for x in 0..ChunkPos::CHUNK_SIZE.x / work.oplist_size.x() {
        for y in 0..ChunkPos::CHUNK_SIZE.y / work.oplist_size.y() {
            let pos_within_chunk = IVec2::new(x as i32, y as i32);
            let gpos = work.chunk_pos.to_tilepos()
                + GlobalTilePos(pos_within_chunk * work.oplist_size.inner().as_ivec2());
            let Some(bit_idx) = work.chunk_pos.bit_index_in_chunk(gpos) else {
                continue;
            };
            if work.blocked_gpos.is_set(bit_idx) {
                continue;
            }
            count += 1;
            callback(gpos);
        }
    }
    count
}

pub(crate) fn register_completed_chunk_gpos(
    completed_chunk_gpos: &[((DimensionRef, ChunkPos), GlobalTilePos)],
    expected_root_gpos_by_chunk: &mut HashMap<(DimensionRef, ChunkPos), usize>,
    completed_root_gpos_by_chunk: &mut HashMap<(DimensionRef, ChunkPos), ChunkGposMask>,
    chunk_built_msgs: &mut Vec<ChunkTerrainBuilt>,
) {
    let mut chunks_built = Vec::new();
    for &((dim_ref, chunk_pos), gpos) in completed_chunk_gpos {
        let Some(&expected_count) = expected_root_gpos_by_chunk.get(&(dim_ref, chunk_pos)) else {
            continue;
        };
        let completed_gpos = completed_root_gpos_by_chunk.entry((dim_ref, chunk_pos)).or_default();
        completed_gpos.set_gpos(chunk_pos, gpos);
        if completed_gpos.count_set() < expected_count {
            continue;
        }
        chunks_built.push((dim_ref, chunk_pos));
    }
    for (dim_ref, chunk_pos) in chunks_built {
        expected_root_gpos_by_chunk.remove(&(dim_ref, chunk_pos));
        completed_root_gpos_by_chunk.remove(&(dim_ref, chunk_pos));
        chunk_built_msgs.push(ChunkTerrainBuilt { dimension_ref: dim_ref, chunk_pos });
    }
}

pub(crate) fn push_debug_sample(
    result: &mut TerrGenOpTaskResult,
    context: &TerrGenTaskContext,
    dimension_ref: DimensionRef,
    gpos: GlobalTilePos,
    oplist: HashId,
    output: f32,
    variables: &HashIdMap<f32>,
) {
    let mut debug_values = HashIdMap::new();
    if let Ok(var_ids) = context.oplist_debug_var_ids.get(oplist) {
        for var_id in var_ids {
            let Ok(value) = variables.get(*var_id) else { continue; };
            let _ = debug_values.overwrite(*var_id, *value);
        }
    }
    result.debug_samples.push(TerrGenDebugSample {
        dimension_ref,
        gpos,
        oplist,
        oplist_id: oplist,
        output,
        variables: debug_values,
    });
}

pub(crate) fn try_emit_filter_match(
    source_ev: &PendingOp,
    context: &TerrGenTaskContext,
    source_oplist: HashId,
    gpos: GlobalTilePos,
    output_value: f32,
    computed_vars: &HashIdMap<f32>,
    filter: Option<&OpFilter>,
    emitted_per_probe: &mut EntityHashMap<u32>,
    last_success_idx_for_requester: &mut EntityHashMap<usize>,
    sampled_matrices_by_requester: &mut EntityHashMap<SampledValues>,
    result: &mut TerrGenOpTaskResult,
) {
    let Some(filter) = filter else {
        return;
    };
    let Ok(oplist_tags) = context.oplist_tags.get(source_oplist) else {
        return;
    };
    if !oplist_tags.intersects(&filter.tags) {
        return;
    }
    if !filter.passes_filter_value(computed_vars, output_value) {
        return;
    }
    if source_ev.matrix_spec().is_some() {
        let sampled_value = filter.sampled_value_from_filter(computed_vars, output_value);
        set_sample_matrix_value_for_pending(source_ev, Some(sampled_value), sampled_matrices_by_requester);
        return;
    }
    let requester = source_ev.requester();
    let emitted = emitted_per_probe.entry(requester).or_insert(0);
    if *emitted >= source_ev.max_emitted_results() {
        return;
    }
    result.sampled_value_events.push(SuitablePosFound {
        requester,
        val: output_value,
        found_pos: gpos,
        is_last: false,
    });
    if source_ev.mark_last_success_in_batch() {
        last_success_idx_for_requester.insert(requester, result.sampled_value_events.len() - 1);
    }
    *emitted = emitted.saturating_add(1);
}

pub(crate) fn collect_branch_outputs(
    result: &mut TerrGenOpTaskResult,
    source_ev: &PendingOp,
    source_oplist: HashId,
    gpos: GlobalTilePos,
    oplist_size: OplistSize,
    dimension_hash: HashId,
    biome_tags: &[BiomeTagWeightAtMacrochunk],
    tiles: &[HashId],
) {
    if !source_ev.filtered_op_is_placeholder() {
        return;
    }
    match &source_ev.purpose {
        PendingOpPurpose::ChunkTerrainGen { .. } => {
            if tiles.is_empty() {
                return;
            }
            result.tile_requests.push(TerrGenTileRequest {
                bif_tiles: tiles.to_vec(),
                pending: PendingOp {
                    oplist: DimensionRootOplist(source_oplist),
                    input: PendingOpInput {
                        dimension_ref: source_ev.dimension_ref(),
                        gpos,
                    },
                    purpose: source_ev.purpose.clone(),
                },
                oplist_size,
                dimension_hash,
            });
        }
        PendingOpPurpose::MacroChunkBiomeSampling { macro_chunk_ent } => {
            if biome_tags.is_empty() {
                return;
            }
            result.biome_tag_samples.push(TerrGenBiomeTagSample {
                macro_chunk_ent: *macro_chunk_ent,
                sample_chunk_pos: ChunkPos::from(gpos),
                biome_tags: biome_tags.to_vec(),
            });
        }
        PendingOpPurpose::ValueProbe(_) => {}
    }
}

pub(crate) fn upsert_sample_matrix_for_pending(
    source_ev: &PendingOp,
    sampled_matrices_by_requester: &mut EntityHashMap<SampledValues>,
) {
    let Some(spec) = source_ev.matrix_spec() else {
        return;
    };
    sampled_matrices_by_requester
        .entry(source_ev.requester())
        .or_insert_with(|| SampledValues::new(spec.min, spec.matrix_size, spec.spacing));
}

pub(crate) fn set_sample_matrix_value_for_pending(
    source_ev: &PendingOp,
    value: Option<f32>,
    sampled_matrices_by_requester: &mut EntityHashMap<SampledValues>,
) {
    let Some(matrix) = sampled_matrices_by_requester.get_mut(&source_ev.requester()) else {
        return;
    };
    if value.is_none() {
        let _ = matrix.set(source_ev.gpos(), None);
        return;
    }
    let Some(current) = matrix.get(source_ev.gpos()) else {
        return;
    };
    if current.is_none() {
        let _ = matrix.set(source_ev.gpos(), value);
    }
}
