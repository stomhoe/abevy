use bevy::{ecs::entity::{EntityHashMap, EntityHashSet}, prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use camera::camera_components::CameraTarget;
use common::{common_components::{HashId, HashIdMap, StrId}, common_tag_components::HashedTagsVec};
use debug_unwraps::DebugUnwrapExt;
use std::{collections::HashSet, mem::take};

use crate::{
    chunking::chunking_components::*,
    terrain_gen::{
        terrgen_components::*,
        terrgen_messages::*,
        terrgen_operaton_list_components::*,
        terrgen_resources::*,
    },
    tilemap_resources::{CloneSpawnParamSet, MassCollectedTiles},
};
use ::tilemap_shared::*;

pub use crate::terrain_gen::terrgen_search::search_suitable_positions;

#[allow(unused_parens)]
pub fn launch_terrain_gen_operations(
    mut commands: Commands,
    chunks_query: Query<(Entity, &ChunkPos, &DimensionRef), (Without<TerrGenOpsLaunched>, With<Chunk>, With<ReadyForTerrgen>)>,
    dimension_query: Query<(&DimensionRootOplist,), ()>,
    oplists: Query<(Entity, &OplistSize), (With<OperationList>,)>,
    mut launch_queue: ResMut<TerrGenLaunchQueue>,
) {
    if chunks_query.is_empty() { return; }

    let chunk_count = chunks_query.iter().size_hint().0;
    let mut terr_gen_ops = Vec::with_capacity(chunk_count);
    for (chunk_ent, chunk_pos, &dim_ref) in chunks_query.iter() {
        let Ok((dim_root_op_list,)) = dimension_query.get(dim_ref.0) else {
            error!(target: "terrgen_systems", "No root operation list for chunk {:?} in dimension {:?}", chunk_pos, dim_ref);
            continue;
        };
        let Ok((oplist, oplist_size)) = oplists.get(dim_root_op_list.0) else {
            error!(target: "terrgen_systems", "Dimension references non-existent root operation list {:?}", dim_root_op_list);
            continue;
        };
        launch_queue.0.push(TerrGenLaunchWork {
            chunk_ent,
            chunk_pos: *chunk_pos,
            dim_ref,
            root_oplist: oplist,
            oplist_size: *oplist_size,
        });
        terr_gen_ops.push((chunk_ent, TerrGenOpsLaunched));
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
    mut pending_ops_batch: Local<Vec<PendingOp>>,
    mut sampled_value_events: Local<Vec<SuitablePosFound>>,
    mut tile_requests: Local<Vec<TerrGenTileRequest>>,
    mut ewriter_sampled_value: MessageWriter<SuitablePosFound>,
) {
    let Ok(gen_settings) = param_set.gen_settings.single() else {
        error!("Failed to get gen settings");
        return;
    };
    pending_ops_batch.clear();
    sampled_value_events.clear();
    tile_requests.clear();

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
            pending_ops_batch.extend(batch);
            false
        } else {
            true
        }
    });

    terrgen_tasks.op_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            sampled_value_events.extend(result.sampled_value_events);
            tile_requests.extend(result.tile_requests);
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

    for request in tile_requests.drain(..) {
        collected.collect_tiles(
            &mut cmd,
            request.bif_tiles,
            &request.pending,
            &param_set,
            request.dimension_hash,
        );
    }

    if !sampled_value_events.is_empty() {
        ewriter_sampled_value.write_batch(sampled_value_events.drain(..));
    }

    if !launch_queue.0.is_empty() {
        let work_items = take(&mut launch_queue.0);
        let task_pool = AsyncComputeTaskPool::get();
        terrgen_tasks.launch_tasks.push(task_pool.spawn(async move {
            build_pending_ops_for_launch(work_items)
        }));
    }

    let gen_settings = gen_settings.clone();
    pending_ops_batch.extend(pending_ops_events.drain());
    if pending_ops_batch.is_empty() { return; }

    let task_context = build_terrgen_task_context(
        &pending_ops_batch,
        &oplist_query,
        &fnl_noises,
        &op_filters,
        &dim_hash_query,
    );

    let pending_ops_batch = take(&mut *pending_ops_batch);
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

    let mut to_visit: Vec<Entity> = pending_ops.iter().map(|ev| ev.oplist).collect();
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
        if ev.filtered_op == Entity::PLACEHOLDER {
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
                    max_emitted_results: 0,
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
    use crate::terrain_gen::terrgen_expression::EvalContext;
    let mut result = TerrGenOpTaskResult::default();
    let mut pending_queue = pending_ops;
    let mut emitted_per_filter: EntityHashMap<u16> = EntityHashMap::new();

    while let Some(ev) = pending_queue.pop() { unsafe {
        if !context.oplists.contains_key(&ev.oplist) {
            error!(target: "terrgen_systems", "Oplist entity {:?} not found in terrgen_process_pending_ops", ev.oplist);
            continue;
        }
        let Some(&my_oplist_size) = context.oplist_sizes.get(&ev.oplist) else {
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
            .get(&ev.oplist)
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
                &mut emitted_per_filter,
                &mut result,
                capture_debug,
            );
            continue;
        }

        let mut frame_stack = vec![EvalFrame {
            oplist: ev.oplist,
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
                let mut debug_values = HashIdMap::new();
                if let Some(var_ids) = context.oplist_debug_var_ids.get(&frame.oplist) {
                    for var_id in var_ids {
                        if let Ok(value) = frame.variables.get(*var_id) {
                            let _ = debug_values.overwrite(*var_id, *value);
                        }
                    }
                }
                result.debug_samples.push(TerrGenDebugSample {
                    dimension_ref: ev.dimension_ref,
                    gpos: frame.gpos,
                    oplist: frame.oplist,
                    oplist_id: context.oplist_ids.get(&frame.oplist).cloned().unwrap_or_default(),
                    output: output_value,
                    variables: debug_values,
                });
            }

            if has_filter
                && let Some(filter) = filter
                && let Some(Some(oplist_tags)) = context.oplist_tags.get(&frame.oplist)
                && oplist_tags.intersects(&filter.tags)
                && filter.op_i <= -1
                && (filter.min_val..=filter.max_val).contains(&output_value)
            {
                let emitted = emitted_per_filter.entry(ev.filtered_op).or_insert(0);
                if *emitted < ev.max_emitted_results {
                    result.sampled_value_events.push(SuitablePosFound {
                        op_filter_ent: ev.filtered_op,
                        val: output_value,
                        found_pos: frame.gpos,
                    });
                    *emitted = emitted.saturating_add(1);
                }
            }

            let destination_i = (output_value as usize).min(oplist.bifurcations.len() - 1);
            let bifurcation = oplist.bifurcations.get(destination_i).debug_unwrap_unchecked();

            if !bifurcation.tiles.is_empty() && ev.filtered_op == Entity::PLACEHOLDER {
                result.tile_requests.push(TerrGenTileRequest {
                    bif_tiles: bifurcation.tiles.clone(),
                    pending: PendingOp {
                        oplist: frame.oplist,
                        dimension_ref: ev.dimension_ref,
                        gpos: frame.gpos,
                        filtered_op: ev.filtered_op,
                        max_emitted_results: ev.max_emitted_results,
                    },
                    oplist_size: frame.oplist_size,
                    dimension_hash,
                });
            }

            if let Some(child_oplist) = bifurcation.oplist
                && let Some(child_sizes) = context.child_oplist_sizes.get(&frame.oplist)
                && let Some(&child_oplist_size) = child_sizes.get(&child_oplist)
            {
                spawn_bifurcation_frames(
                    &mut frame_stack,
                    &frame,
                    child_oplist,
                    child_oplist_size,
                );
            }
        }
    }}

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
    emitted_per_filter: &mut EntityHashMap<u16>,
    result: &mut TerrGenOpTaskResult,
    capture_debug: bool,
) {
    use crate::terrain_gen::terrgen_expression::EvalContext;

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
        let mut debug_values = HashIdMap::new();
        if let Some(var_ids) = context.oplist_debug_var_ids.get(&node.source_oplist) {
            for var_id in var_ids {
                if let Ok(value) = computed_vars.get(*var_id) {
                    let _ = debug_values.overwrite(*var_id, *value);
                }
            }
        }
        result.debug_samples.push(TerrGenDebugSample {
            dimension_ref: source_ev.dimension_ref,
            gpos,
            oplist: node.source_oplist,
            oplist_id: context.oplist_ids.get(&node.source_oplist).cloned().unwrap_or_default(),
            output: output_value,
            variables: debug_values,
        });
    }

    if has_filter
        && let Some(filter) = filter
        && let Some(Some(oplist_tags)) = context.oplist_tags.get(&node.source_oplist)
        && oplist_tags.intersects(&filter.tags)
        && filter.op_i <= -1
        && (filter.min_val..=filter.max_val).contains(&output_value)
    {
        let emitted = emitted_per_filter.entry(source_ev.filtered_op).or_insert(0);
        if *emitted < source_ev.max_emitted_results {
            result.sampled_value_events.push(SuitablePosFound {
                op_filter_ent: source_ev.filtered_op,
                val: output_value,
                found_pos: gpos,
            });
            *emitted = emitted.saturating_add(1);
        }
    }

    let destination_i = (output_value as usize).min(node.branches.len() - 1);
    let branch = &node.branches[destination_i];

    if !branch.tiles.is_empty() && source_ev.filtered_op == Entity::PLACEHOLDER {
        result.tile_requests.push(TerrGenTileRequest {
            bif_tiles: branch.tiles.clone(),
            pending: PendingOp {
                oplist: node.source_oplist,
                dimension_ref: source_ev.dimension_ref,
                gpos,
                filtered_op: source_ev.filtered_op,
                max_emitted_results: source_ev.max_emitted_results,
            },
            oplist_size,
            dimension_hash,
        });
    }

    if let Some(child) = branch.child.as_ref()
        && let Some(child_oplist_size) = branch.child_size
    {
        if oplist_size <= child_oplist_size {
            if gpos.0.abs().as_uvec2() % child_oplist_size.inner() == UVec2::ZERO {
                process_compiled_branch_node(
                    child,
                    gpos,
                    child_oplist_size,
                    &computed_vars,
                    source_ev,
                    context,
                    gen_settings,
                    dimension_hash,
                    filter,
                    has_filter,
                    emitted_per_filter,
                    result,
                    capture_debug,
                );
            }
        } else {
            let x_end = oplist_size.x() as i32 / child_oplist_size.x() as i32;
            let y_end = oplist_size.y() as i32 / child_oplist_size.y() as i32;
            for x in 0..x_end {
                for y in 0..y_end {
                    process_compiled_branch_node(
                        child,
                        gpos + GlobalTilePos::new(x, y),
                        child_oplist_size,
                        &computed_vars,
                        source_ev,
                        context,
                        gen_settings,
                        dimension_hash,
                        filter,
                        has_filter,
                        emitted_per_filter,
                        result,
                        capture_debug,
                    );
                }
            }
        }
    }
}

fn spawn_bifurcation_frames(
    frames: &mut Vec<EvalFrame>,
    frame: &EvalFrame,
    child_oplist: Entity,
    child_oplist_size: OplistSize,
) {
    if frame.oplist_size <= child_oplist_size {
        if frame.gpos.0.abs().as_uvec2() % child_oplist_size.inner() == UVec2::ZERO {
            frames.push(EvalFrame {
                oplist: child_oplist,
                gpos: frame.gpos,
                oplist_size: child_oplist_size,
                variables: frame.variables.clone(),
            });
        }
    } else {
        let x_end = frame.oplist_size.x() as i32 / child_oplist_size.x() as i32;
        let y_end = frame.oplist_size.y() as i32 / child_oplist_size.y() as i32;
        for x in 0..x_end {
            for y in 0..y_end {
                let gpos = frame.gpos + GlobalTilePos::new(x, y);
                frames.push(EvalFrame {
                    oplist: child_oplist,
                    gpos,
                    oplist_size: child_oplist_size,
                    variables: frame.variables.clone(),
                });
            }
        }
    }
}
