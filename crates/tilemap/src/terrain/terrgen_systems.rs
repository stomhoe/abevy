use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use bevy_replicon::prelude::ClientState;
use camera::camera_components::CameraTarget;
use common::{common_components::{HashId, HashIdMap, StrId}, common_tag_components::HashedTagsVec};
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_samplers::{BiomeTagWeightAtMacroChunk, MacroChunkBiomeTagDistributionMap};
use std::mem::take;

use crate::{
    chunking::chunking_components::*,
    terrain::{
        terrprobe::opfilter::opfilter_components::OpFilter,
        operation_list::operation_list_components::*,
        terrprobe::terrprobe_messages::*,
        terrgen_async_resources::*,
        terrgen_components::*,
        terrgen_messages::PendingOp,
        terrgen_resources::*,
    },
    tilemap_resources::{CloneSpawnParamSet, MassCollectedTiles},
};
use ::tilemap_shared::*;

pub use crate::terrain::terrprobe::terrprobe_systems::search_suitable_positions;

#[derive(bevy::ecs::system::SystemParam)]
pub struct TerrgenCollectBuffers<'w, 's> {
    pub chunk_biome_tag_dist: ResMut<'w, MacroChunkBiomeTagDistributionMap>,
    pub pending_ops_batch: Local<'s, Vec<PendingOp>>,
    pub tile_requests: Local<'s, Vec<TerrGenTileRequest>>,
    pub biome_tag_samples: Local<'s, Vec<TerrGenBiomeTagSample>>,
}

#[allow(unused_parens)]
pub fn launch_terrain_operations(
    mut commands: Commands,
    chunks_query: Query<(Entity, &ChunkPos, &DimensionRef, &TerrGenState), With<Chunk>>,
    dimension_query: Query<(&DimensionRootOplist), ()>,
    oplists: Query<(&OplistSize), (With<OperationList>,)>,
    mut launch_queue: ResMut<TerrGenLaunchQueue>,
    mut blocked_terrgen_gpos: ResMut<TerrGenDisabledGposByChunk>,
) {
    if chunks_query.is_empty() { return; }

    let chunk_count = chunks_query.iter().size_hint().0;
    let mut terr_gen_ops = Vec::with_capacity(chunk_count);
    for (chunk_ent, &chunk_pos, &dim_ref, terrgen_state) in chunks_query.iter() {
        if *terrgen_state != TerrGenState::Ready {
            continue;
        }
        let Ok(&dim_root_op_list) = dimension_query.get(dim_ref.0) else {
            error!(target: "terrgen_systems", "No root operation list for chunk {:?} in dimension {:?}", chunk_pos, dim_ref);
            continue;
        };
        let Ok(&oplist_size) = oplists.get(dim_root_op_list.0) else {
            error!(target: "terrgen_systems", "Dimension references non-existent root operation list {:?}", dim_root_op_list);
            continue;
        };
        launch_queue.0.push(TerrGenLaunchWork {
            chunk_ent,
            chunk_pos: chunk_pos,
            dim_ref,
            root_oplist: dim_root_op_list,
            oplist_size: oplist_size,
            blocked_gpos: blocked_terrgen_gpos.take_for_chunk(dim_ref, chunk_pos),
        });
        terr_gen_ops.push((chunk_ent, TerrGenState::OpsLaunched));
    }
    commands.try_insert_batch(terr_gen_ops);
}

