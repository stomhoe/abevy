use bevy::{ecs::entity::EntityHashMap, prelude::*};
use common::{
    common_components::{HashId, HashIdMap},
    common_tag_components::{HashedTagsVec, TagSet},
};
use std::sync::Arc;

use crate::{
    chunking::macro_chunk_components::BiomeTagWeightAtMacrochunk,
    terrain::{
        operation_list::operation_list_components::*,
        terrprobe::{opfilter::opfilter_components::OpFilter, terrprobe_messages::*},
        terrgen_async_resources::*,
        terrgen_components::*,
        terrgen_messages::{ChunkTerrainBuilt, PendingOp, PendingOpInput, PendingOpPurpose},
        terrgen_resources::{TerrGenSharedTaskData, TerrGenSharedTaskDataInner, *},
    },
};
use ::tilemap_shared::*;

#[derive(Clone)]
pub(crate) struct TerrGenTaskContext {
    shared: Arc<TerrGenSharedTaskDataInner>,
}

#[derive(Clone)]
struct EvalFrame {
    oplist: HashId,
    gpos: GlobalTilePos,
    oplist_size: OplistSize,
    variables: HashIdMap<f32>,
}

#[allow(unused_parens, )]
pub fn init_terrgen_shared_task_data(
    mut shared_task_data: ResMut<TerrGenSharedTaskData>,
    oplist_query: Query<
        (
            &HashId,
            &OperationList,
            &OplistSize,
            Option<&HashedTagsVec>,
            Option<&TagSet>,
        ),
        (),
    >,
    fnl_noises: Query<(&HashId, &FnlNoiseComp), ()>,
    op_filters: Query<(&HashId, &OpFilter), ()>,
) {
    let oplists_count = oplist_query.iter().count();
    let noises_count = fnl_noises.iter().count();
    let filters_count = op_filters.iter().count();
    let any_oplist_has_tags = oplist_query
        .iter()
        .any(|(_, _, _, oplist_tags_opt, oplist_tagset_opt, )| {
            oplist_tags_opt.is_some() || oplist_tagset_opt.is_some()
        });
    if let Some(shared) = shared_task_data.shared.as_ref()
        && shared.oplists.len() == oplists_count
        && shared.noises.len() == noises_count
        && shared.filters.len() == filters_count
        && (!any_oplist_has_tags || !shared.oplist_tags.is_empty())
    {
        return;
    }
    if oplists_count == 0 || noises_count == 0 {
        return;
    }

    let mut oplists: HashIdMap<OperationList> = HashIdMap::default();
    let mut oplist_debug_var_ids: HashIdMap<Vec<HashId>> = HashIdMap::default();
    let mut oplist_sizes: HashIdMap<OplistSize> = HashIdMap::default();
    let mut oplist_tags: HashIdMap<HashedTagsVec> = HashIdMap::default();
    for (&oplist_hash, oplist, &oplist_size, oplist_tags_opt, oplist_tagset_opt) in
        oplist_query.iter()
    {
        let _ = oplists.overwrite(oplist_hash, oplist.clone());
        let _ = oplist_debug_var_ids.overwrite(
            oplist_hash,
            oplist.hash_ids_mapped_to_strids.keys().copied().collect::<Vec<_>>(),
        );
        let _ = oplist_sizes.overwrite(oplist_hash, oplist_size);
        let tags_to_cache = oplist_tags_opt
            .cloned()
            .or_else(|| oplist_tagset_opt.map(HashedTagsVec::from));
        if let Some(tags) = tags_to_cache {
            let _ = oplist_tags.overwrite(oplist_hash, tags);
        };
    }

    let mut noises: HashIdMap<FnlNoiseComp> = HashIdMap::default();
    for (noise_hash, noise) in fnl_noises.iter() {
        let _ = noises.overwrite(*noise_hash, noise.clone());
    }

    let mut filters: HashIdMap<OpFilter> = HashIdMap::default();
    for (&filter_hash, filter) in op_filters.iter() {
        let _ = filters.overwrite(filter_hash, filter.clone());
    }

    shared_task_data.shared = Some(Arc::new(TerrGenSharedTaskDataInner {
        oplists,
        oplist_debug_var_ids,
        oplist_sizes,
        oplist_tags,
        noises,
        filters,
    }));
}

impl TerrGenTaskContext {
    pub(crate) fn from_shared(shared: &Arc<TerrGenSharedTaskDataInner>) -> Self {
        Self {
            shared: Arc::clone(shared),
        }
    }
}

