use bevy::{ecs::entity::EntityHashMap, prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use camera::camera_components::CameraTarget;
use common::{common_components::{HashId, StrId}, common_tag_components::HashedTagsVec};
use common::log_targets::TERRGEN_SYSTEM;
use std::mem::take;
use tilemap_shared::NewMacrochunkLoaded;

use crate::{
    chunking::{macro_chunk_components::{BiomeDistribution, MacrochunkPendingBiomeSamples}},
    terrain::{
        terrprobe::opfilter::opfilter_components::OpFilter,
        operation_list::operation_list_components::*,
        terrprobe::terrprobe_messages::*,
        terrgen_async_resources::*,
        terrgen_components::*,
        terrgen_helpers::{
            build_terrgen_task_context,
            pending_root_gpos_count_for_chunk,
            process_pending_ops_batch,
            register_completed_chunk_gpos,
        },
        terrgen_messages::{ChunkTerrainBuilt, PendingOp, PendingOpInput, PendingOpPurpose},
        terrgen_resources::*,
    },
    tilemap_resources::{CloneSpawnParamSet, MassCollectedTiles},
};
use ::tilemap_shared::*;

pub use crate::terrain::terrprobe::terrprobe_systems::search_suitable_positions;

#[derive(bevy::ecs::system::SystemParam)]
pub struct TerrgenQueries<'w, 's> {
	pub oplist_query: Query<'w, 's, (&'static OperationList, &'static OplistSize, Option<&'static HashedTagsVec>, &'static StrId), ()>,
	pub fnl_noises: Query<'w, 's, &'static FnlNoiseComp>,
	pub op_filters: Query<'w, 's, &'static OpFilter>,
	pub dim_hash_query: Query<'w, 's, &'static HashId, common::AnyDisabling>,
	pub camera_query: Query<'w, 's, (&'static DimensionRef, &'static GlobalTransform), With<CameraTarget>>,
	pub macro_chunk_biome_distributions: Query<'w, 's, &'static mut BiomeDistribution>,
	pub macro_chunk_biome_sampling_states: Query<'w, 's, &'static mut MacrochunkPendingBiomeSamples>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct TerrgenResources<'w> {
    pub collected: ResMut<'w, MassCollectedTiles>,
    pub terrgen_tasks: ResMut<'w, TerrGenAsyncTasks>,
    pub debug_grid: ResMut<'w, TerrGenDebugGrid>,
    pub launch_queue: ResMut<'w, TerrGenLaunchQueue>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct TerrgenMessageWriters<'w> {
    pub chunk_built_writer: MessageWriter<'w, ChunkTerrainBuilt>,
    pub sampled_value_writer: MessageWriter<'w, SuitablePosFound>,
    pub sampled_value_matrix_writer: MessageWriter<'w, SampledValuesCollected>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct TerrgenLocalBuffers<'s> {
    pub pending_ops_batch: Local<'s, Vec<PendingOp>>,
    pub tile_requests: Local<'s, Vec<TerrGenTileRequest>>,
    pub expected_root_gpos_by_chunk: Local<'s, EntityHashMap<usize>>,
    pub completed_root_gpos_by_chunk: Local<'s, EntityHashMap<ChunkGposMask>>,
    pub chunk_built_msgs: Local<'s, Vec<ChunkTerrainBuilt>>,
    pub sampled_value_events: Local<'s, Vec<SuitablePosFound>>,
    pub sampled_value_matrix_events: Local<'s, Vec<SampledValuesCollected>>,
    pub finished_macro_chunk_biome_samples: Local<'s, Vec<(Vec<TerrGenBiomeTagSample>, Vec<Entity>)>>,
    pub pending_ops: Local<'s, Vec<PendingOp>>,
}

#[allow(unused_parens, )]
pub fn request_macrochunk_biome_sampling(
    mut cmd: Commands,
    mut loaded_macrochunks: MessageReader<NewMacrochunkLoaded>,
    mut macro_chunk_query: Query<(&DimensionRef, &MacrochunkPos, &mut MacrochunkPendingBiomeSamples, ), (With<MacroChunk>, )>,
    dimension_query: Query<&DimensionRootOplist>,
    oplists: Query<&OplistSize, With<OperationList>>,
    mut pending_ops_writer: MessageWriter<PendingOp>,
    mut local_buffers: TerrgenLocalBuffers,
    mut sample_positions: Local<Vec<GlobalTilePos>>,
) {
    local_buffers.pending_ops.clear();
    for msg in loaded_macrochunks.read() {
        let macro_chunk_ent = msg.macro_chunk_ent;
        let Ok((&dim_ref, &macro_chunk_pos, mut biome_state)) = macro_chunk_query.get_mut(macro_chunk_ent) else {
            continue;
        };
        if biome_state.0 != 0 {
            continue;
        }
        let Ok(&root_oplist) = dimension_query.get(dim_ref.0) else {
            error!(target: TERRGEN_SYSTEM, "No root operation list for macrochunk {} in {:?}", macro_chunk_pos, dim_ref);
            continue;
        };
        let Ok(_) = oplists.get(root_oplist.0) else {
            error!(target: TERRGEN_SYSTEM, "No oplist size for root operation list {:?}", root_oplist);
            continue;
        };
        sample_positions.clear();
        let sample_positions = macro_chunk_pos.sample_macro_chunk_positions(3, &mut sample_positions);
        let expected_samples = sample_positions.len() as u32;
        if expected_samples == 0 {
            cmd.entity(macro_chunk_ent).try_remove::<MacrochunkPendingBiomeSamples>();
            debug!(target: TERRGEN_SYSTEM, "Completed biome sampling for macrochunk {} in {:?} without pending samples", macro_chunk_pos, dim_ref);
            continue;
        }
        biome_state.0 = expected_samples;
        for &gpos in sample_positions {
            local_buffers.pending_ops.push(PendingOp {
                oplist: root_oplist,
                input: PendingOpInput {
                    dimension_ref: dim_ref,
                    gpos,
                },
                purpose: PendingOpPurpose::MacroChunkBiomeSampling {
                    macro_chunk_ent,
                },
            });
        }
        trace!(target: TERRGEN_SYSTEM, "Queued {} biome samples for macrochunk {} in {:?}", expected_samples, macro_chunk_pos, dim_ref);
    }
    pending_ops_writer.write_batch(local_buffers.pending_ops.drain(..));
}

#[allow(unused_parens, )]
pub fn launch_terrain_operations(
    mut commands: Commands,
    chunks_query: Query<(Entity, &ChunkPos, &DimensionRef, &TerrGenState), (With<Chunk>, Changed<TerrGenState>)>,
    dimension_query: Query<(&DimensionRootOplist), ()>,
    oplists: Query<(&OplistSize), (With<OperationList>,)>,
    mut launch_queue: ResMut<TerrGenLaunchQueue>,
    blocked_terrgen_gpos: Res<TerrGenDisabledGposByChunk>,
) {
    if chunks_query.is_empty() { return; }

    let chunk_count = chunks_query.iter().size_hint().0;
    let mut terr_gen_ops = Vec::with_capacity(chunk_count);
    for (chunk_ent, &chunk_pos, &dim_ref, terrgen_state) in chunks_query.iter() {
        if *terrgen_state != TerrGenState::Ready {
            continue;
        }
        let Ok(&dim_root_op_list) = dimension_query.get(dim_ref.0) else {
            error_once!(target: TERRGEN_SYSTEM, "No root operation list for chunk {:?} in dimension {:?}", chunk_pos, dim_ref);
            continue;
        };
        let Ok(&oplist_size) = oplists.get(dim_root_op_list.0) else {
            error_once!(target: TERRGEN_SYSTEM, "Dimension references non-existent root operation list {:?}", dim_root_op_list);
            continue;
        };
        let blocked_gpos = blocked_terrgen_gpos.get_for_chunk(dim_ref, chunk_pos);
        if !blocked_gpos.is_empty() {
            debug!(target: TERRGEN_SYSTEM, "launch_terrain_operations chunk {:?} dim {:?} blocked_gpos={}", chunk_pos, dim_ref, blocked_gpos.count_set());
        }

        launch_queue.0.push(TerrGenLaunchWork {
            chunk_ent,
            chunk_pos: chunk_pos,
            dim_ref,
            root_oplist: dim_root_op_list,
            oplist_size: oplist_size,
            blocked_gpos,
        });
        terr_gen_ops.push((chunk_ent, TerrGenState::OpsLaunched));
    }
    commands.try_insert_batch(terr_gen_ops);
}

#[allow(unused_parens, )]
pub fn process_pending_ops_and_collect_tiles(
    mut cmd: Commands,
    mut queries: TerrgenQueries,
    mut resources: TerrgenResources,
    param_set: CloneSpawnParamSet,
    mut msg_buffers: TerrgenMessageWriters,
    mut local_buffers: TerrgenLocalBuffers,
    mut pending_ops_reader: MessageReader<PendingOp>,
) {
    let oplist_query = &queries.oplist_query;
    let fnl_noises = &queries.fnl_noises;
    let op_filters = &queries.op_filters;
    let dim_hash_query = &queries.dim_hash_query;
    let camera_query = &queries.camera_query;
    let collected = &mut resources.collected;
    let terrgen_tasks = &mut resources.terrgen_tasks;
    let debug_grid = &mut resources.debug_grid;
    let launch_queue = &mut resources.launch_queue;

    let Ok(gen_settings) = param_set.gen_settings.single() else {
        error!("Failed to get gen settings");
        return;
    };
    local_buffers.pending_ops_batch.clear();
    local_buffers.tile_requests.clear();
    local_buffers.sampled_value_events.clear();
    local_buffers.sampled_value_matrix_events.clear();
    local_buffers.finished_macro_chunk_biome_samples.clear();

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
            local_buffers.pending_ops_batch.extend(batch);
            false
        } else {
            true
        }
    });

    terrgen_tasks.op_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            register_completed_chunk_gpos(
                &result.completed_chunk_gpos,
                &mut local_buffers.expected_root_gpos_by_chunk,
                &mut local_buffers.completed_root_gpos_by_chunk,
                &mut local_buffers.chunk_built_msgs,
            );
            local_buffers.sampled_value_events.extend(result.sampled_value_events);
            local_buffers.sampled_value_matrix_events.extend(result.sampled_value_matrix_events);
            local_buffers.tile_requests.extend(result.tile_requests);
            local_buffers.finished_macro_chunk_biome_samples.push((
                result.biome_tag_samples,
                result.completed_macro_chunk_biome_samples,
            ));
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

    for request in local_buffers.tile_requests.drain(..) {
        let dim_ref = request.pending.dimension_ref();
        let base_gpos = request.pending.gpos();
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

    let mut finished_macro_chunk_ents = Vec::new();
    for (samples, completed) in local_buffers.finished_macro_chunk_biome_samples.drain(..) {
        for sample in samples {
            let Ok(mut distribution) = queries.macro_chunk_biome_distributions.get_mut(sample.macro_chunk_ent) else {
                continue;
            };
            let Ok(biome_sampling_state) = queries.macro_chunk_biome_sampling_states.get_mut(sample.macro_chunk_ent) else {
                continue;
            };
            if biome_sampling_state.0 == 0 {
                continue;
            }
            distribution.add_tag_weights_in_chunk(sample.sample_chunk_pos, sample.biome_tags);
        }
        for macro_chunk_ent in completed {
            let Ok(mut biome_sampling_state) = queries.macro_chunk_biome_sampling_states.get_mut(macro_chunk_ent) else {
                continue;
            };
            if biome_sampling_state.0 == 0 {
                continue;
            }
            biome_sampling_state.0 = biome_sampling_state.0.saturating_sub(1);
            if biome_sampling_state.0 > 0 {
                continue;
            }
            finished_macro_chunk_ents.push(macro_chunk_ent);
        }
    }
    for macro_chunk_ent in finished_macro_chunk_ents {
        cmd.entity(macro_chunk_ent).try_remove::<MacrochunkPendingBiomeSamples>();
        debug!(target: TERRGEN_SYSTEM, "Completed biome sampling for macrochunk entity {:?}", macro_chunk_ent);
    }

    msg_buffers.sampled_value_writer.write_batch(local_buffers.sampled_value_events.drain(..));
    msg_buffers
        .sampled_value_matrix_writer
        .write_batch(local_buffers.sampled_value_matrix_events.drain(..));
    for chunk_built in local_buffers.chunk_built_msgs.iter() {
        cmd.entity(chunk_built.chunk_ent).try_insert(TerrGenState::Finished);
    }
    msg_buffers
        .chunk_built_writer
        .write_batch(local_buffers.chunk_built_msgs.drain(..));
    if !launch_queue.0.is_empty() {
        let work_items = take(&mut launch_queue.0);
        for work in work_items.iter() {
            let expected_count = pending_root_gpos_count_for_chunk(work);
            if expected_count == 0 {
                local_buffers.chunk_built_msgs.push(ChunkTerrainBuilt {
                    chunk_ent: work.chunk_ent,
                });
                continue;
            }
            local_buffers.expected_root_gpos_by_chunk.insert(work.chunk_ent, expected_count);
            *local_buffers.completed_root_gpos_by_chunk.entry(work.chunk_ent).or_default() = ChunkGposMask::default();
        }
        local_buffers
            .chunk_built_msgs
            .iter()
            .for_each(|chunk_built| {
                cmd.entity(chunk_built.chunk_ent).insert(TerrGenState::Finished);
            });
        msg_buffers
            .chunk_built_writer
            .write_batch(local_buffers.chunk_built_msgs.drain(..));
        let task_pool = AsyncComputeTaskPool::get();
        terrgen_tasks.launch_tasks.push(task_pool.spawn(async move {
            build_pending_ops_for_launch(work_items)
        }));
    }

    let gen_settings = gen_settings.clone();
    local_buffers.pending_ops_batch.extend(pending_ops_reader.read().cloned());
    if local_buffers.pending_ops_batch.is_empty() { return; }

    let task_context = build_terrgen_task_context(
        &local_buffers.pending_ops_batch,
        &oplist_query,
        &fnl_noises,
        &op_filters,
        &dim_hash_query,
    );

    let pending_ops_batch = take(&mut *local_buffers.pending_ops_batch);
    let capture_debug = debug_grid.enabled;
    let task_pool = AsyncComputeTaskPool::get();
    terrgen_tasks.op_tasks.push(task_pool.spawn(async move {
        process_pending_ops_batch(pending_ops_batch, task_context, gen_settings, capture_debug)
    }));
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
                let Some(bit_idx) = work.chunk_pos.bit_index_in_chunk(gpos) else {
                    continue;
                };
                if work.blocked_gpos.is_set(bit_idx) {
                    continue;
                }
                trace!(
                    target: TERRGEN_SYSTEM,
                    "Spawning terr operation {:?} at {:?} in chunk {:?}, pos_within_chunk: {:?}, oplist_size: {:?}",
                    work.root_oplist,
                    gpos,
                    work.chunk_ent,
                    pos_within_chunk,
                    work.oplist_size
                );
                batch.push(PendingOp {
                    oplist: work.root_oplist,
                    input: PendingOpInput {
                        dimension_ref: work.dim_ref,
                        gpos,
                    },
                    purpose: PendingOpPurpose::ChunkTerrainGen {
                        chunk_ent: work.chunk_ent,
                    },
                });
            }
        }
    }
    batch
}
