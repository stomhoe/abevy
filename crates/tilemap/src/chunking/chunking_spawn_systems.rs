
use std::mem::take;

use bevy::prelude::*;
use common::common_components::{StrId20B};
use common::log_targets::CHUNK_ACTIVATION;
use tilemap_shared::{DimensionRef, LoadedChunks, LoadedMacroChunks, GlobalTilePos};
use tilemap_shared::{ChunkPos, MACRO_CHUNK_SIZE_IN_CHUNKS};
use being_shared::Being;

use super::chunking_components::*;
use super::macro_chunk_components::{BiomeDistribution, MacroChunkBiomeSamplingState};
use super::chunking_resources::*;
use crate::regioning::{regioning_components::Region, regioning_resources::LoadedRegions};



#[allow(unused_parens, )]
pub fn update_activating_chunk_positions(
    mut reader: MessageReader<ReactivateChunksFor>,
    mut query: Query<(&GlobalTransform, &mut ActivateChunksAround, &mut ActivatingChunks, )>,
    tilemap_settings: Res<AaChunkRangeSettings>,
) {
    let cnt = tilemap_settings.discovery_range as i32;

    for msg in reader.read() {
        let Ok((transform, mut activate_chunks_around, mut activates_chunks, )) = query.get_mut(msg.0) else {
            debug!(target: CHUNK_ACTIVATION, "Skipping stale chunk reactivation request for missing activator {:?}", msg.0);
            continue;
        };
        let center_chunk_pos = ChunkPos::from(transform.translation().xy());
        activate_chunks_around.reactivation_timer.reset();
        activates_chunks.insert_positions_around(center_chunk_pos, cnt);
    }
}