#[allow(unused_parens)]
pub fn process_pending_ops_and_collect_tiles(
    mut cmd: Commands,
    mut pending_ops_events: ResMut<Messages<PendingOp>>,
    oplist_query: Query<(&OperationList, &OplistSize, Option<&HashedTagsVec>, &StrId), ()>,
    fnl_noises: Query<&FnlNoiseComp>,
    op_filters: Query<&OpFilter>,
    param_set: CloneSpawnParamSet,
    dim_hash_query: Query<&HashId, common::AnyDisabling>,
    mut collected: ResMut<MassCollectedTiles>,
    camera_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    mut terrgen_tasks: ResMut<TerrGenAsyncTasks>,
    mut debug_grid: ResMut<TerrGenDebugGrid>,
    mut launch_queue: ResMut<TerrGenLaunchQueue>,
    mut buffers: TerrgenCollectBuffers,
    mut ewriter_sampled_value: MessageWriter<SuitablePosFound>,
    mut ewriter_sampled_value_matrix: MessageWriter<SampledValuesCollected>,
    client_state: Res<State<ClientState>>,
) {
    let Ok(gen_settings) = param_set.gen_settings.single() else {
        error!("Failed to get gen settings");
        return;
    };
    buffers.pending_ops_batch.clear();
    buffers.tile_requests.clear();
    buffers.biome_tag_samples.clear();
    let mut sampled_value_events: Vec<SuitablePosFound> = Vec::new();
    let mut sampled_value_matrix_events: Vec<SampledValuesCollected> = Vec::new();

    let bucket_size = debug_grid.bucket_size_tiles.max(1);
    let capture_margin = (debug_grid.bucket_radius + debug_grid.capture_margin_buckets).max(4);
    let camera_info = camera_query.iter().next().map(|(dim_ref, transform)| {
        let gpos = GlobalTilePos::from(transform.translation().xy()).0;
        let bucket = IVec2::new(gpos.x.div_euclid(bucket_size), gpos.y.div_euclid(bucket_size));
        (dim_ref.0, bucket)
    });

    if debug_grid.enabled {
        if let Some((cam_dim, cam_bucket)) = camera_info {
            debug_grid.tiles.retain(|key, _| {
                if key.dimension != cam_dim {
                    return false;
                }
                let bucket = key.gpos;
                (bucket.x - cam_bucket.x).abs() <= capture_margin
                    && (bucket.y - cam_bucket.y).abs() <= capture_margin
            });
        } else {
            debug_grid.tiles.clear();
        }
    }

    terrgen_tasks.launch_tasks.retain_mut(|task| {
        if let Some(batch) = future::block_on(future::poll_once(task)) {
            buffers.pending_ops_batch.extend(batch);
            false
        } else {
            true
        }
    });

    terrgen_tasks.op_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            sampled_value_events.extend(result.sampled_value_events);
            sampled_value_matrix_events.extend(result.sampled_value_matrix_events);
            buffers.tile_requests.extend(result.tile_requests);
            buffers.biome_tag_samples.extend(result.biome_tag_samples);
            if debug_grid.enabled {
                let Some((cam_dim, cam_bucket)) = camera_info else { return false; };
                let mut samples = result.debug_samples;
                samples.sort_by_key(|sample| {
                    let sb = IVec2::new(
                        sample.gpos.0.x.div_euclid(bucket_size),
                        sample.gpos.0.y.div_euclid(bucket_size),
                    );
                    (sb.x - cam_bucket.x).abs() + (sb.y - cam_bucket.y).abs()
                });

                for sample in samples {
                    if sample.dimension_ref.0 != cam_dim {
                        continue;
                    }
                    let sb = IVec2::new(
                        sample.gpos.0.x.div_euclid(bucket_size),
                        sample.gpos.0.y.div_euclid(bucket_size),
                    );
                    if (sb.x - cam_bucket.x).abs() > capture_margin
                        || (sb.y - cam_bucket.y).abs() > capture_margin
                    {
                        continue;
                    }
                    let key = TerrGenDebugTileKey {
                        dimension: sample.dimension_ref.0,
                        gpos: sb,
                        oplist: sample.oplist,
                    };
                    if let Some(existing) = debug_grid.tiles.get_mut(&key) {
                        existing.oplist = sample.oplist;
                        existing.oplist_id = sample.oplist_id;
                        existing.output = sample.output;
                        existing.variables = sample.variables;
                    } else {
                        if debug_grid.tiles.len() >= debug_grid.max_entries {
                            continue;
                        }
                        debug_grid.tiles.insert(key, TerrGenTileDebugInfo {
                            oplist: sample.oplist,
                            oplist_id: sample.oplist_id,
                            output: sample.output,
                            variables: sample.variables,
                        });
                    }
                }
            }
            false
        } else {
            true
        }
    });

    for request in buffers.tile_requests.drain(..) {
        let dim_ref = request.pending.dimension_ref;
        let base_gpos = request.pending.gpos;
        collected.collect_tiles_at_positions(
            &mut cmd,
            request.bif_tiles.into_iter().map(|tile_ent| {
                let offset = param_set
                    .terrgen_offsets
                    .get(tile_ent)
                    .copied()
                    .unwrap_or_default()
                    .0;
                (tile_ent, base_gpos + offset)
            }),
            dim_ref,
            &param_set,
            request.dimension_hash,
        );
    }

    if *client_state.get() == ClientState::Disconnected {
        for sample in buffers.biome_tag_samples.drain(..) {
            buffers.chunk_biome_tag_dist.add_tag_weights(
                sample.dimension_ref,
                sample.macro_chunk_pos,
                sample.biome_tags.into_iter(),
            );
        }
    } else {
        buffers.biome_tag_samples.clear();
    }

    if !sampled_value_events.is_empty() {
        ewriter_sampled_value.write_batch(sampled_value_events);
    }
    if !sampled_value_matrix_events.is_empty() {
        ewriter_sampled_value_matrix.write_batch(sampled_value_matrix_events);
    }

    if !launch_queue.0.is_empty() {
        let work_items = take(&mut launch_queue.0);
        let task_pool = AsyncComputeTaskPool::get();
        terrgen_tasks.launch_tasks.push(task_pool.spawn(async move {
            build_pending_ops_for_launch(work_items)
        }));
    }

    let gen_settings = gen_settings.clone();
    buffers.pending_ops_batch.extend(pending_ops_events.drain());
    if buffers.pending_ops_batch.is_empty() { return; }

    let task_context = build_terrgen_task_context(
        &buffers.pending_ops_batch,
        &oplist_query,
        &fnl_noises,
        &op_filters,
        &dim_hash_query,
    );

    let pending_ops_batch = take(&mut *buffers.pending_ops_batch);
    let capture_debug = debug_grid.enabled;
    let task_pool = AsyncComputeTaskPool::get();
    terrgen_tasks.op_tasks.push(task_pool.spawn(async move {
        process_pending_ops_batch(pending_ops_batch, task_context, gen_settings, capture_debug)
    }));
}

