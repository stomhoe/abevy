
use bevy::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use being_shared::prelude::UnloadBeing;
use ::tilemap_shared::*;
use common::log_targets::CHUNK_ACTIVATION;

use super::chunking_components::*;
use super::chunking_resources::*;
use crate::prelude::ChunkWithBeingsWantsDespawn;
use crate::tile::tile_messages::SavedTileHadChunkDespawn;


#[allow(unused_parens, )]
pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&GlobalTilePos, &ActivateChunksAround, &mut ActivatingChunks, &DimensionRef), (Or<(Changed<GlobalTilePos>, Changed<DimensionRef>, Changed<ActivateChunksAround>)>, )>,
    loaded_chunks: Res<LoadedChunks>,
    mut ewriter: MessageWriter<CheckIfChunkShouldDespawn>,
    mut to_despawn: Local<Vec<CheckIfChunkShouldDespawn>>,
) {
    for (act_gpos, chunkrange_settings, mut activates_chunks, &act_dim) in activator_query.iter_mut() {
        let act_chunk_pos = ChunkPos::from(*act_gpos);
        let mut i = 0;
        while i < activates_chunks.chunk_positions.len() {
            let chunk_pos = activates_chunks.chunk_positions[i];
            let keep = if chunkrange_settings.is_one_chunk() && is_border_adjacent_chunk(*act_gpos, act_chunk_pos, chunk_pos) {
                true
            } else {
                !chunkrange_settings.out_of_discovery_range(act_chunk_pos, chunk_pos)
            };
            if !keep {
                if let Some(&chunk_ent) = loaded_chunks.0.get(&(act_dim, chunk_pos)) {
                    to_despawn.push(CheckIfChunkShouldDespawn(chunk_ent));
                }
                activates_chunks.chunk_positions.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }
    ewriter.write_batch(to_despawn.drain(..));
}

fn is_border_adjacent_chunk(center_gpos: GlobalTilePos, center_chunk_pos: ChunkPos, chunk_pos: ChunkPos) -> bool {
    let chunk_size = ChunkPos::CHUNK_SIZE.as_ivec2();
    let local_tile_pos = center_gpos.0 - center_chunk_pos.to_tilepos().0;
    (local_tile_pos.x == 0 && chunk_pos == center_chunk_pos + IVec2::new(-1, 0))
        || (local_tile_pos.x == chunk_size.x - 1 && chunk_pos == center_chunk_pos + IVec2::new(1, 0))
        || (local_tile_pos.y == 0 && chunk_pos == center_chunk_pos + IVec2::new(0, -1))
        || (local_tile_pos.y == chunk_size.y - 1 && chunk_pos == center_chunk_pos + IVec2::new(0, 1))
}

#[derive(Debug, Message)]
pub struct CheckIfChunkShouldDespawn(pub Entity);


#[allow(unused_parens)]
pub fn on_message_signal_despawn_all_chunks(
    mut reader: MessageReader<ForceAllChunksDespawn>,
    chunks_query: Query<(Entity),(With<Chunk>)>,
    mut writer: MessageWriter<MakeChunkDespawn>,
) {
    if reader.is_empty() {
        return;
    }
    reader.clear();
    let mut evs = Vec::with_capacity(chunks_query.iter().size_hint().0);
    for ent in chunks_query.iter() {
        evs.push(MakeChunkDespawn { chunk_ent: ent, delegate_if_beings: false });
    }

    writer.write_batch(evs);
}


#[derive(Debug, Message, Clone, Copy)]
pub struct MakeChunkDespawn { pub chunk_ent: Entity, pub delegate_if_beings: bool }
impl MakeChunkDespawn {
    pub fn default(chunk_ent: Entity) -> Self {
        Self { chunk_ent, delegate_if_beings: true }
    }
    pub fn new_no_delegate_if_beings(chunk_ent: Entity) -> Self {
        Self { chunk_ent, delegate_if_beings: false }
    }
}

pub fn periodically_check_despawn_unreferenced_chunks(
    mut ewriter: MessageWriter<CheckIfChunkShouldDespawn>,
    chunks_query: Query<Entity, With<Chunk>,>,
    mut to_check: Local<Vec<CheckIfChunkShouldDespawn>>,
) {
    for chunk_ent in chunks_query.iter() {
        to_check.push(CheckIfChunkShouldDespawn(chunk_ent));
    }
    ewriter.write_batch(to_check.drain(..));
}

#[allow(unused_parens)]
pub fn despawn_chunks(//DEJARLO DE ESTA FORMA PARA CENTRALIZAR EL SISTEMA DONDE PUEDEN DESPAWNEAR LOS CHUNKS, PARA RESPETAR EL ORDEN DE SISTEMAS
    mut cmd: Commands,
    activator_query: Query<(&DimensionRef, &ActivatingChunks, ), >,
    chunks_query: Query<(Option<&Tilemaps>, Option<&TilesToSave>, Option<&BeingsWithinChunk>), >,
    loaded_chunks: Res<LoadedChunks>,
    mut despawn_events: ResMut<Messages<CheckIfChunkShouldDespawn>>,
    mut tosave_event_writer: MessageWriter<SavedTileHadChunkDespawn>,
    mut force_despawn_reader: MessageReader<MakeChunkDespawn>,
    mut referenced_chunks: Local<EntityHashSet>,
    mut tosave_events: Local<Vec<SavedTileHadChunkDespawn>>,
    mut chunks_to_despawn: Local<Vec<MakeChunkDespawn>>,
    mut bcd_writer: MessageWriter<ChunkWithBeingsWantsDespawn>,
    mut bcd_msgs: Local<Vec<ChunkWithBeingsWantsDespawn>>,
) {
    referenced_chunks.clear();
    referenced_chunks.reserve(activator_query.iter().map(|(_, a)| a.chunk_positions.len()).sum());
    for (&dimension_ref, activates_chunks) in activator_query.iter() {
        referenced_chunks.extend(
            activates_chunks
                .chunk_positions
                .iter()
                .filter_map(|&chunk_pos| loaded_chunks.0.get(&(dimension_ref, chunk_pos)).copied())
        );
    }
    tosave_events.clear();

    for CheckIfChunkShouldDespawn(chunk_ent) in despawn_events.drain() {

        let referenced = referenced_chunks.contains(&chunk_ent);

        if !referenced {
            chunks_to_despawn.push(MakeChunkDespawn::default(chunk_ent));
        }
    }
    chunks_to_despawn.extend(force_despawn_reader.read().cloned());

    for MakeChunkDespawn { chunk_ent, delegate_if_beings } in chunks_to_despawn.drain(..) {
        let mut delegate_chunk_despawn_to_other_system = false;
        let Ok((tilemaps, _tiles_to_save, beings_within_chunk)) = chunks_query.get(chunk_ent) else {
            cmd.entity(chunk_ent).try_despawn();
            continue;
        };
        if let Some(tilemaps) = tilemaps {
            for _tmap_ent in tilemaps.iter() {
                // if tiles_to_save.entities().contains(&child) {
                //     cmd.entity(child).try_remove::<ChildOf>();//esto hace q el sistema limpiador la borre, hay q hacer algo
                //     tosave_events.push(SavedTileHadChunkDespawn(child));
                // } else{//HACE FALTA
                //     cmd.entity(child).try_despawn();
                // }
            }
        }
        let beings_within_chunk_count = beings_within_chunk.map_or(0, |beings| beings.len());
        if beings_within_chunk_count > 0 {
            delegate_chunk_despawn_to_other_system = true;
        }

        if delegate_chunk_despawn_to_other_system && delegate_if_beings {
            bcd_msgs.push(ChunkWithBeingsWantsDespawn { chunk_ent });
            debug!(target: CHUNK_ACTIVATION, "Delegated chunk {:?} despawn decision for {} beings", chunk_ent, beings_within_chunk_count);
        } else {
            cmd.entity(chunk_ent).try_despawn();
        }
    }
    tosave_event_writer.write_batch(tosave_events.drain(..));

    bcd_writer.write_batch(bcd_msgs.drain(..));
}
#[allow(unused_parens)]
pub fn on_chunk_despawn(
    trig: On<Despawn, (Chunk, )>,
    chunk_query: Query<(&DimensionRef, &ChunkPos, Option<&BeingsWithinChunk>), ()>,
    mut cmd: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut loaded_macro_chunks: ResMut<LoadedMacroChunks>,
    mut ub_writer: MessageWriter<UnloadBeing>,
    mut ub_messages: Local<Vec<UnloadBeing>>,
){
    let Ok((&chunk_dimension, &chunk_pos, beings_within_chunk)) = chunk_query.get(trig.entity) else {
        error!(target: "chunk_despawn", "Chunk entity {:?} despawned but its DimensionRef or ChunkPos component is missing", trig.entity);
        loaded_chunks.0.retain(|_, chunk_entity| chunk_entity.clone() != trig.entity);

        return;
    };
    let Some(chunk_ent) = loaded_chunks.0.get(&(chunk_dimension, chunk_pos))
    else {
        return;
    };
    if *chunk_ent == trig.entity {
        loaded_chunks.0.remove(&(chunk_dimension, chunk_pos));
    }
    if let Some(beings_within_chunk) = beings_within_chunk {
        for being_ent in beings_within_chunk.iter() {
            ub_messages.push(UnloadBeing(being_ent));
        }
        ub_writer.write_batch(ub_messages.drain(..));
    }


    let macro_chunk_pos = chunk_pos.to_macrochunk_pos();
    let has_remaining_chunks = loaded_chunks.0.keys().any(|&(dim_ref, chunk_pos)| {
        dim_ref == chunk_dimension && chunk_pos.to_macrochunk_pos() == macro_chunk_pos
    });
    if has_remaining_chunks {
        return;
    }
    let Some(macro_chunk_ent) = loaded_macro_chunks.0.remove(&(chunk_dimension, macro_chunk_pos)) else {
        return;
    };
    cmd.entity(macro_chunk_ent).try_despawn();
}
