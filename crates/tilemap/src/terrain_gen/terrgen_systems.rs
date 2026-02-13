


use bevy::{prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use common::{common_components::{HashId}, common_tag_components::HashedTagsVec, };
use debug_unwraps::DebugUnwrapExt;
use crate::{chunking::chunking_components::*, terrain_gen::{terrgen_components::*, terrgen_messages::*, terrgen_operaton_list_components::*, terrgen_resources::*}, tilemap_resources::{CloneSpawnParamSet, MassCollectedTiles} };
use std::{collections::{HashMap, HashSet}, f32::consts::PI, mem::take};
use ::tilemap_shared::*;

#[allow(unused_parens)]
pub fn launch_terrain_gen_operations (
    mut commands: Commands,
    chunks_query: Query<(Entity, &ChunkPos, &DimensionRef), (Without<TerrGenOpsLaunched>, With<Chunk>, With<ReadyForTerrgen>)>,
    dimension_query: Query<(&DimensionRootOplist, ), ()>,
    oplists: Query<(Entity, &OplistSize), (With<OperationList>, )>,
    mut launch_queue: ResMut<TerrGenLaunchQueue>,
) {
    if chunks_query.is_empty() { return; }

    let chunk_count = chunks_query.iter().size_hint().0;
    let mut terr_gen_ops = Vec::with_capacity(chunk_count);
    for (chunk_ent, chunk_pos, &dim_ref) in chunks_query.iter() {
        let Ok((dim_root_op_list, )) = dimension_query.get(dim_ref.0) else {
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
pub fn process_pending_ops_and_collect_tiles(mut cmd: Commands,
    mut pending_ops_events: ResMut<Messages<PendingOp>>,
    oplist_query: Query<(&OperationList, &OplistSize, Option<&HashedTagsVec>), ( )>,
    fnl_noises: Query<&FnlNoiseComp,>,
    op_filters: Query<&OpFilter,>,
    param_set: CloneSpawnParamSet,
    dim_hash_query: Query<&HashId, common::AnyDisabling>,
    mut collected: ResMut<MassCollectedTiles>,
    mut terrgen_tasks: ResMut<TerrGenAsyncTasks>,
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
    let task_pool = AsyncComputeTaskPool::get();
    terrgen_tasks.op_tasks.push(task_pool.spawn(async move {
        process_pending_ops_batch(pending_ops_batch, task_context, gen_settings)
    }));
}

#[derive(Clone)]
struct TerrGenTaskContext {
    oplists: HashMap<Entity, OperationList>,
    oplist_sizes: HashMap<Entity, OplistSize>,
    oplist_tags: HashMap<Entity, Option<HashedTagsVec>>,
    child_oplist_sizes: HashMap<Entity, HashMap<Entity, OplistSize>>,
    noises: HashMap<Entity, FnlNoiseComp>,
    filters: HashMap<Entity, OpFilter>,
    dimension_hashes: HashMap<Entity, HashId>,
}

#[derive(Clone)]
struct EvalFrame {
    oplist: Entity,
    gpos: GlobalTilePos,
    oplist_size: OplistSize,
    variables: HashMap<String, f32>,
}

fn build_terrgen_task_context(
    pending_ops: &[PendingOp],
    oplist_query: &Query<(&OperationList, &OplistSize, Option<&HashedTagsVec>), ()>,
    fnl_noises: &Query<&FnlNoiseComp>,
    op_filters: &Query<&OpFilter>,
    dim_hash_query: &Query<&HashId, common::AnyDisabling>,
) -> TerrGenTaskContext {
    let pending_len = pending_ops.len();
    let mut oplists: HashMap<Entity, OperationList> = HashMap::with_capacity(pending_len);
    let mut oplist_sizes: HashMap<Entity, OplistSize> = HashMap::with_capacity(pending_len);
    let mut oplist_tags: HashMap<Entity, Option<HashedTagsVec>> = HashMap::with_capacity(pending_len);
    let mut child_oplist_sizes: HashMap<Entity, HashMap<Entity, OplistSize>> = HashMap::with_capacity(pending_len);
    let mut noise_entities: HashSet<Entity> = HashSet::with_capacity(pending_len);

    let mut to_visit: Vec<Entity> = pending_ops.iter().map(|ev| ev.oplist).collect();
    let mut visited: HashSet<Entity> = HashSet::with_capacity(pending_len);

    while let Some(oplist_ent) = to_visit.pop() {
        if !visited.insert(oplist_ent) {
            continue;
        }

        let Ok((oplist, &oplist_size, oplist_tags_opt)) = oplist_query.get(oplist_ent) else {
            error!(target: "terrgen_systems", "Oplist entity {:?} not found in terrgen_process_pending_ops", oplist_ent);
            continue;
        };

        let mut child_sizes: HashMap<Entity, OplistSize> = HashMap::with_capacity(oplist.bifurcations.len());
        for bifurcation in oplist.bifurcations.iter() {
            if let Some(child_oplist) = bifurcation.oplist {
                let Ok((_, &child_size, _)) = oplist_query.get(child_oplist) else {
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
        oplist_sizes.insert(oplist_ent, oplist_size);
        oplist_tags.insert(oplist_ent, oplist_tags_opt.cloned());
    }

    let mut noises: HashMap<Entity, FnlNoiseComp> = HashMap::with_capacity(noise_entities.len());
    for ent in noise_entities {
        let Ok(noise) = fnl_noises.get(ent) else {
            error!(target: "terrgen_systems", "Noise entity {} not found", ent);
            continue;
        };
        noises.insert(ent, noise.clone());
    }

    let mut filters: HashMap<Entity, OpFilter> = HashMap::with_capacity(pending_len);
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

    let mut dimension_hashes: HashMap<Entity, HashId> = HashMap::with_capacity(pending_len / 2 + 1);
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
        .map(|work| (ChunkPos::CHUNK_SIZE.x / work.oplist_size.x()) as usize
            * (ChunkPos::CHUNK_SIZE.y / work.oplist_size.y()) as usize)
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
) -> TerrGenOpTaskResult {
    use crate::terrain_gen::terrgen_expression::EvalContext;
    let mut result = TerrGenOpTaskResult::default();
    let mut pending_queue = pending_ops;

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
        let mut frame_stack = vec![EvalFrame {
            oplist: ev.oplist,
            gpos: ev.gpos,
            oplist_size: my_oplist_size,
            variables: HashMap::new(),
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

            if has_filter
                && let Some(filter) = filter
                && let Some(Some(oplist_tags)) = context.oplist_tags.get(&frame.oplist)
                && oplist_tags.intersects(&filter.tags)
                && filter.op_i <= -1
                && (filter.min_val..=filter.max_val).contains(&output_value)
            {
                result.sampled_value_events.push(SuitablePosFound {
                    op_filter_ent: ev.filtered_op,
                    val: output_value,
                    found_pos: frame.gpos,
                });
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


fn spawn_bifurcation_frames(
    frames: &mut Vec<EvalFrame>,
    frame: &EvalFrame,
    child_oplist: Entity,
    child_oplist_size: OplistSize,
) {
    if frame.oplist_size <= child_oplist_size
    {
        if frame.gpos.0.abs().as_uvec2() % child_oplist_size.inner() == UVec2::ZERO
        {
            frames.push(EvalFrame {
                oplist: child_oplist,
                gpos: frame.gpos,
                oplist_size: child_oplist_size,
                variables: frame.variables.clone(),
            });
        }
    }
    else{
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


#[derive(Clone)]
struct TerrGenSearchTaskInput {
    probe: TerrainProbe,
    opfilter: Option<OpFilter>,
}

fn process_search_batch(inputs: Vec<TerrGenSearchTaskInput>, found_suitable_positions: HashSet<Entity>) -> TerrGenSearchTaskResult {
    let pending_count = inputs.len();
    let mut new_pending_ops = Vec::with_capacity(pending_count);
    let mut new_pos_searches = Vec::with_capacity(pending_count);
    let mut search_failed = Vec::with_capacity(pending_count);

    for input in inputs {
        let pos_search = input.probe;

        if found_suitable_positions.contains(&pos_search.operation_filter) {
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

#[allow(unused_parens)]
//input: PosSearch messages. output: SearchFailed or SuitablePosFound(emitted in produce_tiles)
pub fn search_suitable_positions(
    mut cmd: Commands,
    mut terrain_probe: ResMut<Messages<TerrainProbe>>, mut mwriter_search_failed: MessageWriter<SearchFailed>,
    mut mwriter_pending_ops: MessageWriter<PendingOp>, mut mreader_suitable_pos_found: MessageReader<SuitablePosFound>,
    studied_ops: Query<&OpFilter, ( )>,
    failed_search_oplist_filter_holder: Query<Entity, (With<FailedSearchOplistFilterHolder>)>,
    mut terrgen_tasks: ResMut<TerrGenAsyncTasks>,
    mut found_suitable_positions: Local<HashSet<Entity>>,
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

    let found = found_suitable_positions.drain().collect::<HashSet<_>>();
    let task_pool = AsyncComputeTaskPool::get();
    terrgen_tasks.search_tasks.push(task_pool.spawn(async move {
        process_search_batch(inputs, found)
    }));
}
