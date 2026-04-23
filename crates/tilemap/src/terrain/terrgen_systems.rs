use bevy::{platform::collections::HashMap, prelude::*, tasks::{AsyncComputeTaskPool, futures_lite::future}};
use camera::camera_components::CameraTarget;
use std::sync::Arc;
use common::{common_components::{HashId, HashIdMap}};
use common::log_targets::TERRGEN_SYSTEM;
use ::tilemap_shared::*;
use std::mem::take;

use crate::{
    chunking::{macro_chunk_components::{BiomeDistribution, MacrochunkPendingBiomeSamples}},
    terrain::{
        operation_list::operation_list_components::*,
        operation_list::operation_list_resources::OperationListEntityMap,
        terrprobe::terrprobe_messages::*,
        terrgen_async_resources::*,
        terrgen_helpers::*,
        terrgen_async_fns::*,
        terrgen_messages::*,
        terrgen_resources::*,
    },
    tile::{tile_components::*, tile_sampler_components::*},
    tilemap_resources::{CloneSpawnParamSet, MassCollectedTiles},
};

pub use crate::terrain::terrprobe::terrprobe_systems::search_suitable_positions;

#[derive(bevy::ecs::system::SystemParam)]
pub struct TerrgenQueries<'w, 's> {
	pub camera_query: Query<'w, 's, (&'static DimensionRef, &'static GlobalTransform), With<CameraTarget>>,
	pub macro_chunk_biome_distributions: Query<'w, 's, &'static mut BiomeDistribution>,
	pub macro_chunk_biome_sampling_states: Query<'w, 's, &'static mut MacrochunkPendingBiomeSamples>,
	pub tile_hash_query: Query<'w, 's, (Entity, &'static HashId), Or<(With<Tile>, With<TileWeightedSampler>)>>,
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct TerrgenResources<'w> {
    pub collected: ResMut<'w, MassCollectedTiles>,
    pub(crate) shared_task_data: Option<Res<'w, TerrGenSharedTaskData>>,
    pub debug_grid: ResMut<'w, TerrGenDebugGrid>,
    pub terrgen_tasks: ResMut<'w, TerrAsyncTasks>,
    pub chunk_terrgen_queue: ResMut<'w, ChunksTerrgenQueue>,
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
    pub expected_root_gpos_by_chunk: Local<'s, HashMap<(DimensionRef, ChunkPos), usize>>,
    pub completed_root_gpos_by_chunk: Local<'s, HashMap<(DimensionRef, ChunkPos), ChunkGposMask>>,
    pub chunk_built_msgs: Local<'s, Vec<ChunkTerrainBuilt>>,
    pub sampled_value_events: Local<'s, Vec<SuitablePosFound>>,
    pub sampled_value_matrix_events: Local<'s, Vec<SampledValuesCollected>>,
    pub finished_macro_chunk_biome_samples: Local<'s, Vec<(Vec<TerrGenBiomeTagSample>, Vec<Entity>)>>,
}

#[allow(unused_parens, )]
pub fn enqueue_chunk_terrgen_jobs(
    mut chunks_query: Query<(Entity, &ChunkPos, &DimensionRef, &mut TerrGenState), (With<Chunk>, Changed<TerrGenState>)>,
    dimension_query: Query<(&DimensionRootOplist), ()>,
    dimension_map: Res<DimensionEntityMap>,
    oplist_map: Res<OperationListEntityMap>,
    oplist_size_query: Query<(&OplistSize), (With<OperationList>,)>,
    mut chunk_terrgen_queue: ResMut<ChunksTerrgenQueue>,
    blocked_terrgen_gpos: Res<TerrGenDisabledGposByChunk>,
) {
    if chunks_query.is_empty() { return; }

    for (_chunk_ent, &chunk_pos, &dim_ref, mut terrgen_state) in chunks_query.iter_mut() {
        if *terrgen_state != TerrGenState::Ready {
            continue;
        }
        let Ok(dim_ent) = dimension_map.0.get_cloned(dim_ref.0) else {
            error_once!(target: TERRGEN_SYSTEM, "Missing Dimension entity for hash {:?}", dim_ref.0);
            continue;
        };
        let Ok(&dim_root_op_list) = dimension_query.get(dim_ent) else {
            error_once!(target: TERRGEN_SYSTEM, "No root operation list for dimension {:?}", dim_ref);
            continue;
        };
        let Ok(root_oplist_ent) = oplist_map.0.get_cloned(dim_root_op_list.0) else {
            error_once!(target: TERRGEN_SYSTEM, "Dimension references missing root operation list hash {:?}", dim_root_op_list.0);
            continue;
        };
        let Ok(&oplist_size) = oplist_size_query.get(root_oplist_ent) else {
            error_once!(target: TERRGEN_SYSTEM, "Dimension references non-existent root operation list {:?}", dim_root_op_list);
            continue;
        };
        let blocked_gpos = blocked_terrgen_gpos.get_for_chunk(dim_ref, chunk_pos);

        chunk_terrgen_queue.0.push(ChunkTerrGenWork {
            chunk_pos: chunk_pos,
            dim_ref,
            root_oplist: dim_root_op_list,
            oplist_size: oplist_size,
            blocked_gpos,
        });
        *terrgen_state = TerrGenState::OpsLaunched;
    }
}

#[allow(unused_parens, )]
pub fn process_pending_ops_and_collect_tiles(
    mut cmd: Commands,
    mut queries: TerrgenQueries,
    mut resources: TerrgenResources,
    loaded_chunks: Res<LoadedChunks>,
    param_set: CloneSpawnParamSet,
    mut msg_buffers: TerrgenMessageWriters,
    mut local_buffers: TerrgenLocalBuffers,
    mut pending_ops_reader: MessageReader<PendingOp>,
) {
    let camera_query = &queries.camera_query;
    let tile_hash_query = &queries.tile_hash_query;
    let collected = &mut resources.collected;
    let terrgen_tasks = &mut resources.terrgen_tasks;
    let debug_grid = &mut resources.debug_grid;
    let chunk_terrgen_queue = &mut resources.chunk_terrgen_queue;

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
        let mut tile_entities = HashIdMap::default();
        for (tile_ent, hash_id) in tile_hash_query.iter() {
            let _ = tile_entities.overwrite(*hash_id, tile_ent);
        }
        collected.collect_tiles_at_positions(
            &mut cmd,
            request.bif_tiles.into_iter().filter_map(|tile_hash| {
                let Ok(tile_ent) = tile_entities.get(tile_hash) else {
                    error!(target: TERRGEN_SYSTEM, "Tile hash {:?} not found in local tile hash map", tile_hash);
                    return None;
                };
                let offset = param_set
                    .terrgen_offsets
                    .get(*tile_ent)
                    .copied()
                    .unwrap_or_default()
                    .0;
                Some((*tile_ent, base_gpos + offset))
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
    }

    msg_buffers.sampled_value_writer.write_batch(local_buffers.sampled_value_events.drain(..));
    msg_buffers
        .sampled_value_matrix_writer
        .write_batch(local_buffers.sampled_value_matrix_events.drain(..));
    for chunk_built in local_buffers.chunk_built_msgs.iter() {
        if let Some(&chunk_ent) = loaded_chunks.0.get(&(chunk_built.dimension_ref, chunk_built.chunk_pos)) {
            cmd.entity(chunk_ent).try_insert(TerrGenState::Finished);
        }
    }
    msg_buffers
        .chunk_built_writer
        .write_batch(local_buffers.chunk_built_msgs.drain(..));
    if !chunk_terrgen_queue.0.is_empty() {
        let work_items = take(&mut chunk_terrgen_queue.0);
        for work in work_items.iter() {
            let expected_count = pending_root_gpos_count_for_chunk(work);
            if expected_count == 0 {
                local_buffers.chunk_built_msgs.push(ChunkTerrainBuilt {
                    dimension_ref: work.dim_ref,
                    chunk_pos: work.chunk_pos,
                });
                continue;
            }
            local_buffers.expected_root_gpos_by_chunk.insert((work.dim_ref, work.chunk_pos), expected_count);
            *local_buffers.completed_root_gpos_by_chunk.entry((work.dim_ref, work.chunk_pos)).or_default() = ChunkGposMask::default();
        }
        local_buffers
            .chunk_built_msgs
            .iter()
            .for_each(|chunk_built| {
                if let Some(&chunk_ent) = loaded_chunks.0.get(&(chunk_built.dimension_ref, chunk_built.chunk_pos)) {
                    cmd.entity(chunk_ent).insert(TerrGenState::Finished);
                }
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

    let Some(shared_task_data) = resources.shared_task_data.as_ref() else {
        error!(target: TERRGEN_SYSTEM, "TerrGenSharedTaskData was not initialized before processing pending ops");
        return;
    };

    let task_context = Arc::clone(&shared_task_data.0);

    let pending_ops_batch = take(&mut *local_buffers.pending_ops_batch);
    let capture_debug = debug_grid.enabled;
    let task_pool = AsyncComputeTaskPool::get();
    terrgen_tasks.op_tasks.push(task_pool.spawn(async move {
        process_pending_ops_batch(pending_ops_batch, task_context, gen_settings, capture_debug)
    }));
}
