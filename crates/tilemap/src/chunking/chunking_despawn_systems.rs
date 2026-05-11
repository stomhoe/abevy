
use bevy::prelude::*;
use being_shared::FaithfulSimBeing;
use bevy_replicon::shared::backend::ClientState;
use ::tilemap_shared::*;
use common::log_targets::{CHUNK_ACTIVATION, CHUNK_DESPAWN};

#[allow(unused_parens, )]
pub fn rem_outofrange_chunks_from_activators(
    mut activator_query: Query<(&GlobalTilePos, &LoadChunksAround, &mut ActivatingChunks, &DimensionRef), (Or<(Changed<GlobalTilePos>, Changed<DimensionRef>, Changed<LoadChunksAround>, Changed<ActivatingChunks>)>, )>,
    loaded_chunks: Res<LoadedChunks>,
    mut ewriter: MessageWriter<CheckIfChunkShouldDespawn>,
    mut to_despawn: Local<Vec<CheckIfChunkShouldDespawn>>,
) {
    for (act_gpos, chunkrange_settings, mut activates_chunks, &act_dim) in activator_query.iter_mut() {
        let act_chunk_pos = ChunkPos::from(*act_gpos);
        let mut i = 0;
        while i < activates_chunks.0.len() {
            let chunk_pos = activates_chunks.0[i];
            let keep = if chunkrange_settings.is_one_chunk() && is_border_adjacent_chunk(*act_gpos, act_chunk_pos, chunk_pos) {
                true
            } else {
                !chunkrange_settings.out_of_discovery_range(act_chunk_pos, chunk_pos)
            };
            if !keep {
                if let Some(&chunk_ent) = loaded_chunks.0.get(&(act_dim, chunk_pos)) {
                    to_despawn.push(CheckIfChunkShouldDespawn(chunk_ent));
                }
                activates_chunks.0.swap_remove(i);
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
    let iter = chunks_query.iter();
    let (lower, upper) = iter.size_hint();
    let mut evs = Vec::with_capacity(upper.unwrap_or(lower));
    for ent in iter {
        evs.push(MakeChunkDespawn { chunk_ent: ent, reschedule_if_beings_present: false });
    }

    writer.write_batch(evs);
}

#[allow(unused_parens)]
pub fn make_checked_chunks_despawn_if_unreferenced(
    mut reader: MessageReader<CheckIfChunkShouldDespawn>,
    activator_query: Query<(&DimensionRef, &ActivatingChunks), >,
    loaded_chunks: Res<LoadedChunks>,
    mut referenced_chunks: Local<EntityHashSet>,
    mut chunks_to_check: Local<Vec<Entity>>,
    mut writer: MessageWriter<MakeChunkDespawn>,
    mut make_despawn_msgs: Local<Vec<MakeChunkDespawn>>,
) {
    chunks_to_check.clear();
    make_despawn_msgs.clear();
    for &CheckIfChunkShouldDespawn(chunk_ent) in reader.read() {
        chunks_to_check.push(chunk_ent);
    }
    if chunks_to_check.is_empty() {
        return;
    }

    referenced_chunks.clear();
    referenced_chunks.reserve(activator_query.iter().map(|(_, a)| a.0.len()).sum());
    for (&dimension_ref, activates_chunks) in activator_query.iter() {
        referenced_chunks.extend(
            activates_chunks
                .0
                .iter()
                .filter_map(|&chunk_pos| loaded_chunks.0.get(&(dimension_ref, chunk_pos)).copied())
        );
    }

    for chunk_ent in chunks_to_check.drain(..) {
        if !referenced_chunks.contains(&chunk_ent) {
            make_despawn_msgs.push(MakeChunkDespawn::default(chunk_ent));
        }
    }

    writer.write_batch(make_despawn_msgs.drain(..));
}

#[allow(unused_parens)]
pub fn despawn_chunks(//DEJARLO DE ESTA FORMA PARA CENTRALIZAR EL SISTEMA DONDE PUEDEN DESPAWNEAR LOS CHUNKS, PARA RESPETAR EL ORDEN DE SISTEMAS
    mut cmd: Commands,
    mut despawn_reader: MessageReader<MakeChunkDespawn>,
    chunks_query: Query<(&DimensionRef, &ChunkPos, ), >,
    beings_within_chunk: Res<BeingsInCpos>,
    mut bcd_writer: MessageWriter<ChunkWithBeingsWantsDespawn>,
    mut bcd_msgs: Local<Vec<ChunkWithBeingsWantsDespawn>>,
    client_state: Res<State<ClientState>>,
) {
    let is_host = *client_state.get() == ClientState::Disconnected;

    for &MakeChunkDespawn { chunk_ent, reschedule_if_beings_present } in despawn_reader.read() {
        let Ok((&chunk_dimension, &chunk_pos, )) = chunks_query.get(chunk_ent) else {
            continue;
        };
        if is_host && reschedule_if_beings_present {
            let beings_within_chunk_count = beings_within_chunk
                .beings_in_chunk(chunk_dimension, chunk_pos)
                .map_or(0, |beings| beings.len());
            if beings_within_chunk_count > 0 {
                bcd_msgs.push(ChunkWithBeingsWantsDespawn { chunk_ent });
                continue;
            }
        }
        cmd.entity(chunk_ent).try_despawn();
    }
    bcd_writer.write_batch(bcd_msgs.drain(..));
}
#[allow(unused_parens)]
pub fn on_chunk_despawn(
    trig: On<Despawn, (Chunk, )>,
    chunk_query: Query<(&DimensionRef, &ChunkPos), ()>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    beings_within_chunk: Res<BeingsInCpos>,
    mut ub_writer: MessageWriter<FaithfulSimBeing>,
    mut ub_messages: Local<Vec<FaithfulSimBeing>>,
){
    let Ok((&chunk_dimension, &chunk_pos)) = chunk_query.get(trig.entity) else {
        error!(target: CHUNK_DESPAWN, "Chunk entity {:?} despawned but its DimensionRef or ChunkPos component is missing", trig.entity);
        loaded_chunks.0.retain(|_, chunk_entity| *chunk_entity != trig.entity);

        return;
    };
    if loaded_chunks.0.get(&(chunk_dimension, chunk_pos)).copied() != Some(trig.entity) {
        return;
    }
    loaded_chunks.0.remove(&(chunk_dimension, chunk_pos));

    let Some(beings) = beings_within_chunk.beings_in_chunk(chunk_dimension, chunk_pos) else {
        return;
    };
    ub_messages.reserve(beings.len());
    for &being_ent in beings {
        ub_messages.push(FaithfulSimBeing(being_ent));
    }
    ub_writer.write_batch(ub_messages.drain(..));


    /*
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
    */
}
