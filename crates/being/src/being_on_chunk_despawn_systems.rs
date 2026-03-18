use crate::being_components::{Being, Chasing};
use crate::nav::RetainedChasePathSnapshot;
use crate::prelude::*;
use ::being_shared::*;
use bevy::prelude::*;
use common::log_targets::BEING_SYSTEM;
use tilemap::chunking::*;

//todo hacer comando
fn unload_being_for_chunk_despawn(commands: &mut Commands, being_ent: Entity) {
    let mut entity = commands.entity(being_ent);
    entity.try_insert(BackgroundSimulated);
    entity.try_remove::<(Name, ActivateChunksAround, ActivatingChunks, RetainedChasePathSnapshot)>();
}

#[allow(unused_parens, )]
pub fn on_chunk_with_beings_attempt_unload(
    mut commands: Commands,
    mut reader: MessageReader<BeingChunkDespawned>,
    chunks_query: Query<&BeingsWithinChunk>,
    beings: Query<&Chasing, With<Being>>,
    player_targets: Query<(), (With<Being>, PlayerBeing)>,
    mut writer: MessageWriter<MakeChunkSnapshotForChaser>,
    mut messages: Local<Vec<MakeChunkSnapshotForChaser>>,
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
                messages.push(MakeChunkSnapshotForChaser(being_ent));
            }
            debug!(target: BEING_SYSTEM, "Canceled despawn for chunk {:?} because at least one resident must stay loaded", msg.chunk_ent);
            continue;
        }

        for being_ent in beings_within_chunk.iter() {
            unload_being_for_chunk_despawn(&mut commands, being_ent);
        }

        commands.entity(msg.chunk_ent).try_despawn();
        writer.write_batch(messages.drain(..));
    }
}
