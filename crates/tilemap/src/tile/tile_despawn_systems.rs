use crate::{
    tile::{tile_components::*, tile_messages::*},
    tilemap_resources::*,
};
use ::sprite_shared::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use common::{AnyDisabling, common_tag_components::TagSet};
use game_common::game_common_components::*;
use ::tilemap_shared::*;

fn should_delete_tile(
    spec: &DeleteOtherTiles,
    target_z: &AcZ,
    target_tags: Option<&TagSet>,
) -> bool {
    if !spec.targeted_z.is_empty() {
        if !spec.targeted_z.contains(target_z) {
            return false;
        }
        if let Some(tags) = target_tags {
            if spec.spared_tags.intersects(tags) {
                return false;
            }
        }
        return true;
    }
    if !spec.targeted_tags.is_empty() {
        let Some(tags) = target_tags else {
            return false;
        };
        if !spec.targeted_tags.intersects(tags) {
            return false;
        }
        if spec.spared_z.contains(target_z) {
            return false;
        }
        return true;
    }
    if spec.spared_z.contains(target_z) {
        return false;
    }
    if let Some(tags) = target_tags {
        if spec.spared_tags.intersects(tags) {
            return false;
        }
    }
    true
}

#[allow(unused_parens)]
pub fn on_spritetile_despawn(
    trig: On<Despawn, (Tile, Transform, SpriteTile)>,
    query: Query<(&DimensionRef, &GlobalTilePos, &EntityZeroRef), (Without<TilemapId>, Without<TilePos>, Without<EntityZero>, AnyDisabling)>,
    ezero_size_query: Query<&SizeInTiles, (With<EntityZero>, common::AnyDisabling)>,
    mut spritetiles_at_gpos: ResMut<SpriteTilesAtGpos>,
) {
    let Ok((&dim_ref, &gpos, ezero_ref)) = query.get(trig.entity) else {
        return;
    };
    let size = ezero_size_query.get(ezero_ref.0).copied().unwrap_or_default();
    spritetiles_at_gpos.remove_tile(dim_ref, gpos, trig.entity, size);
}

