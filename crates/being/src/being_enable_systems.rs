use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;
use common::common_components::StrId;
use common::file_logging::file_log;
use common::log_targets::BEING_SYSTEM;
use param_sets::BlockingTileParamSet;
use tilemap::{
    chunking::chunking_components::Chunk,
    terrain::terrgen_messages::ChunkTerrainBuilt,
};

use crate::being_bundles::ReinsertOnUnfreeze;
use ::being_shared::*;
use ::being_shared::being_shared_resources::FrozenBgSimulatedBeingsMap;
use ::tilemap_shared::*;

#[allow(unused_parens, )]
pub fn activate_beings_in_first_time_loaded_chunks(
    mut cmd: Commands,
    mut built_chunks: MessageReader<ChunkTerrainBuilt>,
    built_chunk_query: Query<(&DimensionRef, &ChunkPos), With<Chunk>>,
    mut param_set: BlockingTileParamSet,
    mut pending_enable_by_cpos: ResMut<BeingsToEnableOnChunkLoad>,
    spawn_tile_tags_query: Query<(
        Option<&WhitelistedSpawnTileTags>,
        Option<&BlacklistedSpawnTileTags>,
    )>,
    being_spawn_tag_extension_query: Query<(
        Has<DontExtendBitSpawnWhitelist>,
        Has<DontExtendBitSpawnBlacklist>,
        Has<DontExtendRaceSpawnWhitelist>,
        Has<DontExtendRaceSpawnBlacklist>,
    )>,
    bit_race_query: Query<&RaceRef>,
    being_str_id_query: Query<&StrId>,
    mut whitelisted_spawn_tile_tags: Local<WhitelistedSpawnTileTags>,
    mut blacklisted_spawn_tile_tags: Local<BlacklistedSpawnTileTags>,
) {
    let mut activated_beings = 0usize;
    let mut touched_chunks = 0usize;
    for built_chunk in built_chunks.read() {
        let Ok((&dim_ref, &chunk_pos)) = built_chunk_query.get(built_chunk.chunk_ent) else {
            error!(target: BEING_SYSTEM, "Natural spawn unfreeze got ChunkTerrainBuilt for missing chunk entity {:?}", built_chunk.chunk_ent);
            continue;
        };
        let Some(being_ents) = pending_enable_by_cpos.by_chunk.get_mut(&(dim_ref, chunk_pos)) else {
            trace!(target: BEING_SYSTEM, "Natural spawn unfreeze got built chunk {:?} in {:?} but no pending wildlife reservation existed", chunk_pos, dim_ref);
            continue;
        };
        touched_chunks += 1;
        being_ents.retain(|&being_ent| {
            let _being_str_id = being_str_id_query.get(being_ent).ok();
            let bit_ref = param_set.get_being_bit_ref(being_ent);
            let race_ref = param_set.get_being_race_ref(being_ent);
            Being::select_spawn_tile_tag_filters(
                being_ent,
                bit_ref.map(|bit_ref| bit_ref.0),
                race_ref.map(|race_ref| race_ref.0),
                &bit_race_query,
                &spawn_tile_tags_query,
                &being_spawn_tag_extension_query,
                &mut whitelisted_spawn_tile_tags,
                &mut blacklisted_spawn_tile_tags,
            );
            let search_start_gpos = param_set.gpos_query.get(being_ent).copied().unwrap_or_else(|_| chunk_pos.center_gpos());
            let Some(found_gpos) = param_set.find_closest_allowed_gpos(
                dim_ref,
                search_start_gpos,
                being_ent,
                &whitelisted_spawn_tile_tags.0,
                &blacklisted_spawn_tile_tags.0,
            ) else {
                return true;
            };
            cmd.entity(being_ent)
                .try_insert(found_gpos)
                .try_remove::<(Disabled, )>();
            activated_beings += 1;
            false
        });
        if being_ents.is_empty() {
            pending_enable_by_cpos.by_chunk.remove(&(dim_ref, chunk_pos));
        }
    }

    if activated_beings > 0 {
        debug!(target: BEING_SYSTEM, "Natural spawn activated {} reserved wildlife beings", activated_beings);
    }
    file_log(
        "being",
        "host",
        &format!("activate touched_chunks={touched_chunks} activated_beings={activated_beings}"),
    );
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
