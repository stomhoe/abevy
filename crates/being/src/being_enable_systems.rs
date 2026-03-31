use bevy::ecs::entity_disabling::Disabled;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::system::SystemParam;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use common::common_components::StrId;
use common::file_logging::file_log;
use common::log_targets::BEING_SYSTEM;
use ::param_sets::*;
use tilemap::{
    chunking::chunking_components::Chunk,
    terrain::terrgen_messages::ChunkTerrainBuilt,
};

use crate::being_bundles::ReinsertOnUnfreeze;
use ::being_shared::*;
use ::being_shared::being_shared_resources::FrozenBgSimulatedBeingsMap;
use ::tilemap_shared::*;

#[derive(Default)]
struct SpawnTileTagsGroup {
    whitelist: WhitelistedTags,
    blacklist: BlacklistedTags,
    entities: Vec<Entity>,
}

#[derive(SystemParam)]
pub(crate) struct ActivateFirstLoadQueries<'w, 's> {
    built_chunk_query: Query<'w, 's, (&'static DimensionRef, &'static ChunkPos), (With<Chunk>, )>,
    spawn_tile_tags_query: Query<'w, 's, (
        Option<&'static WhitelistedSpawnTileTags>,
        Option<&'static BlacklistedSpawnTileTags>,
    ), >,
    being_spawn_tag_extension_query: Query<'w, 's, (
        Has<DontExtendBitSpawnWhitelist>,
        Has<DontExtendBitSpawnBlacklist>,
        Has<DontExtendRaceSpawnWhitelist>,
        Has<DontExtendRaceSpawnBlacklist>,
    ), >,
    pending_group_policy_query: Query<'w, 's, (
        &'static PendingNaturalSpawnGroupId,
        &'static PendingNaturalSpawnPlacementPolicy,
    ), >,
    bit_race_query: Query<'w, 's, &'static RaceRef, >,
    being_str_id_query: Query<'w, 's, &'static StrId, >,
}

#[derive(SystemParam)]
pub(crate) struct ActivateFirstLoadLocals<'s> {
    whitelisted_spawn_tile_tags: Local<'s, WhitelistedSpawnTileTags>,
    blacklisted_spawn_tile_tags: Local<'s, BlacklistedSpawnTileTags>,
    grouped_entities_by_key: Local<'s, HashMap<u64, Vec<Entity>>>,
    grouped_entities: Local<'s, Vec<Entity>>,
    ungrouped_entities: Local<'s, Vec<Entity>>,
    keep_pending_entities: Local<'s, Vec<Entity>>,
    grouped_tag_sets: Local<'s, Vec<SpawnTileTagsGroup>>,
    spawn_positions: Local<'s, Vec<GlobalTilePos>>,
    claimed_positions: Local<'s, HashSet<GlobalTilePos>>,
    assigned_spawn_positions: Local<'s, Vec<(Entity, GlobalTilePos)>>,
    assigned_entities: Local<'s, EntityHashSet>,
}

