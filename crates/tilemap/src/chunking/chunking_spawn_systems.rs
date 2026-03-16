
use bevy::prelude::*;
use common::common_components::{StrId20B};
use common::log_targets::CHUNK_ACTIVATION;
use tilemap_shared::{DimensionRef, LoadedChunks, LoadedMacroChunks};
use tilemap_shared::{ChunkPos, HashablePosVec, MACRO_CHUNK_SIZE_IN_CHUNKS};

use super::chunking_components::*;
use super::macro_chunk_components::{BiomeDistribution, MacroChunkBiomeSamplingState};
use super::chunking_resources::*;
use crate::regioning::{regioning_components::Region, regioning_resources::LoadedRegions};



#[allow(unused_parens, )]
pub fn spawn_chunks_around_activators(
    mut reader: MessageReader<ReactivateChunksFor>,
    mut cmd: Commands,
    mut query: Query<(&GlobalTransform, &mut ActivatingChunks, &DimensionRef), ()>,
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

    for msg in reader.read() {
        let Ok((transform, mut activates_chunks, &dimension_ref)) = query.get_mut(msg.0) else {
            debug!(target: CHUNK_ACTIVATION, "Skipping stale chunk reactivation request for missing activator {:?}", msg.0);
            continue;
        };
        comps_for_macrochunk_ents.reserve(approx_macro_chunks);
        comps_for_chunk_ents.reserve(approx_chunks);
        comps_for_region_ents.reserve(approx_chunks / 8);
        let center_chunk_pos = ChunkPos::from(transform.translation().xy());

        activates_chunks.reactivation_timer.reset();

        for y in (center_chunk_pos.y() - cnt + 1)..(center_chunk_pos.y() + cnt) {
            for x in (center_chunk_pos.x() - cnt + 1)..(center_chunk_pos.x() + cnt) {

                let chunk_pos = ChunkPos::new(x, y);
                let key = (dimension_ref, chunk_pos);
                let chunk_ent = loaded_chunks.0.get(&key)
                .copied()
                .or_else(|| loaded_chunks.0.get(&key).copied())
                .unwrap_or_else(|| {
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
                                StrId20B::trunc(format!("MacroChunk({}, {})", macro_chunk_pos.0.x, macro_chunk_pos.0.y)),
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
                        .or_else(|| loaded_regions.0.get(&region_key).copied())
                        .unwrap_or_else(|| {
                            let region_ent = cmd.spawn_empty().id();
                            comps_for_region_ents.push((region_ent, (
                                region_pos,
                                Region,
                                StrId20B::trunc(format!("Region({}, {})", region_pos.0.x, region_pos.0.y)),
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
                        StrId20B::trunc(format!("Chunk({}, {})", chunk_pos.0.x, chunk_pos.0.y)),
                        Transform::default(),
                        chunk_pos,
                        ChildOf(region_ent),
                        dimension_ref,
                    )));
                    chunk_ent
                });
                if !activates_chunks.entities.contains(&chunk_ent) {
                    activates_chunks.entities.push(chunk_ent);
                }
            }
        }
    }
    cmd.try_insert_batch(comps_for_macrochunk_ents);
    cmd.try_insert_batch(comps_for_region_ents);
    cmd.try_insert_batch(comps_for_chunk_ents);
    macro_chunk_loaded_writer.write_batch(macro_chunk_loaded_msgs.drain(..));

}

pub fn track_discovered_areas(
    mut cmd: Commands,
    chunk_query: Query<(&MacroChunkRef, &TerrGenState), (With<Chunk>, Changed<TerrGenState>)>,
) {
    for (&macro_chunk_ref, terrgen_state) in chunk_query.iter() {
        if *terrgen_state != TerrGenState::OpsLaunched {
            continue;
        }
        cmd.entity(macro_chunk_ref.0).try_insert(TerrgenDiscoveredMacroChunk);
    }
}



#[derive(Message, Debug, Clone, )]
pub struct ReactivateChunksFor(pub Entity);

#[derive(Message, Debug, Clone, Copy)]
pub struct MacroChunkLoaded {
    pub macro_chunk_ent: Entity,
}


#[allow(unused_parens)]
pub fn activate_chunks_every_second( //TODO borrar esto y hacer que se haga 1 segundo despues del ultimo movimiento
    mut query: Query<(Entity, &mut ActivatingChunks),(With<DimensionRef>, With<GlobalTransform>)>,
    time: Res<Time>,
    mut writer: MessageWriter<ReactivateChunksFor>,
    mut to_reactivate: Local<Vec<ReactivateChunksFor>>,
) {
    for (being_entity, mut activates_chunks) in query.iter_mut() {
        activates_chunks.reactivation_timer.tick(time.delta());
        if activates_chunks.reactivation_timer.is_finished() {
            to_reactivate.push(ReactivateChunksFor(being_entity));
        }
    }
    writer.write_batch(to_reactivate.drain(..));
}
#[allow(unused_parens, )]
pub fn detect_activators_with_pos_changes(
    query: Query<(Entity),
    (Or<(Changed<GlobalTransform>, Changed<DimensionRef>, Added<ActivatingChunks>,)>, With<ActivatingChunks>, With<DimensionRef>, With<GlobalTransform>)>,
    mut writer: MessageWriter<ReactivateChunksFor>,
    mut msgs: Local<Vec<ReactivateChunksFor>>,
) {
    msgs.extend(query.iter().map(ReactivateChunksFor));
    writer.write_batch(msgs.drain(..));
}