pub(crate) fn process_pending_ops_batch(
    pending_ops: Vec<PendingOp>,
    context: TerrGenTaskContext,
    gen_settings: GlobalGenSettings,
    capture_debug: bool,
) -> TerrGenOpTaskResult {
    use crate::terrain::terrgen_expression::EvalContext;

    let mut result = TerrGenOpTaskResult::default();
    let mut pending_queue = pending_ops;
    let mut emitted_per_probe: EntityHashMap<u32> = EntityHashMap::new();
    let mut last_success_idx_for_requester: EntityHashMap<usize> = EntityHashMap::new();
    let mut sampled_matrices_by_requester: EntityHashMap<SampledValues> = EntityHashMap::new();

    while let Some(ev) = pending_queue.pop() {
        upsert_sample_matrix_for_pending(&ev, &mut sampled_matrices_by_requester);
        let root_oplist_hash = ev.oplist.0;
        if !context.shared.oplists.contains_key(root_oplist_hash) {
            error!(target: "terrgen_systems", "Oplist hash {:?} not found in terrgen_process_pending_ops", root_oplist_hash);
            continue;
        }
        let Ok(&my_oplist_size) = context.shared.oplist_sizes.get(root_oplist_hash) else {
            error!(target: "terrgen_systems", "OplistSize not found for oplist hash {:?}", root_oplist_hash);
            continue;
        };

        let dimension_hash = ev.dimension_ref().0;
        let filtered_op = ev.filtered_op();
        let filter = if filtered_op != HashId::default() {
            context.shared.filters.get_opt(filtered_op)
        } else {
            None
        };
        let has_filter = filtered_op != HashId::default();

        let mut frame_stack = vec![EvalFrame {
            oplist: root_oplist_hash,
            gpos: ev.gpos(),
            oplist_size: my_oplist_size,
            variables: HashIdMap::new(),
        }];

        while let Some(mut frame) = frame_stack.pop() {
            let Ok(oplist) = context.shared.oplists.get(frame.oplist) else { continue; };
            if oplist.bifurcations.is_empty() {
                continue;
            }
            let eval_context = EvalContext {
                global_pos: frame.gpos,
                dimension_hash,
                gen_settings: &gen_settings,
                oplist_size: frame.oplist_size,
                noises: &context.shared.noises,
                variables: &frame.variables,
            };
            let (output_value, computed_vars) = oplist.expr_tree.eval(&frame.variables, &eval_context);
            frame.variables = computed_vars;
            if capture_debug {
                push_debug_sample(
                    &mut result,
                    &context,
                    ev.dimension_ref(),
                    frame.gpos,
                    frame.oplist,
                    output_value,
                    &frame.variables,
                );
            }

            let destination_i = (output_value as usize).min(oplist.bifurcations.len() - 1);
            try_emit_filter_match(
                &ev,
                &context,
                frame.oplist,
                frame.gpos,
                output_value,
                &frame.variables,
                filter,
                has_filter,
                &mut emitted_per_probe,
                &mut last_success_idx_for_requester,
                &mut sampled_matrices_by_requester,
                &mut result,
            );

            let Some(bifurcation) = oplist.bifurcations.get(destination_i) else { continue; };
            collect_branch_outputs(
                &mut result,
                &ev,
                frame.oplist,
                frame.gpos,
                frame.oplist_size,
                dimension_hash,
                &bifurcation.biome_tags,
                &bifurcation.tiles,
            );

            if let Some(child_oplist_hash) = bifurcation.oplist
                && let Ok(&child_oplist_size) = context.shared.oplist_sizes.get(child_oplist_hash)
            {
                spawn_bifurcation_frames(&mut frame_stack, &frame, child_oplist_hash, child_oplist_size);
            }
        }
        mark_pending_op_complete(&ev, &mut result);
    }

    for (_, sample_idx) in last_success_idx_for_requester.drain() {
        let Some(sample) = result.sampled_value_events.get_mut(sample_idx) else { continue; };
        sample.is_last = true;
    }
    for (requester, matrix) in sampled_matrices_by_requester.drain() {
        result.sampled_value_matrix_events.push(SampledValuesCollected {
            requester,
            matrix,
        });
    }
    result
}

pub(crate) fn pending_root_gpos_count_for_chunk(work: &TerrGenLaunchWork) -> usize {
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
        }
    }
    count
}

pub(crate) fn register_completed_chunk_gpos(
    completed_chunk_gpos: &[(Entity, GlobalTilePos)],
    expected_root_gpos_by_chunk: &mut EntityHashMap<usize>,
    completed_root_gpos_by_chunk: &mut EntityHashMap<ChunkGposMask>,
    chunk_built_msgs: &mut Vec<ChunkTerrainBuilt>,
) {
    let mut chunks_built = Vec::new();
    for &(chunk_ent, gpos) in completed_chunk_gpos {
        let Some(&expected_count) = expected_root_gpos_by_chunk.get(&chunk_ent) else {
            continue;
        };
        let completed_gpos = completed_root_gpos_by_chunk.entry(chunk_ent).or_default();
        completed_gpos.set_gpos(ChunkPos::from(gpos), gpos);
        if completed_gpos.count_set() < expected_count {
            continue;
        }
        chunks_built.push(chunk_ent);
    }
    for chunk_ent in chunks_built {
        expected_root_gpos_by_chunk.remove(&chunk_ent);
        completed_root_gpos_by_chunk.remove(&chunk_ent);
        chunk_built_msgs.push(ChunkTerrainBuilt { chunk_ent });
    }
}

