use crate::being_bundles::*;
use crate::being_messages::MakeChunkSnapshotForChaser;

use ::being_shared::*;
use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use common::log_targets::BEING_SYSTEM;
use faction_shared::BelongsToAPlayerFaction;
use sprite_shared::HeldSprites;
use tilemap_shared::{ChunkLoaded, ChunkWithBeingsWantsDespawn, LoadChunksAround, LoadedMacroChunks, MakeChunkDespawn};
use tilemap::chunking::*;
use tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos};
use ::being_shared::being_shared_resources::FrozenBgSimulatedBeingsMap;



#[allow(unused_parens)]
pub fn faithful_sim_being(mut cmd: Commands,
    mut reader: MessageReader<FaithfulSimBeing>,
    query: Query<(&DimensionRef, &ChunkPos, &HeldSprites), (With<Being>, )>,
    mut frozen_bg_simulated_being_map: ResMut<FrozenBgSimulatedBeingsMap>,
    macro_chunk_pos_map: Res<LoadedMacroChunks>,
) {
    for unloaded_being in reader.read() {
        let Ok((&dim, &chunk_pos, _held_sprites)) = query.get(unloaded_being.0) else {
            continue;
        };

        //for sprite_ent in held_sprites.iter() {cmd.entity(sprite_ent).try_despawn();}
        frozen_bg_simulated_being_map.0.entry((dim, chunk_pos)).or_default().push(unloaded_being.0);

        let macro_chunk_pos = chunk_pos.to_macrochunk_pos();
        let Some(&_macro_chunk_ent) = macro_chunk_pos_map.0.get(&(dim, macro_chunk_pos)) else {
            error!("macro chunk not found for being on freeze");
            continue;
        };


        let mut entity = cmd.entity(unloaded_being.0);
        //entity.try_insert(BgSimulatedIn{ macro_chunk_ent });
        entity.try_remove::<RemoveOnEnterSemiRealSimMode>();
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
            vec_ins_batch.push((being_ent, ReinsertOnUnfreeze::new(msg)));
        }
    }
    cmd.try_insert_batch_if_new(vec_ins_batch);
}

#[allow(unused_parens, )]
pub fn on_chunk_with_beings_attempt_unload(
    mut reader: MessageReader<ChunkWithBeingsWantsDespawn>,
    chunks_query: Query<&BeingsWithinChunk>,
    chaser_beings: Query<&Chasing, With<Being>>,
    prevents_chunk_unloading_query: Query<(), (Or<(With<PreventsChunkUnloading>, With<LoadChunksAround>),>,)>,
    player_targets: Query<(), (PlayerBeing)>,
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
pub const MAX_LOADED_BEINGS: usize = 30;
#[allow(unused_parens, )]
pub fn cull_loaded_beings_far_from_humans(
    mut cmd: Commands,
    non_hc_beings_query: Query<(Entity, &GlobalTilePos, &DimensionRef, ), (With<Being>, Without<Unloaded>, Without<BelongsToAPlayerFaction>, Without<HumanControlled>),>,
    hc_beings_query: Query<(&GlobalTilePos, &DimensionRef, ), (With<Being>, With<HumanControlled>, Without<Unloaded>),>,
    mut human_positions_by_dim: Local<HashMap<DimensionRef, Vec<GlobalTilePos>>>,
) {
    let non_hc_iter = non_hc_beings_query.iter();
    let non_hc_count = non_hc_iter.size_hint().1.unwrap_or(non_hc_iter.size_hint().0);
    let hc_iter = hc_beings_query.iter();
    let hc_count = hc_iter.size_hint().1.unwrap_or(hc_iter.size_hint().0);
    let loaded_count = non_hc_count + hc_count;
    if loaded_count <= MAX_LOADED_BEINGS {
        return;
    }

    human_positions_by_dim.clear();
    for (human_gpos, human_dim, ) in hc_beings_query.iter() {
        human_positions_by_dim.entry(*human_dim).or_default().push(*human_gpos);
    }

    let mut farthest_non_hc: Option<(Entity, f32, GlobalTilePos, DimensionRef)> = None;
    for (being_ent, being_gpos, being_dim, ) in non_hc_beings_query.iter() {
        let Some(human_positions) = human_positions_by_dim.get(being_dim) else {
            continue;
        };
        let Ok(human_count_u32) = u32::try_from(human_positions.len()) else {
            continue;
        };
        if human_count_u32 == 0 {
            continue;
        }
        let avg_distance = human_positions
            .iter()
            .map(|human_gpos| being_gpos.taxicab_tile_distance(*human_gpos))
            .sum::<f32>()
            / human_count_u32 as f32;
        if farthest_non_hc
            .as_ref()
            .map(|(_, farthest_avg, _, _)| avg_distance > *farthest_avg)
            .unwrap_or(true)
        {
            farthest_non_hc = Some((being_ent, avg_distance, *being_gpos, *being_dim));
        }
    }

    let Some((being_ent, avg_distance, being_gpos, being_dim)) = farthest_non_hc else {
        return;
    };
    debug!(target: BEING_SYSTEM, "Despawning loaded non-human being {:?} at {:?} in {:?}; loaded_count={} exceeded threshold=300 and avg_distance_to_humans={:.2}", being_ent, being_gpos, being_dim, loaded_count, avg_distance);
    cmd.entity(being_ent).try_despawn();
}
