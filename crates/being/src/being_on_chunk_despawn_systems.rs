use crate::being_components::{Being, Chasing};
use crate::prelude::*;

use ::being_shared::prelude::*;
use bevy::prelude::*;
use common::log_targets::BEING_SYSTEM;
use tilemap::chunking::*;
use tilemap_shared::{ChunkPos, DimensionRef};
use ::being_shared::being_shared_resources::FrozenBgSimulatedBeingsMap;



// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------

#[allow(unused_parens)]
pub fn freeze_being(mut cmd: Commands,
    mut reader: MessageReader<UnloadBeing>,
    query: Query<(&DimensionRef, &ChunkPos), (With<Being>, )>,
    mut frozen_bg_simulated_being_map: ResMut<FrozenBgSimulatedBeingsMap>,
) {
    for unloaded_being in reader.read() {
        let Ok((dimension_ref, chunk_pos)) = query.get(unloaded_being.0) else {
            continue;
        };

        frozen_bg_simulated_being_map.0.entry((*dimension_ref, *chunk_pos)).or_default().push(unloaded_being.0);
        let mut entity = cmd.entity(unloaded_being.0);
        entity.try_insert(BackgroundSimulated);
        entity.try_remove::<RemoveOnFreeze>();
    }

}


#[allow(unused_parens, )]
pub fn on_chunk_with_beings_attempt_unload(
    mut commands: Commands,
    mut reader: MessageReader<ChunkWithBeingsWantsDespawn>,
    chunks_query: Query<&BeingsWithinChunk>,
    beings: Query<&Chasing, With<Being>>,
    player_targets: Query<(), (With<Being>, PlayerBeing)>,
    mut mcsfc_writer: MessageWriter<MakeChunkSnapshotForChaser>,
    mut mcsfc_messages: Local<Vec<MakeChunkSnapshotForChaser>>,
    mut chunks_to_despawn: Local<Vec<MakeChunkDespawn>>,
    mut chunk_despawn_writer: MessageWriter<MakeChunkDespawn>,
) {
    for msg in reader.read() {
        let Ok(beings_within_chunk) = chunks_query.get(msg.chunk_ent) else {
            continue;
        };

        let should_cancel = beings_within_chunk.iter().any(|being_ent| {
            let Ok(chaser) = beings.get(being_ent) else {
                return false;
            };
            player_targets.get(chaser.target).is_ok()
        });

        if should_cancel {
            for being_ent in beings_within_chunk.iter() {
                let Ok(chaser) = beings.get(being_ent) else {
                    continue;
                };
                if player_targets.get(chaser.target).is_err() {
                    continue;
                }
                mcsfc_messages.push(MakeChunkSnapshotForChaser(being_ent));
            }
            debug!(target: BEING_SYSTEM, "Canceled despawn for chunk {:?} because at least one resident must stay loaded", msg.chunk_ent);
            continue;
        }
        chunks_to_despawn.push(MakeChunkDespawn::new_no_delegate_if_beings(msg.chunk_ent));
    }
    mcsfc_writer.write_batch(mcsfc_messages.drain(..));
    chunk_despawn_writer.write_batch(chunks_to_despawn.drain(..));
}