#[derive(Clone)]
struct TerrGenTaskContext {
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

fn build_terrgen_task_context(
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
        for bifurcation in oplist.bifurcations.iter() {
            if let Some(child_oplist) = bifurcation.oplist {
                let Ok((_, &child_size, _, _)) = oplist_query.get(child_oplist) else {
                    error!(target: "terrgen_systems", "OplistSize not found for child oplist {:?}", child_oplist);
                    continue;
                };
                child_sizes.insert(child_oplist, child_size);
                to_visit.push(child_oplist);
            }
        }

        noise_entities.extend(collect_noise_entities(oplist));
        child_oplist_sizes.insert(oplist_ent, child_sizes);
        oplists.insert(oplist_ent, oplist.clone());
        oplist_ids.insert(oplist_ent, HashId::from(oplist_id.as_str()));
        oplist_debug_var_ids.insert(
            oplist_ent,
            oplist
                .hash_ids_mapped_to_strids
                .keys()
                .copied()
                .collect::<Vec<_>>(),
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
    for ev in pending_ops.iter() {
        if ev.filtered_op_is_placeholder() {
            continue;
        }
        if let Ok(filter) = op_filters.get(ev.filtered_op) {
            filters.insert(ev.filtered_op, filter.clone());
        } else {
            trace!(target: "terrgen_process", "Failed to get OpFilter of entity {:?}", ev.filtered_op);
        }
    }

    let mut dimension_hashes: EntityHashMap<HashId> = EntityHashMap::with_capacity(pending_len / 2 + 1);
    for ev in pending_ops.iter() {
        if !dimension_hashes.contains_key(&ev.dimension_ref.0) {
            let hash = dim_hash_query
                .get(ev.dimension_ref.0)
                .cloned()
                .unwrap_or_default();
            dimension_hashes.insert(ev.dimension_ref.0, hash);
        }
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

fn build_pending_ops_for_launch(work_items: Vec<TerrGenLaunchWork>) -> Vec<PendingOp> {
    let total_area: usize = work_items
        .iter()
        .map(|work| {
            (ChunkPos::CHUNK_SIZE.x / work.oplist_size.x()) as usize
                * (ChunkPos::CHUNK_SIZE.y / work.oplist_size.y()) as usize
        })
        .sum();
    let mut batch = Vec::with_capacity(total_area);
    for work in work_items {
        for x in 0..ChunkPos::CHUNK_SIZE.x / work.oplist_size.x() {
            for y in 0..ChunkPos::CHUNK_SIZE.y / work.oplist_size.y() {
                let pos_within_chunk = IVec2::new(x as i32, y as i32);
                let gpos = work.chunk_pos.to_tilepos() + GlobalTilePos(pos_within_chunk * work.oplist_size.inner().as_ivec2());
                if work.blocked_gpos.contains(&gpos) {
                    continue;
                }
                trace!(
                    target: "terrgen_systems",
                    "Spawning terr operation {:?} at {:?} in chunk {:?}, pos_within_chunk: {:?}, oplist_size: {:?}",
                    work.root_oplist,
                    gpos,
                    work.chunk_ent,
                    pos_within_chunk,
                    work.oplist_size
                );
                batch.push(PendingOp {
                    oplist: work.root_oplist,
                    dimension_ref: work.dim_ref,
                    gpos,
                    filtered_op: Entity::PLACEHOLDER,
                    requester: Entity::PLACEHOLDER,
                    max_emitted_results: 0,
                    mark_last_success_in_batch: false,
                    matrix_spec: None,
                });
            }
        }
    }
    batch
}



fn collect_noise_entities(oplist: &OperationList) -> Vec<Entity> {
    let mut out = Vec::new();
    for assignment in oplist.expr_tree.assignments.iter() {
        assignment.expr.collect_noise_entities(&mut out);
    }
    oplist.expr_tree.output.collect_noise_entities(&mut out);
    out
}

fn process_pending_ops_batch(
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
            .get(&ev.dimension_ref.0)
            .cloned()
            .unwrap_or_default();
        let filter = if ev.filtered_op != Entity::PLACEHOLDER {
            context.filters.get(&ev.filtered_op)
        } else {
            None
        };

        let has_filter = ev.filtered_op != Entity::PLACEHOLDER;
        if let Some(compiled_root) = context
            .oplists
            .get(&ev.oplist.0)
            .and_then(|o| o.compiled_branch_ast.as_ref())
            .cloned()
        {
            process_compiled_branch_node(
                &compiled_root,
                ev.gpos,
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
            continue;
        }

        let mut frame_stack = vec![EvalFrame {
            oplist: ev.oplist.0,
            gpos: ev.gpos,
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
                    ev.dimension_ref,
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
    }}
    for (_, sample_idx) in last_success_idx_for_requester.drain() {
        if let Some(sample) = result.sampled_value_events.get_mut(sample_idx) {
            sample.is_last = true;
        }
    }
    for (requester, matrix) in sampled_matrices_by_requester.drain() {
        result.sampled_value_matrix_events.push(SampledValuesCollected {
            requester,
            matrix,
        });
    }
    result
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
            source_ev.dimension_ref,
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
    if source_ev.matrix_spec.is_some() {
        let sampled_value = sampled_value_from_filter(Some(filter), computed_vars, output_value);
        set_sample_matrix_value_for_pending(source_ev, Some(sampled_value), sampled_matrices_by_requester);
        return;
    }
    let emitted = emitted_per_probe.entry(source_ev.requester).or_insert(0);
    if *emitted >= source_ev.max_emitted_results {
        return;
    }
    result.sampled_value_events.push(SuitablePosFound {
        requester: source_ev.requester,
        val: output_value,
        found_pos: gpos,
        is_last: false,
    });
    if source_ev.mark_last_success_in_batch {
        last_success_idx_for_requester.insert(source_ev.requester, result.sampled_value_events.len() - 1);
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
    if source_ev.filtered_op != Entity::PLACEHOLDER {
        return;
    }
    if !biome_tags.is_empty() {
        result.biome_tag_samples.push(TerrGenBiomeTagSample {
            dimension_ref: source_ev.dimension_ref,
            macro_chunk_pos: gpos.to_chunkpos().to_macrochunk_pos(),
            biome_tags: biome_tags.to_vec(),
        });
    }
    if !tiles.is_empty() {
        result.tile_requests.push(TerrGenTileRequest {
            bif_tiles: tiles.to_vec(),
            pending: PendingOp {
                oplist: DimensionRootOplist(source_oplist),
                dimension_ref: source_ev.dimension_ref,
                gpos,
                filtered_op: source_ev.filtered_op,
                requester: source_ev.requester,
                max_emitted_results: source_ev.max_emitted_results,
                mark_last_success_in_batch: source_ev.mark_last_success_in_batch,
                matrix_spec: source_ev.matrix_spec,
            },
            oplist_size,
            dimension_hash,
        });
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
        if let Ok(val) = computed_vars.get(var_name_hash) {
            return (filter.min_val..=filter.max_val).contains(val);
        }
        return false;
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
    let Some(spec) = source_ev.matrix_spec else {
        return;
    };
    sampled_matrices_by_requester
        .entry(source_ev.requester)
        .or_insert_with(|| SampledValues::new(spec.min, spec.matrix_size, spec.spacing));
}

fn set_sample_matrix_value_for_pending(
    source_ev: &PendingOp,
    value: Option<f32>,
    sampled_matrices_by_requester: &mut EntityHashMap<SampledValues>,
) {
    let Some(_) = source_ev.matrix_spec else {
        return;
    };
    let Some(matrix) = sampled_matrices_by_requester.get_mut(&source_ev.requester) else {
        return;
    };
    if value.is_none() {
        let _ = matrix.set(source_ev.gpos, None);
        return;
    }
    let Some(current) = matrix.get(source_ev.gpos) else {
        return;
    };
    if current.is_none() {
        let _ = matrix.set(source_ev.gpos, value);
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
