use crate::{
    tile::{tile_components::*, tile_delete_others_helpers::*, tile_messages::*},
    tilemap_resources::*,
};
use ::sprite_shared::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use common::{AnyDisabling, common_tag_components::TagSet};
use game_common::game_common_components::*;
use ::tilemap_shared::*;

pub use crate::tile::tile_delete_others_helpers::{process_tile_despawns_from_ezero, tile_is_pending_despawn};



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

pub fn despawn_other_tiles_in_same_pos_if_not_excepted_from_added_delete_other_tiles(
    query: Query<
        (Entity, &DimensionRef, &GlobalTilePos, &EntityZeroRef, &DeleteOtherTilesInSamePos, Option<&TagSet>),
        (Added<DeleteOtherTilesInSamePos>, common::AnyDisabling, Without<EntityZero>),
    >,
    registered_positions: Res<ImportantRegisteredPositions>,
    gather_params: TileGatheringParamSet,
    mut despawn_params: TileDeleteOthersParamSet,
    mut writer: MessageWriter<SafeDespawn>,
) {
    for (newtile_ent, &dim, &gpos, &ezero_ref, delete_others, newtile_tags) in &query {
        let bundle = temp_tile_mass_spawn_bundle(ezero_ref, dim, gpos);
        process_tile_despawns_from_added_delete_others(&mut despawn_params, &registered_positions, &gather_params, newtile_ent, &bundle, delete_others, newtile_tags);
    }
    writer.write_batch(despawn_params.msgs.drain(..));
}

pub fn despawn_other_tiles_in_same_pos_if_not_excepted(
    query: Query<
        (Entity, &DimensionRef, &GlobalTilePos, &EntityZeroRef),
        (common::AnyDisabling, Without<EntityZero>),
    >,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
    registered_positions: Res<ImportantRegisteredPositions>,
    gather_params: TileGatheringParamSet,
    mut despawn_params: TileDeleteOthersParamSet,
    mut writer: MessageWriter<SafeDespawn>,
) {
    for ent in changed_pos.read().map(|msg| msg.entity) {
        let Ok((newtile_ent, &dim, &gpos, &ezero_ref)) = query.get(ent) else {
            continue;
        };
        let bundle = temp_tile_mass_spawn_bundle(ezero_ref, dim, gpos);
        process_tile_despawns_from_ezero(&mut despawn_params, &registered_positions, &gather_params, newtile_ent, &bundle);
    }
    writer.write_batch(despawn_params.msgs.drain(..));
}

pub fn safe_despawn_tile_at(
    mut cmd: Commands,
    mut reader: MessageReader<SafeDespawn>,
    mut recheck_writer: MessageWriter<RecheckTileAdjacency>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_children: Query<&Tilemaps>,
    mut tilemap_query: Query<(&mut TileStorage, &HashIdToTexIndex)>,
    dim_query: Query<&DimensionRef, (With<Tile>, common::AnyDisabling)>,
    gpos_query: Query<&GlobalTilePos, (With<Tile>, common::AnyDisabling)>,
    mut rechecks: Local<Vec<RecheckTileAdjacency>>,
) {
    for &SafeDespawn(tile_ent) in reader.read() {
        let (Ok(&dim), Ok(&gpos)) = (dim_query.get(tile_ent), gpos_query.get(tile_ent)) else {
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
        for tmap_ent in tilemaps.iter() {
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
