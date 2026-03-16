use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, platform::collections::HashSet, prelude::*};
use common::{common_components::{HashId, HashIdMap, StrId}, common_tag_components::HashedTagsVec};
use debug_unwraps::DebugUnwrapExt;

use crate::{
    chunking::macro_chunk_components::BiomeTagWeightAtMacroChunk,
    terrain::{
        operation_list::operation_list_components::*,
        terrprobe::{opfilter::opfilter_components::OpFilter, terrprobe_messages::*},
        terrgen_async_resources::*,
        terrgen_components::*,
        terrgen_messages::{ChunkTerrainBuilt, PendingOp, PendingOpInput, PendingOpPurpose},
        terrgen_resources::*,
    },
};
use ::tilemap_shared::*;

#[derive(Clone)]
pub(crate) struct TerrGenTaskContext {
    oplists: EntityHashMap<OperationList>,
    oplist_ids: EntityHashMap<HashId>,
    oplist_debug_var_ids: EntityHashMap<Vec<HashId>>,
    oplist_sizes: EntityHashMap<OplistSize>,
    oplist_tags: EntityHashMap<Option<HashedTagsVec>>,
    child_oplist_sizes: EntityHashMap<EntityHashMap<OplistSize>>,
    noises: EntityHashMap<FnlNoiseComp>,
    filters: EntityHashMap<OpFilter>,
    dimension_hashes: EntityHashMap<HashId>,
}

#[derive(Clone)]
struct EvalFrame {
    oplist: Entity,
    gpos: GlobalTilePos,
    oplist_size: OplistSize,
    variables: HashIdMap<f32>,
}

