
use bevy::prelude::*;
use common::log_targets::CHUNK_ACTIVATION;
use tilemap_shared::*;
use being_shared::{Being, Unloaded};

use super::macro_chunk_components::{BiomeDistribution, MacrochunkPendingBiomeSamples};
use crate::{chunking::MacroChunkTileIndices, regioning::{regioning_components::Region, regioning_resources::LoadedRegions}};



#[allow(unused_parens, )]
pub fn add_activating_chunks_to_activate_chunks_around(
    mut cmd: Commands,
    load_chunks_around_query: Query<(Entity, &LoadChunksAround), (Changed<LoadChunksAround>, )>,
    mut removed_load_chunks_around: RemovedComponents<LoadChunksAround>,
) {
    for (ent, chunk_range) in load_chunks_around_query.iter() {
        cmd.entity(ent).insert(ActivatingChunks::with_capacity(chunk_range));
    }
    for ent in removed_load_chunks_around.read() {
        cmd.entity(ent).try_remove::<ActivatingChunks>();
    }
}

#[allow(unused_parens, )]
pub fn update_activating_chunk_positions(
    mut reader: MessageReader<UpdateActivatedChunkPos>,
    mut query: Query<(&GlobalTilePos, &LoadChunksAround, &mut ActivatingChunks, )>,
) {
    for msg in reader.read() {
        let Ok((gpos, chunk_range_settings, mut activates_chunks, )) = query.get_mut(msg.being_ent) else {
            debug!(target: CHUNK_ACTIVATION, "Skipping stale chunk reactivation request for missing activator {:?}", msg.being_ent);
            continue;
        };
        let center_chunk_pos = ChunkPos::from(*gpos);
        let cnt = chunk_range_settings.discovery_range as i32;
        let is_one_chunk = chunk_range_settings.is_one_chunk();
        activates_chunks.0.clear();
        activates_chunks.insert_positions_around(center_chunk_pos, cnt);
        if is_one_chunk {
            insert_border_chunk_positions(&mut activates_chunks, center_chunk_pos, *gpos);
        }
    }
}

fn insert_border_chunk_positions(
    activates_chunks: &mut ActivatingChunks,
    center_chunk_pos: ChunkPos,
    center_gpos: GlobalTilePos,
) {
    let chunk_size = ChunkPos::CHUNK_SIZE.as_ivec2();
    let local_tile_pos = center_gpos.0 - center_chunk_pos.to_tilepos().0;

    let mut push_if_missing = |chunk_pos: ChunkPos| {
        if activates_chunks.0.contains(&chunk_pos) {
            return;
        }
        activates_chunks.0.push(chunk_pos);
    };

    if local_tile_pos.x == 0 {
        push_if_missing(center_chunk_pos + IVec2::new(-1, 0));
    } else if local_tile_pos.x == chunk_size.x - 1 {
        push_if_missing(center_chunk_pos + IVec2::new(1, 0));
    }

    if local_tile_pos.y == 0 {
        push_if_missing(center_chunk_pos + IVec2::new(0, -1));
    } else if local_tile_pos.y == chunk_size.y - 1 {
        push_if_missing(center_chunk_pos + IVec2::new(0, 1));
    }
}

