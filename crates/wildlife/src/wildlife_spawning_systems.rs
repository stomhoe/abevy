use bevy::ecs::entity_disabling::Disabled;
use bevy::{
    prelude::*,
};
use ::being_shared::*;
use common::file_logging::file_log;
use common::common_components::StrId;
use common::log_targets::WILDLIFE_SYSTEM;
use param_sets::BlockingTileParamSet;
use tilemap::{
    chunking::chunking_components::Chunk,
    terrain::terrgen_messages::*,
};
use ::tilemap_shared::*;

use crate::wildlife_spawning_helpers::*;

#[allow(unused_parens, )]
pub fn activate_beings_in_first_time_loaded_chunks(
    mut cmd: Commands,
    mut built_chunks: MessageReader<ChunkTerrainBuilt>,
    built_chunk_query: Query<(&DimensionRef, &ChunkPos), With<Chunk>>,
    mut param_set: BlockingTileParamSet,
    mut pending_wildlife_by_chunk: ResMut<BeingsToEnableOnChunkLoad>,
    mut being_query: Query<
        (
            &DimensionRef,
            &NaturalSpawnOrigin,
        ),
        (With<Disabled>, )
    >,
    spawn_tile_tags_query: Query<(
        Option<&WhitelistedSpawnTileTags>,
        Option<&BlacklistedSpawnTileTags>,
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
            error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze got ChunkTerrainBuilt for missing chunk entity {:?}", built_chunk.chunk_ent);
            continue;
        };
        let key = (dim_ref, chunk_pos);
        let Some(being_ents) = pending_wildlife_by_chunk.by_chunk.get_mut(&key) else {
            trace!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze got built chunk {:?} in {:?} but no pending wildlife reservation existed", chunk_pos, dim_ref);
            continue;
        };
        touched_chunks += 1;
        being_ents.retain(|&being_ent| {
            let being_str_id = being_str_id_query.get(being_ent).ok();
            let Ok((&dim_ref, &NaturalSpawnOrigin(home_chunk), )) = being_query.get_mut(being_ent) else {
                error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze could not fetch reserved being {:?} in {:?} for chunk {:?}; likely despawned before activation", being_str_id.map(StrId::as_str).unwrap_or("<no-strid>"), dim_ref, key.1);
                return false;
            };

            let bit_ref = param_set.get_being_bit_ref(being_ent);
            let race_ref = param_set.get_being_race_ref(being_ent);
            Being::collect_spawn_tile_tag_filters(
                bit_ref.map(|bit_ref| bit_ref.0),
                race_ref.map(|race_ref| race_ref.0),
                &spawn_tile_tags_query,
                |bit_ent| bit_race_query.get(bit_ent).ok().map(|race_ref| race_ref.0),
                &mut whitelisted_spawn_tile_tags,
                &mut blacklisted_spawn_tile_tags,
            );
            let Ok(&gpos) = param_set.gpos_query.get(being_ent) else {
                error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze could not read gpos for reserved being {:?} in {:?} for chunk {:?}", being_str_id.map(StrId::as_str).unwrap_or("<no-strid>"), dim_ref, key.1);
                return false;
            };
            let Some(found_gpos) = param_set.find_closest_allowed_gpos(
                dim_ref,
                gpos,
                being_ent,
                &whitelisted_spawn_tile_tags.0,
                &blacklisted_spawn_tile_tags.0,
            ) else {
                error!(target: WILDLIFE_SYSTEM, "Natural spawn unfreeze found no valid tile for reserved being {:?} in {:?} around chunk {:?}; it will stay frozen for now", being_str_id.map(StrId::as_str).unwrap_or("<no-strid>"), dim_ref, home_chunk);
                return true;
            };
            cmd.entity(being_ent)
                .try_insert(found_gpos)
                .try_remove::<(Disabled, )>();
            activated_beings += 1;
            false
        });
        if being_ents.is_empty() {
            pending_wildlife_by_chunk.by_chunk.remove(&key);
        }
    }

    if activated_beings > 0 {
        debug!(target: WILDLIFE_SYSTEM, "Natural spawn activated {} reserved wildlife beings", activated_beings);
    }
    file_log(
        "wildlife",
        "host",
        &format!("activate touched_chunks={touched_chunks} activated_beings={activated_beings}"),
    );
}