#[allow(unused_parens, )]
pub fn activate_beings_in_first_time_loaded_chunks(
    mut cmd: Commands,
    mut built_chunks: MessageReader<ChunkTerrainBuilt>,
    mut param_set: BlockingTileParamSet,
    mut pending_enable_by_cpos: ResMut<BeingsToEnableOnChunkLoad>,
    queries: ActivateFirstLoadQueries,
    mut locals: ActivateFirstLoadLocals,
) {
    let mut activated_beings = 0usize;
    let mut touched_chunks = 0usize;
    for built_chunk in built_chunks.read() {
        let Ok((&dim_ref, &chunk_pos)) = queries.built_chunk_query.get(built_chunk.chunk_ent) else {
            error!(target: BEING_SYSTEM, "Natural spawn unfreeze got ChunkTerrainBuilt for missing chunk entity {:?}", built_chunk.chunk_ent);
            continue;
        };
        let Some(being_ents) = pending_enable_by_cpos.by_chunk.get_mut(&(dim_ref, chunk_pos)) else {
            trace!(target: BEING_SYSTEM, "Natural spawn unfreeze got built chunk {:?} in {:?} but no pending wildlife reservation existed", chunk_pos, dim_ref);
            continue;
        };
        touched_chunks += 1;
        let mut pending_entities = std::mem::take(being_ents);
        locals.grouped_entities_by_key.clear();
        locals.ungrouped_entities.clear();
        locals.keep_pending_entities.clear();
        locals.grouped_entities_by_key.reserve(pending_entities.len());
        locals.ungrouped_entities.reserve(pending_entities.len());
        locals.keep_pending_entities.reserve(pending_entities.len());
        for being_ent in pending_entities.drain() {
            if let Ok((&group_key, _)) = queries.pending_group_policy_query.get(being_ent) {
                locals.grouped_entities_by_key.entry(group_key.0).or_default().push(being_ent);
                continue;
            }
            locals.ungrouped_entities.push(being_ent);
        }

        for being_ent in locals.ungrouped_entities.drain(..) {
            let _being_str_id = queries.being_str_id_query.get(being_ent).ok();
            let bit_ref = param_set.get_being_bit_ref(being_ent);
            let race_ref = param_set.get_being_race_ref(being_ent);
            Being::select_spawn_tile_tag_filters(
                being_ent,
                bit_ref.map(|bit_ref| bit_ref.0),
                race_ref.map(|race_ref| race_ref.0),
                &queries.bit_race_query,
                &queries.spawn_tile_tags_query,
                &queries.being_spawn_tag_extension_query,
                &mut locals.whitelisted_spawn_tile_tags,
                &mut locals.blacklisted_spawn_tile_tags,
            );
            let search_start_gpos = param_set.gpos_query.get(being_ent).copied().unwrap_or_else(|_| chunk_pos.center_gpos());
            let Some(found_gpos) = param_set.find_closest_allowed_gpos(
                dim_ref,
                search_start_gpos,
                being_ent,
                GposSearchConfig::default(),
                &locals.whitelisted_spawn_tile_tags.0,
                &locals.blacklisted_spawn_tile_tags.0,
            ) else {
                locals.keep_pending_entities.push(being_ent);
                continue;
            };
            cmd.entity(being_ent)
                .try_insert(found_gpos)
                .try_remove::<(Disabled, )>();
            activated_beings += 1;
        }

        for (_, entities) in locals.grouped_entities_by_key.drain() {
            locals.grouped_entities.clear();
            locals.grouped_entities.extend(entities);
            let Some(&group_seed_ent) = locals.grouped_entities.first() else {
                continue;
            };
            let Ok((_, &group_policy)) = queries.pending_group_policy_query.get(group_seed_ent) else {
                locals.keep_pending_entities.extend(locals.grouped_entities.drain(..));
                continue;
            };
            locals.grouped_tag_sets.clear();
            locals.grouped_tag_sets.reserve(locals.grouped_entities.len());
            for &being_ent in locals.grouped_entities.iter() {
                let bit_ref = param_set.get_being_bit_ref(being_ent);
                let race_ref = param_set.get_being_race_ref(being_ent);
                Being::select_spawn_tile_tag_filters(
                    being_ent,
                    bit_ref.map(|bit_ref| bit_ref.0),
                    race_ref.map(|race_ref| race_ref.0),
                    &queries.bit_race_query,
                    &queries.spawn_tile_tags_query,
                    &queries.being_spawn_tag_extension_query,
                    &mut locals.whitelisted_spawn_tile_tags,
                    &mut locals.blacklisted_spawn_tile_tags,
                );
                let whitelist = locals.whitelisted_spawn_tile_tags.0.clone();
                let blacklist = locals.blacklisted_spawn_tile_tags.0.clone();
                let mut matching_group_idx = None;
                for (idx, group) in locals.grouped_tag_sets.iter().enumerate() {
                    if group.whitelist == whitelist && group.blacklist == blacklist {
                        matching_group_idx = Some(idx);
                        break;
                    }
                }
                if let Some(group_idx) = matching_group_idx {
                    locals.grouped_tag_sets[group_idx].entities.push(being_ent);
                    continue;
                }
                locals.grouped_tag_sets.push(SpawnTileTagsGroup {
                    whitelist,
                    blacklist,
                    entities: vec![being_ent],
                });
            }

            locals.assigned_spawn_positions.clear();
            locals.assigned_spawn_positions.reserve(locals.grouped_entities.len());
            locals.claimed_positions.clear();
            locals.claimed_positions.reserve(locals.grouped_entities.len());
            for tags_group in locals.grouped_tag_sets.iter() {
                let Some(&tags_seed_ent) = tags_group.entities.first() else {
                    continue;
                };
                let needed_count = tags_group.entities.len();
                locals.spawn_positions.clear();
                locals.spawn_positions.reserve(needed_count);
                let hard_max_radius_tiles = group_policy.hard_max_radius_tiles_for_count(needed_count);
                param_set.find_allowed_gposes_in_area(
                    dim_ref,
                    group_policy.anchor_gpos,
                    needed_count,
                    Some(group_policy.preferred_radius_tiles),
                    Some(hard_max_radius_tiles),
                    true,
                    group_policy.only_same_island,
                    tags_seed_ent,
                    &tags_group.whitelist,
                    &tags_group.blacklist,
                    &mut locals.spawn_positions,
                );
                let mut assigned_count = 0usize;
                for &gpos in locals.spawn_positions.iter() {
                    if !locals.claimed_positions.insert(gpos) {
                        continue;
                    }
                    locals.assigned_spawn_positions.push((tags_group.entities[assigned_count], gpos));
                    assigned_count += 1;
                    if assigned_count >= needed_count {
                        break;
                    }
                }
            }
            if locals.assigned_spawn_positions.len() < locals.grouped_entities.len() {
                trace!(target: BEING_SYSTEM, "Natural spawn activation in {:?} around {} found only {} allowed gposes for grouped pending beings count {}", dim_ref, group_policy.anchor_gpos, locals.assigned_spawn_positions.len(), locals.grouped_entities.len());
            }
            locals.assigned_entities.clear();
            locals.assigned_entities.reserve(locals.assigned_spawn_positions.len());
            for (being_ent, gpos) in locals.assigned_spawn_positions.drain(..) {
                locals.assigned_entities.insert(being_ent);
                cmd.entity(being_ent)
                    .try_insert(gpos)
                    .try_remove::<(
                        Disabled,
                        PendingNaturalSpawnGroupId,
                        PendingNaturalSpawnPlacementPolicy,
                    )>();
                activated_beings += 1;
            }
            for being_ent in locals.grouped_entities.drain(..) {
                if !locals.assigned_entities.contains(&being_ent) {
                    locals.keep_pending_entities.push(being_ent);
                }
            }
        }

        being_ents.extend(locals.keep_pending_entities.drain(..));
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