#[allow(unused_parens, )]
pub fn spawn_activated_chunks(
    mut cmd: Commands,
    query: Query<(&ActivatingChunks, &DimensionRef, ), Changed<ActivatingChunks>>,
    macro_chunk_holder_query: Query<&MacroChunkHolderRef>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut loaded_macro_chunks: ResMut<LoadedMacroChunks>,
    mut loaded_regions: ResMut<LoadedRegions>,
    mut macro_chunk_loaded_writer: MessageWriter<NewMacrochunkLoaded>,
    mut macro_chunk_loaded_msgs: Local<Vec<NewMacrochunkLoaded>>,
    mut chunk_loaded_writer: MessageWriter<ChunkLoaded>,
    mut chunk_loaded_msgs: Local<Vec<ChunkLoaded>>,
) {
    let mut comps_for_macrochunk_ents = Vec::new();
    let mut comps_for_region_ents = Vec::new();
    let mut comps_for_chunk_ents = Vec::new();

    for (activates_chunks, &dimension_ref, ) in query.iter() {
        for &chunk_pos in activates_chunks.0.iter() {
                let key = (dimension_ref, chunk_pos);
                if loaded_chunks.0.contains_key(&key) {
                    continue;
                }
                let macro_chunk_pos = chunk_pos.to_macrochunk_pos();
                let macro_chunk_key = (dimension_ref, macro_chunk_pos);
                let Ok(macro_chunk_holder_ref) = macro_chunk_holder_query.get(dimension_ref.0) else {
                    error!(target: CHUNK_ACTIVATION, "Dimension {:?} has no MacroChunkHolderRef for macrochunk {}", dimension_ref, macro_chunk_pos);
                    continue;
                };
                let macro_chunk_ent = loaded_macro_chunks.0.get(&macro_chunk_key)
                    .copied()
                    .unwrap_or_else(|| {
                        let macro_chunk_ent = cmd.spawn_empty().id();
                        comps_for_macrochunk_ents.push((macro_chunk_ent, (
                            MacroChunk,
                            MacroChunkTileIndices::default(),
                            BiomeDistribution::default(),
                            MacrochunkPendingBiomeSamples::default(),
                            macro_chunk_pos,
                            Name::new(format!("{:?}", macro_chunk_pos)),
                            ChildOf(macro_chunk_holder_ref.0),
                            dimension_ref,
                        )));
                        loaded_macro_chunks.0.insert(macro_chunk_key, macro_chunk_ent);
                        macro_chunk_loaded_msgs.push(NewMacrochunkLoaded {
                            macro_chunk_ent,
                        });
                        macro_chunk_ent
                    });
                cmd.entity(macro_chunk_ent).try_insert(ChildOf(macro_chunk_holder_ref.0));
                let region_ent = {
                    let region_pos = chunk_pos.to_region_pos();
                    let region_key = (dimension_ref, region_pos);
                    loaded_regions.0.get(&region_key)
                    .copied()
                    .unwrap_or_else(|| {
                        let region_ent = cmd.spawn_empty().id();
                        comps_for_region_ents.push((region_ent, (
                            region_pos,
                            Region,
                            Name::new(format!("{:?}", region_pos)),
                            Transform::default(),
                            ChildOf(dimension_ref.0),
                            dimension_ref,
                        )));
                        loaded_regions.0.insert(region_key, region_ent);
                        region_ent
                    })
                };

                let chunk_ent = cmd.spawn_empty().id();
                loaded_chunks.0.insert(key, chunk_ent);
                chunk_loaded_msgs.push(ChunkLoaded {
                    dimension: dimension_ref,
                    chunk_pos,
                });
                comps_for_chunk_ents.push((chunk_ent, (
                    Chunk { region_ent, },
                    MacroChunkRef(macro_chunk_ent),
                    TerrGenState::Pending,
                    Visibility::Hidden,
                    TilesToSave::default(),
                    Name::new(format!("{:?}", chunk_pos)),
                    Transform::default(),
                    chunk_pos,
                    ChildOf(region_ent),
                    dimension_ref,
                )));
        }
    }
    cmd.try_insert_batch(comps_for_macrochunk_ents);
    cmd.try_insert_batch(comps_for_region_ents);
    cmd.try_insert_batch(comps_for_chunk_ents);
    macro_chunk_loaded_writer.write_batch(macro_chunk_loaded_msgs.drain(..));
    chunk_loaded_writer.write_batch(chunk_loaded_msgs.drain(..));

}
#[allow(unused_parens, )]
pub fn detect_activators_with_pos_changes(
    query: Query<(Entity),
    (Or<(Changed<GlobalTilePos>, Changed<DimensionRef>, Changed<LoadChunksAround>, Added<ActivatingChunks>)>, With<ActivatingChunks>, With<DimensionRef>, With<GlobalTilePos>, With<LoadChunksAround>)>,
    mut writer: MessageWriter<UpdateActivatedChunkPos>,
    mut msgs: Local<Vec<UpdateActivatedChunkPos>>,
) {
    msgs.extend(query.iter().map(|ent| UpdateActivatedChunkPos { being_ent: ent }));
    writer.write_batch(msgs.drain(..));
}



#[allow(unused_parens)]
pub fn update_beings_within_chunk_res(
    mut cmd: Commands,
    mut query: Query<
        (Entity, &GlobalTilePos, &DimensionRef, Option<&mut ChunkPos>),
        (
            With<Being>,
            Without<Unloaded>,
            Or<(Changed<GlobalTilePos>, Changed<DimensionRef>, Added<Being>)>,
        ),
    >,
    loaded_chunks: Res<LoadedChunks>,
    mut beings_within_chunk: ResMut<BeingsInCpos>,
    mut chunk_beings_changed_writer: MessageWriter<ChunkBeingsChanged>,
    mut chunk_beings_changed_msgs: Local<Vec<ChunkBeingsChanged>>,
) {
    for (being_ent, gpos, &dim_ref, being_chunk_pos) in query.iter_mut() {
        let &gpos = gpos;
        let new_chunk_pos = ChunkPos::from(gpos);
        let old_chunk_pos = beings_within_chunk.by_being.get(&being_ent).copied();
        if old_chunk_pos == Some((dim_ref, new_chunk_pos)) {
            continue;
        }

        if let Some(mut being_chunk_pos) = being_chunk_pos {
            if *being_chunk_pos != new_chunk_pos {
                *being_chunk_pos = new_chunk_pos;
            }
        } else{
            cmd.entity(being_ent).try_insert(new_chunk_pos);
        }

        beings_within_chunk.set_being_chunk(being_ent, dim_ref, new_chunk_pos);

        if let Some(old_chunk_pos) = old_chunk_pos {
            if old_chunk_pos != (dim_ref, new_chunk_pos) {
                if loaded_chunks.0.contains_key(&old_chunk_pos) {
                    chunk_beings_changed_msgs.push(ChunkBeingsChanged {
                        dim: old_chunk_pos.0,
                        cpos: old_chunk_pos.1,
                    });
                }
            }
        }

        if loaded_chunks.0.contains_key(&(dim_ref, new_chunk_pos)) {
            chunk_beings_changed_msgs.push(ChunkBeingsChanged {
                dim: dim_ref,
                cpos: new_chunk_pos,
            });
        }
    }
    chunk_beings_changed_writer.write_batch(chunk_beings_changed_msgs.drain(..));
}