pub(crate) fn build_terrgen_task_context(
    pending_ops: &[PendingOp],
    oplist_query: &Query<(&OperationList, &OplistSize, Option<&HashedTagsVec>, &StrId), ()>,
    fnl_noises: &Query<&FnlNoiseComp>,
    op_filters: &Query<&OpFilter>,
    dim_hash_query: &Query<&HashId, common::AnyDisabling>,
) -> TerrGenTaskContext {
    let pending_len = pending_ops.len();
    let mut oplists: EntityHashMap<OperationList> = EntityHashMap::with_capacity(pending_len);
    let mut oplist_ids: EntityHashMap<HashId> = EntityHashMap::with_capacity(pending_len);
    let mut oplist_debug_var_ids: EntityHashMap<Vec<HashId>> = EntityHashMap::with_capacity(pending_len);
    let mut oplist_sizes: EntityHashMap<OplistSize> = EntityHashMap::with_capacity(pending_len);
    let mut oplist_tags: EntityHashMap<Option<HashedTagsVec>> = EntityHashMap::with_capacity(pending_len);
    let mut child_oplist_sizes: EntityHashMap<EntityHashMap<OplistSize>> = EntityHashMap::with_capacity(pending_len);
    let mut noise_entities: EntityHashSet = EntityHashSet::with_capacity(pending_len);

    let mut to_visit: Vec<Entity> = pending_ops.iter().map(|ev| ev.oplist.0).collect();
    let mut visited: EntityHashSet = EntityHashSet::with_capacity(pending_len);

    while let Some(oplist_ent) = to_visit.pop() {
        if !visited.insert(oplist_ent) {
            continue;
        }

        let Ok((oplist, &oplist_size, oplist_tags_opt, oplist_id)) = oplist_query.get(oplist_ent) else {
            error!(target: "terrgen_systems", "Oplist entity {:?} not found in terrgen_process_pending_ops", oplist_ent);
            continue;
        };

        let mut child_sizes: EntityHashMap<OplistSize> = EntityHashMap::with_capacity(oplist.bifurcations.len());
        for bifurcation in &oplist.bifurcations {
            let Some(child_oplist) = bifurcation.oplist else { continue; };
            let Ok((_, &child_size, _, _)) = oplist_query.get(child_oplist) else {
                error!(target: "terrgen_systems", "OplistSize not found for child oplist {:?}", child_oplist);
                continue;
            };
            child_sizes.insert(child_oplist, child_size);
            to_visit.push(child_oplist);
        }

        noise_entities.extend(collect_noise_entities(oplist));
        child_oplist_sizes.insert(oplist_ent, child_sizes);
        oplists.insert(oplist_ent, oplist.clone());
        oplist_ids.insert(oplist_ent, HashId::from(oplist_id.as_str()));
        oplist_debug_var_ids.insert(
            oplist_ent,
            oplist.hash_ids_mapped_to_strids.keys().copied().collect::<Vec<_>>(),
        );
        oplist_sizes.insert(oplist_ent, oplist_size);
        oplist_tags.insert(oplist_ent, oplist_tags_opt.cloned());
    }

    let mut noises: EntityHashMap<FnlNoiseComp> = EntityHashMap::with_capacity(noise_entities.len());
    for ent in noise_entities {
        let Ok(noise) = fnl_noises.get(ent) else {
            error!(target: "terrgen_systems", "Noise entity {} not found", ent);
            continue;
        };
        noises.insert(ent, noise.clone());
    }

    let mut filters: EntityHashMap<OpFilter> = EntityHashMap::with_capacity(pending_len);
    for ev in pending_ops {
        if ev.filtered_op_is_placeholder() {
            continue;
        }
        let filtered_op = ev.filtered_op();
        let Ok(filter) = op_filters.get(filtered_op) else {
            trace!(target: "terrgen_process", "Failed to get OpFilter of entity {:?}", filtered_op);
            continue;
        };
        filters.insert(filtered_op, filter.clone());
    }

    let mut dimension_hashes: EntityHashMap<HashId> = EntityHashMap::with_capacity(pending_len / 2 + 1);
    for ev in pending_ops {
        let dimension_ref = ev.dimension_ref();
        if dimension_hashes.contains_key(&dimension_ref.0) {
            continue;
        }
        let hash = dim_hash_query
            .get(dimension_ref.0)
            .cloned()
            .unwrap_or_default();
        dimension_hashes.insert(dimension_ref.0, hash);
    }

    TerrGenTaskContext {
        oplists,
        oplist_ids,
        oplist_debug_var_ids,
        oplist_sizes,
        oplist_tags,
        child_oplist_sizes,
        noises,
        filters,
        dimension_hashes,
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

    while let Some(ev) = pending_queue.pop() { unsafe {
        upsert_sample_matrix_for_pending(&ev, &mut sampled_matrices_by_requester);
        if !context.oplists.contains_key(&ev.oplist.0) {
            error!(target: "terrgen_systems", "Oplist entity {:?} not found in terrgen_process_pending_ops", ev.oplist);
            continue;
        }
        let Some(&my_oplist_size) = context.oplist_sizes.get(&ev.oplist.0) else {
            error!(target: "terrgen_systems", "OplistSize not found for oplist {:?}", ev.oplist);
            continue;
        };

        let dimension_hash = context
            .dimension_hashes
            .get(&ev.dimension_ref().0)
            .cloned()
            .unwrap_or_default();
        let filtered_op = ev.filtered_op();
        let filter = if filtered_op != Entity::PLACEHOLDER {
            context.filters.get(&filtered_op)
        } else {
            None
        };
        let has_filter = filtered_op != Entity::PLACEHOLDER;

        if let Some(compiled_root) = context
            .oplists
            .get(&ev.oplist.0)
            .and_then(|o| o.compiled_branch_ast.as_ref())
            .cloned()
        {
            process_compiled_branch_node(
                &compiled_root,
                ev.gpos(),
                my_oplist_size,
                &HashIdMap::new(),
                &ev,
                &context,
                &gen_settings,
                dimension_hash,
                filter,
                has_filter,
                &mut emitted_per_probe,
                &mut last_success_idx_for_requester,
                &mut sampled_matrices_by_requester,
                &mut result,
                capture_debug,
            );
            mark_pending_op_complete(&ev, &mut result);
            continue;
        }

        let mut frame_stack = vec![EvalFrame {
            oplist: ev.oplist.0,
            gpos: ev.gpos(),
            oplist_size: my_oplist_size,
            variables: HashIdMap::new(),
        }];

        while let Some(mut frame) = frame_stack.pop() {
            let Some(oplist) = context.oplists.get(&frame.oplist) else { continue; };
            if oplist.bifurcations.is_empty() {
                continue;
            }
            let eval_context = EvalContext {
                global_pos: frame.gpos,
                dimension_hash,
                gen_settings: &gen_settings,
                oplist_size: frame.oplist_size,
                noises: &context.noises,
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

            let bifurcation = oplist.bifurcations.get(destination_i).debug_unwrap_unchecked();
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

            if let Some(child_oplist) = bifurcation.oplist
                && let Some(child_sizes) = context.child_oplist_sizes.get(&frame.oplist)
                && let Some(&child_oplist_size) = child_sizes.get(&child_oplist)
            {
                spawn_bifurcation_frames(&mut frame_stack, &frame, child_oplist, child_oplist_size);
            }
        }
        mark_pending_op_complete(&ev, &mut result);
    }}

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
            if work.blocked_gpos.contains(&gpos) {
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
    completed_root_gpos_by_chunk: &mut EntityHashMap<HashSet<GlobalTilePos>>,
    chunk_built_msgs: &mut Vec<ChunkTerrainBuilt>,
) {
    let mut chunks_built = Vec::new();
    for &(chunk_ent, gpos) in completed_chunk_gpos {
        let Some(&expected_count) = expected_root_gpos_by_chunk.get(&chunk_ent) else {
            continue;
        };
        let completed_gpos = completed_root_gpos_by_chunk.entry(chunk_ent).or_default();
        completed_gpos.insert(gpos);
        if completed_gpos.len() < expected_count {
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

fn collect_noise_entities(oplist: &OperationList) -> Vec<Entity> {
    let mut out = Vec::new();
    for assignment in &oplist.expr_tree.assignments {
        assignment.expr.collect_noise_entities(&mut out);
    }
    oplist.expr_tree.output.collect_noise_entities(&mut out);
    out
}

fn process_compiled_branch_node(
    node: &CompiledBranchNode,
    gpos: GlobalTilePos,
    oplist_size: OplistSize,
    inherited_vars: &HashIdMap<f32>,
    source_ev: &PendingOp,
    context: &TerrGenTaskContext,
    gen_settings: &GlobalGenSettings,
    dimension_hash: HashId,
    filter: Option<&OpFilter>,
    has_filter: bool,
    emitted_per_probe: &mut EntityHashMap<u32>,
    last_success_idx_for_requester: &mut EntityHashMap<usize>,
    sampled_matrices_by_requester: &mut EntityHashMap<SampledValues>,
    result: &mut TerrGenOpTaskResult,
    capture_debug: bool,
) {
    use crate::terrain::terrgen_expression::EvalContext;

    if node.branches.is_empty() {
        return;
    }

    let eval_context = EvalContext {
        global_pos: gpos,
        dimension_hash,
        gen_settings,
        oplist_size,
        noises: &context.noises,
        variables: inherited_vars,
    };
    let (output_value, computed_vars) = node.expr_tree.eval(inherited_vars, &eval_context);
    if capture_debug {
        push_debug_sample(
            result,
            context,
            source_ev.dimension_ref(),
            gpos,
            node.source_oplist,
            output_value,
            &computed_vars,
        );
    }

    let destination_i = (output_value as usize).min(node.branches.len() - 1);
    try_emit_filter_match(
        source_ev,
        context,
        node.source_oplist,
        gpos,
        output_value,
        &computed_vars,
        filter,
        has_filter,
        emitted_per_probe,
        last_success_idx_for_requester,
        sampled_matrices_by_requester,
        result,
    );

    let branch = &node.branches[destination_i];
    collect_branch_outputs(
        result,
        source_ev,
        node.source_oplist,
        gpos,
        oplist_size,
        dimension_hash,
        &branch.biome_tags,
        &branch.tiles,
    );

    if let Some(child) = branch.child.as_ref()
        && let Some(child_oplist_size) = branch.child_size
    {
        for child_gpos in child_positions(gpos, oplist_size, child_oplist_size) {
            process_compiled_branch_node(
                child,
                child_gpos,
                child_oplist_size,
                &computed_vars,
                source_ev,
                context,
                gen_settings,
                dimension_hash,
                filter,
                has_filter,
                emitted_per_probe,
                last_success_idx_for_requester,
                sampled_matrices_by_requester,
                result,
                capture_debug,
            );
        }
    }
}

fn push_debug_sample(
    result: &mut TerrGenOpTaskResult,
    context: &TerrGenTaskContext,
    dimension_ref: DimensionRef,
    gpos: GlobalTilePos,
    oplist: Entity,
    output: f32,
    variables: &HashIdMap<f32>,
) {
    let mut debug_values = HashIdMap::new();
    if let Some(var_ids) = context.oplist_debug_var_ids.get(&oplist) {
        for var_id in var_ids {
            let Ok(value) = variables.get(*var_id) else { continue; };
            let _ = debug_values.overwrite(*var_id, *value);
        }
    }
    result.debug_samples.push(TerrGenDebugSample {
        dimension_ref,
        gpos,
        oplist,
        oplist_id: context.oplist_ids.get(&oplist).cloned().unwrap_or_default(),
        output,
        variables: debug_values,
    });
}

fn try_emit_filter_match(
    source_ev: &PendingOp,
    context: &TerrGenTaskContext,
    source_oplist: Entity,
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
    let Some(Some(oplist_tags)) = context.oplist_tags.get(&source_oplist) else {
        return;
    };
    if !oplist_tags.intersects(&filter.tags) || !passes_filter_value(Some(filter), computed_vars, output_value) {
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
    source_oplist: Entity,
    gpos: GlobalTilePos,
    oplist_size: OplistSize,
    dimension_hash: HashId,
    biome_tags: &[BiomeTagWeightAtMacroChunk],
    tiles: &[Entity],
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
    child_oplist: Entity,
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