pub fn despawn_if_not_excepted(
    ezero_query: Query<
        (Option<&AcZ>, Option<&DeleteOtherTiles>, Option<&TagSet>, Option<&SizeInTiles>),
        (With<EntityZero>, common::AnyDisabling),
    >,
    query: Query<
        (
            Entity, &DimensionRef, &GlobalTilePos, &EntityZeroRef,
            Option<&TagSet>, Option<&DeleteOtherTiles>,
        ),
        (common::AnyDisabling, Without<EntityZero>),
    >,
    otile_query: Query<
        (&EntityZeroRef, Option<&TagSet>, Option<&DeleteOtherTiles>),
        (common::AnyDisabling, Without<EntityZero>),
    >,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
    registered_positions: Res<ImportantRegisteredPositions>,
    params: TileGatheringParamSet,
    mut otile_ents: Local<Vec<Entity>>,
    mut checked_ents: Local<HashSet<Entity>>,
    mut writer: MessageWriter<SafeDespawn>,
    mut msgs: Local<Vec<SafeDespawn>>,
) {
    query.iter_many(changed_pos.read().map(|msg| msg.entity)).for_each(|(newtile_ent, &dim, &gpos, ezero_ref, newtile_tag_hashset, newtile_delete_others_excp)| {
        let Ok((newtile_z, ezero_newtile_delete_others_excp, ezero_newtile_tagset, newtile_size)) = ezero_query.get(ezero_ref.0) else {
            warn_once!(target: common::DEBUG_TILE, "Failed to get EntityZero for tile entity {:?}, skipping despawn check", newtile_ent);
            return;
        };
        let Some(newtile_z) = newtile_z else {
            warn_once!(target: "tilemap", "Tile entity {:?} has no AcZ, skipping despawn check", newtile_ent);
            return;
        };
        let newtile_delete_others_excp = newtile_delete_others_excp.or(ezero_newtile_delete_others_excp);
        let scan_radius = newtile_delete_others_excp.map(|s| s.extra_radius as i32).unwrap_or_default();
        let newtile_size = newtile_size.copied().unwrap_or_default().inner().as_ivec2();
        checked_ents.clear();
        for y in (gpos.0.y - scan_radius)..=(gpos.0.y + newtile_size.y - 1 + scan_radius) {
            for x in (gpos.0.x - scan_radius)..=(gpos.0.x + newtile_size.x - 1 + scan_radius) {
                params.gather_tiles_at(&mut *otile_ents, dim, GlobalTilePos::new(x, y));
                for ent in otile_ents.drain(..) {
                    checked_ents.insert(ent);
                }
            }
        }
        checked_ents.drain().for_each(|otile_ent| {
            if otile_ent == newtile_ent {
                return;
            }
            let Ok((otile_ezero_ref, otile_tag_hashset, otile_delete_others_excp)) = otile_query.get(otile_ent) else {
                trace!(target: "tilemap", "Failed to get prev tile entity {:?}, skipping despawn check", otile_ent);
                return;
            };
            let Ok((otile_z, ezero_otile_delete_others_excp, ezero_otile_tagset, _)) = ezero_query.get(otile_ezero_ref.0) else {
                trace!(target: "tilemap", "Failed to get EntityZero for tile entity {:?}, skipping despawn check", otile_ent);
                return;
            };
            let Some(otile_z) = otile_z else {
                trace!(target: "tilemap", "Tile entity {:?} has no AcZ, skipping despawn check", otile_ent);
                return;
            };
            if let Some(newtile_delete_others_excp) = newtile_delete_others_excp {
                let otile_tags = otile_tag_hashset.or(ezero_otile_tagset);
                if should_delete_tile(newtile_delete_others_excp, otile_z, otile_tags) {
                    trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to new tile entity {:?}", otile_ent, gpos, dim, newtile_ent);
                    if !registered_positions.is_pos_registered(*otile_ezero_ref, dim, gpos) && !registered_positions.exempted.contains(&otile_ent) {
                        msgs.push(SafeDespawn(otile_ent));
                    }
                    return;
                }
            }
            let otile_delete_others_excp = otile_delete_others_excp.or(ezero_otile_delete_others_excp);
            if let Some(otile_delete_others_excp) = otile_delete_others_excp {
                let newtile_tags = newtile_tag_hashset.or(ezero_newtile_tagset);
                if should_delete_tile(otile_delete_others_excp, newtile_z, newtile_tags) {
                    trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to old tile entity {:?}", newtile_ent, gpos, dim, otile_ent);
                    if !registered_positions.is_pos_registered(*ezero_ref, dim, gpos) && !registered_positions.exempted.contains(&newtile_ent) {
                        msgs.push(SafeDespawn(newtile_ent));
                    }
                }
            }
        });
    });
    writer.write_batch(msgs.drain(..));
}

pub fn safe_despawn_tile_at(
    mut cmd: Commands,
    mut reader: MessageReader<SafeDespawn>,
    mut recheck_writer: MessageWriter<RecheckTileAdjacency>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_children: Query<&Tilemaps>,
    mut tilemap_query: Query<(&mut TileStorage, &HashIdToTexIndex)>,
    tile_query: Query<(&DimensionRef, &GlobalTilePos), (With<Tile>, common::AnyDisabling)>,
    mut rechecks: Local<Vec<RecheckTileAdjacency>>,
) {
    for &SafeDespawn(tile_ent) in reader.read() {
        let Ok((&dim, &gpos)) = tile_query.get(tile_ent) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };

        cmd.entity(tile_ent).try_despawn();
        rechecks.push(RecheckTileAdjacency { dim, gpos });
        RecheckTileAdjacency::append_all_adjacent_pos(&mut rechecks, dim, gpos);

        let chunk_pos = gpos.to_chunkpos();
        let Some(&chunk_ent) = loaded_chunks.0.get(&(dim, chunk_pos)) else {
            continue;
        };
        let Ok(tilemaps) = chunk_children.get(chunk_ent) else {
            continue;
        };
        for &tmap_ent in tilemaps.entities() {
            let Ok((mut storage, ..)) = tilemap_query.get_mut(tmap_ent) else {
                continue;
            };
            let tpos = gpos.to_tilepos();
            let Some(found_tile_ent) = storage.get(&tpos) else {
                continue;
            };
            if tile_ent == found_tile_ent {
                storage.remove(&tpos);
            }
        }
    }
    recheck_writer.write_batch(rechecks.drain(..));
}