fn push_debug_sample(
    result: &mut TerrGenOpTaskResult,
    context: &TerrGenTaskContext,
    dimension_ref: DimensionRef,
    gpos: GlobalTilePos,
    oplist: HashId,
    output: f32,
    variables: &HashIdMap<f32>,
) {
    let mut debug_values = HashIdMap::new();
    if let Ok(var_ids) = context.shared.oplist_debug_var_ids.get(oplist) {
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

fn try_emit_filter_match(
    source_ev: &PendingOp,
    context: &TerrGenTaskContext,
    source_oplist: HashId,
    gpos: GlobalTilePos,
    output_value: f32,
    computed_vars: &HashIdMap<f32>,
    filter: Option<&OpFilter>,
    has_filter: bool,
    emitted_per_probe: &mut EntityHashMap<u32>,
    last_success_idx_for_requester: &mut EntityHashMap<usize>,
    sampled_matrices_by_requester: &mut EntityHashMap<SampledValues>,
    result: &mut TerrGenOpTaskResult,
) {
    if !has_filter {
        return;
    }
    let Some(filter) = filter else {
        return;
    };
    let Ok(oplist_tags) = context.shared.oplist_tags.get(source_oplist) else {
        return;
    };
    if !oplist_tags.intersects(&filter.tags) {
        return;
    }
    if !passes_filter_value(Some(filter), computed_vars, output_value) {
        return;
    }
    if source_ev.matrix_spec().is_some() {
        let sampled_value = sampled_value_from_filter(Some(filter), computed_vars, output_value);
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

fn collect_branch_outputs(
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
    match source_ev.purpose {
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
                    purpose: source_ev.purpose,
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
                macro_chunk_ent,
                sample_chunk_pos: ChunkPos::from(gpos),
                biome_tags: biome_tags.to_vec(),
            });
        }
        PendingOpPurpose::ValueProbe(_) => {}
    }
}

fn child_positions(
    gpos: GlobalTilePos,
    parent_size: OplistSize,
    child_size: OplistSize,
) -> Vec<GlobalTilePos> {
    if parent_size <= child_size {
        return if gpos.0.abs().as_uvec2() % child_size.inner() == UVec2::ZERO {
            vec![gpos]
        } else {
            Vec::new()
        };
    }
    let x_end = parent_size.x() as i32 / child_size.x() as i32;
    let y_end = parent_size.y() as i32 / child_size.y() as i32;
    let mut positions = Vec::with_capacity((x_end * y_end) as usize);
    for x in 0..x_end {
        for y in 0..y_end {
            positions.push(gpos + GlobalTilePos::new(x, y));
        }
    }
    positions
}

#[inline]
fn passes_filter_value(
    filter: Option<&OpFilter>,
    computed_vars: &HashIdMap<f32>,
    output_value: f32,
) -> bool {
    let Some(filter) = filter else {
        return false;
    };

    if let Some(var_name_hash) = filter.var_name_hash {
        let Ok(val) = computed_vars.get(var_name_hash) else {
            return false;
        };
        return (filter.min_val..=filter.max_val).contains(val);
    }

    (filter.min_val..=filter.max_val).contains(&output_value)
}

#[inline]
fn sampled_value_from_filter(
    filter: Option<&OpFilter>,
    computed_vars: &HashIdMap<f32>,
    output_value: f32,
) -> f32 {
    if let Some(filter) = filter
        && let Some(var_name_hash) = filter.var_name_hash
        && let Ok(val) = computed_vars.get(var_name_hash)
    {
        return *val;
    }
    output_value
}

fn upsert_sample_matrix_for_pending(
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

fn set_sample_matrix_value_for_pending(
    source_ev: &PendingOp,
    value: Option<f32>,
    sampled_matrices_by_requester: &mut EntityHashMap<SampledValues>,
) {
    let Some(_) = source_ev.matrix_spec() else {
        return;
    };
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

fn spawn_bifurcation_frames(
    frames: &mut Vec<EvalFrame>,
    frame: &EvalFrame,
    child_oplist: HashId,
    child_oplist_size: OplistSize,
) {
    for gpos in child_positions(frame.gpos, frame.oplist_size, child_oplist_size) {
        frames.push(EvalFrame {
            oplist: child_oplist,
            gpos,
            oplist_size: child_oplist_size,
            variables: frame.variables.clone(),
        });
    }
}

fn mark_pending_op_complete(source_ev: &PendingOp, result: &mut TerrGenOpTaskResult) {
    match source_ev.purpose {
        PendingOpPurpose::ChunkTerrainGen { chunk_ent } => {
            if chunk_ent == Entity::PLACEHOLDER {
                return;
            }
            result.completed_chunk_gpos.push((chunk_ent, source_ev.gpos()));
        }
        PendingOpPurpose::MacroChunkBiomeSampling { macro_chunk_ent } => {
            result.completed_macro_chunk_biome_samples.push(macro_chunk_ent);
        }
        PendingOpPurpose::ValueProbe(_) => {}
    }
}
