use crate::being_components::Being;
use crate::being_bundles::{ReinsertOnUnfreeze, RemoveOnFreeze};
use crate::being_messages::MakeChunkSnapshotForChaser;

use ::being_shared::*;
use bevy::prelude::*;
use common::log_targets::BEING_SYSTEM;
use sprite_shared::HeldSprites;
use tilemap_shared::{ChunkLoaded, ChunkWithBeingsWantsDespawn, LoadedMacroChunks, MakeChunkDespawn};
use tilemap::chunking::*;
use tilemap_shared::{ChunkPos, DimensionRef};
use ::being_shared::being_shared_resources::FrozenBgSimulatedBeingsMap;



#[allow(unused_parens)]
pub fn freeze_being(mut cmd: Commands,
    mut reader: MessageReader<UnloadBeing>,
    query: Query<(&DimensionRef, &ChunkPos, &HeldSprites), (With<Being>, )>,
    mut frozen_bg_simulated_being_map: ResMut<FrozenBgSimulatedBeingsMap>,
    macro_chunk_pos_map: Res<LoadedMacroChunks>,
) {
    for unloaded_being in reader.read() {
        let Ok((&dim, &chunk_pos, held_sprites)) = query.get(unloaded_being.0) else {
            continue;
        };

        for sprite_ent in held_sprites.iter() {
            cmd.entity(sprite_ent).try_despawn();
        }
        frozen_bg_simulated_being_map.0.entry((dim, chunk_pos)).or_default().push(unloaded_being.0);

        let macro_chunk_pos = chunk_pos.to_macrochunk_pos();
        let Some(&macro_chunk_ent) = macro_chunk_pos_map.0.get(&(dim, macro_chunk_pos)) else {
            error!("macro chunk not found for being on freeze");
            continue;
        };


        let mut entity = cmd.entity(unloaded_being.0);
        entity.try_insert(BgSimulatedIn{ macro_chunk_ent });
        entity.try_remove::<RemoveOnFreeze>();
    }

}

#[allow(unused_parens, )]
pub fn unfreeze_beings_on_chunk_load(
    mut cmd: Commands,
    mut reader: MessageReader<ChunkLoaded>,
    mut frozen_bg_simulated_being_map: ResMut<FrozenBgSimulatedBeingsMap>,
) {
    let mut vec_ins_batch = Vec::new();
    for &msg in reader.read() {
        let Some(being_ents) = frozen_bg_simulated_being_map.0.remove(&(msg.dimension, msg.chunk_pos)) else {
            continue;
        };

        debug!(
            target: BEING_SYSTEM,
            "Restoring {} frozen beings for loaded chunk {:?} in {:?}",
            being_ents.len(),
            msg.chunk_pos,
            msg.dimension,
        );

        for being_ent in being_ents {
            cmd.entity(being_ent).try_remove::<(BgSimulatedIn, Unloaded)>();
            let ins = ReinsertOnUnfreeze::new(msg);
            vec_ins_batch.push((being_ent, ins));
        }
    }
    cmd.try_insert_batch(vec_ins_batch);
}

#[allow(unused_parens, )]
pub fn on_chunk_with_beings_attempt_unload(
    mut reader: MessageReader<ChunkWithBeingsWantsDespawn>,
    chunks_query: Query<&BeingsWithinChunk>,
    chaser_beings: Query<&Chasing, With<Being>>,
    prevents_chunk_unloading_query: Query<(), (With<PreventsChunkUnloading>, )>,
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
            if prevents_chunk_unloading_query.get(being_ent).is_ok() {
                return true;
            }
            let Ok(chaser) = chaser_beings.get(being_ent) else {
                return false;
            };
            player_targets.get(chaser.target).is_ok()
        });

        if should_cancel {
            for being_ent in beings_within_chunk.iter() {
                if prevents_chunk_unloading_query.get(being_ent).is_ok() {
                    debug!(target: BEING_SYSTEM, "Canceled despawn for chunk {:?} because being {:?} prevents chunk unloading", msg.chunk_ent, being_ent);
                    continue;
                }
                let Ok(chaser) = chaser_beings.get(being_ent) else {
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