#[allow(unused_parens, )]
pub fn spawn_activated_chunks(
    mut cmd: Commands,
    query: Query<(&ActivatingChunks, &DimensionRef), Changed<ActivatingChunks>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut loaded_macro_chunks: ResMut<LoadedMacroChunks>,
    tilemap_settings: Res<AaChunkRangeSettings>,
    mut loaded_regions: ResMut<LoadedRegions>,
    mut macro_chunk_loaded_writer: MessageWriter<MacroChunkLoaded>,
    mut macro_chunk_loaded_msgs: Local<Vec<MacroChunkLoaded>>,
) {
    let cnt = tilemap_settings.discovery_range as i32;
    let range_len = (2 * cnt - 1).max(0) as usize;
    let approx_chunks = range_len.saturating_mul(range_len);
    let approx_macro_chunks = (range_len / MACRO_CHUNK_SIZE_IN_CHUNKS.0.x as usize + 1)
        .saturating_mul(range_len / MACRO_CHUNK_SIZE_IN_CHUNKS.0.y as usize + 1);
    let mut comps_for_macrochunk_ents = Vec::new();
    let mut comps_for_region_ents = Vec::new();
    let mut comps_for_chunk_ents = Vec::new();

    for (activates_chunks, &dimension_ref) in query.iter() {
        comps_for_macrochunk_ents.reserve(approx_macro_chunks);
        comps_for_chunk_ents.reserve(approx_chunks);
        comps_for_region_ents.reserve(approx_chunks / 8);
        for &chunk_pos in activates_chunks.chunk_positions.iter() {
                let key = (dimension_ref, chunk_pos);
                if loaded_chunks.0.contains_key(&key) {
                    continue;
                }

                let macro_chunk_pos = chunk_pos.to_macrochunk_pos();
                let macro_chunk_key = (dimension_ref, macro_chunk_pos);
                let macro_chunk_ent = loaded_macro_chunks.0.get(&macro_chunk_key)
                    .copied()
                    .unwrap_or_else(|| {
                        let macro_chunk_ent = cmd.spawn_empty().id();
                        comps_for_macrochunk_ents.push((macro_chunk_ent, (
                            MacroChunk,
                            BiomeDistribution::default(),
                            MacroChunkBiomeSamplingState::default(),
                            macro_chunk_pos,
                            StrId20B::trunc(format!("{:?}", macro_chunk_pos)),
                            Transform::default(),
                            ChildOf(dimension_ref.0),
                            dimension_ref,
                        )));
                        loaded_macro_chunks.0.insert(macro_chunk_key, macro_chunk_ent);
                        macro_chunk_loaded_msgs.push(MacroChunkLoaded {
                            macro_chunk_ent,
                        });
                        macro_chunk_ent
                    });
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
                            StrId20B::trunc(format!("{:?}", region_pos)),
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
                comps_for_chunk_ents.push((chunk_ent, (
                    Chunk { region_ent, },
                    MacroChunkRef(macro_chunk_ent),
                    TerrGenState::Pending,
                    Visibility::Hidden,
                    TilesToSave::default(),
                    StrId20B::trunc(format!("{:?}", chunk_pos)),
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

}





#[derive(Message, Debug, Clone, )]
pub struct ReactivateChunksFor(pub Entity);

#[derive(Message, Debug, Clone, Copy)]
pub struct MacroChunkLoaded {
    pub macro_chunk_ent: Entity,
}

#[derive(Message, Debug, Clone, )]
pub struct BeingChunkDespawned{
    pub chunk_ent: Entity,
}

#[allow(unused_parens)]
pub fn activate_chunks_every_second( //TODO borrar esto y hacer que se haga 1 segundo despues del ultimo movimiento
    mut query: Query<(Entity, &mut ActivateChunksAround), (With<ActivatingChunks>, With<DimensionRef>, With<GlobalTransform>)>,
    time: Res<Time>,
    mut writer: MessageWriter<ReactivateChunksFor>,
    mut to_reactivate: Local<Vec<ReactivateChunksFor>>,
) {
    for (being_entity, mut activate_chunks_around) in query.iter_mut() {
        activate_chunks_around.reactivation_timer.tick(time.delta());
        if activate_chunks_around.reactivation_timer.is_finished() {
            to_reactivate.push(ReactivateChunksFor(being_entity));
        }
    }
    writer.write_batch(to_reactivate.drain(..));
}
#[allow(unused_parens, )]
pub fn detect_activators_with_pos_changes(
    query: Query<(Entity),
    (Or<(Changed<GlobalTransform>, Changed<DimensionRef>, Added<ActivatingChunks>, Added<ActivateChunksAround>)>, With<ActivatingChunks>, With<ActivateChunksAround>, With<DimensionRef>, With<GlobalTransform>)>,
    mut writer: MessageWriter<ReactivateChunksFor>,
    mut msgs: Local<Vec<ReactivateChunksFor>>,
) {
    msgs.extend(query.iter().map(ReactivateChunksFor));
    writer.write_batch(msgs.drain(..));
}



#[allow(unused_parens)]
pub fn update_within_chunk(
    mut cmd: Commands,
    mut query: Query<
        (Entity, &GlobalTilePos, &DimensionRef, Option<&WithinChunk>, Option<&mut ChunkPos>),
        (With<Being>, Changed<GlobalTilePos>),
    >,
    loaded_chunks: Res<LoadedChunks>,
) {
    let mut within_chunks: Vec<(Entity, WithinChunk)> = Vec::new();
    for (being_ent, gpos, &dim_ref, within_chunk, being_chunk_pos) in query.iter_mut() {
        let new_chunk_pos = ChunkPos::from(*gpos);
        let new_chunk_key = (dim_ref, new_chunk_pos);
        
        let Some(&new_chunk_ent) = loaded_chunks.0.get(&new_chunk_key) else {
            debug!(target: common::log_targets::CHUNK_ACTIVATION, "Being {:?} at {:?} moved to unloaded chunk", being_ent, gpos);
            continue;
        };
        
        if within_chunk.map(|c| c.0) != Some(new_chunk_ent) {

            within_chunks.push((being_ent, WithinChunk(new_chunk_ent)));
            if let Some(mut being_chunk_pos) = being_chunk_pos {
                *being_chunk_pos = new_chunk_pos;
            } else{
                cmd.entity(being_ent).try_insert(new_chunk_pos);
            }
        }
    }
    cmd.try_insert_batch(take(&mut within_chunks));
}
