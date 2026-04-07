use crate::{
    chunking::MacroChunkU16IndexMatrix,
    tile::{tile_components::*, tile_delete_others_systems::*, tile_messages::*},
};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use common::{AnyDisabling, common_tag_components::TagSet};
use game_common::game_common_components::*;
use ::tilemap_shared::{*, DeleteOtherTilesInSamePos};

pub use crate::tile::tile_delete_others_systems::{process_tile_despawns_from_templ, };



#[allow(unused_parens)]
pub fn on_spritetile_despawn(
    trig: On<Despawn, (Tile, Transform, SpriteTile)>,
    query: Query<(&DimensionRef, &GlobalTilePos, &TemplEntiRef), (Without<TilemapId>, Without<TilePos>, Without<Templ>, AnyDisabling)>,
    interaction_zones_query: Query<&InteractionZones, common::AnyDisabling>,
    mut spritetiles_at_gpos: ResMut<SpriteTilesAtGpos>,
) {
    let Ok((&dim_ref, &gpos, templ_ref)) = query.get(trig.entity) else {
        return;
    };
    let interaction_zones = interaction_zones_query.get(templ_ref.0).ok();
    spritetiles_at_gpos.remove_tile(dim_ref, gpos, trig.entity, interaction_zones);
}

pub fn despawn_other_tiles_in_same_pos_if_not_excepted_from_added_delete_other_tiles(
    query: Query<
        (Entity, &DimensionRef, &GlobalTilePos, &TemplEntiRef, &DeleteOtherTilesInSamePos, Option<&TagSet>),
        (Added<DeleteOtherTilesInSamePos>, common::AnyDisabling, Without<Templ>),
    >,
    registered_positions: Res<ImportantRegisteredPositions>,
    gather_params: TileGatheringParamSet,
    mut despawn_params: TileDeleteOthersParamSet,
    mut writer: MessageWriter<SafeDespawn>,
) {
    for (newtile_ent, &dim, &gpos, &templ_ref, delete_others, newtile_tags) in &query {
        process_tile_despawns_from_added_delete_others(&mut despawn_params, &registered_positions, &gather_params, newtile_ent, templ_ref, dim, gpos, delete_others, newtile_tags);
    }
    writer.write_batch(despawn_params.msgs.drain(..));
}

pub fn despawn_other_tiles_in_same_pos_if_not_excepted(
    query: Query<
        (Entity, &DimensionRef, &GlobalTilePos, &TemplEntiRef),
        (common::AnyDisabling, Without<Templ>),
    >,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
    registered_positions: Res<ImportantRegisteredPositions>,
    gather_params: TileGatheringParamSet,
    mut despawn_params: TileDeleteOthersParamSet,
    mut writer: MessageWriter<SafeDespawn>,
) {
    for ent in changed_pos.read().map(|msg| msg.entity) {
        let Ok((newtile_ent, &dim, &gpos, &templ_ref)) = query.get(ent) else {
            continue;
        };
        process_tile_despawns_from_templ(&mut despawn_params, &registered_positions, &gather_params, newtile_ent, templ_ref, dim, gpos);
    }
    writer.write_batch(despawn_params.msgs.drain(..));
}

pub fn safe_despawn_tile_at(
    mut cmd: Commands,
    mut reader: MessageReader<SafeDespawn>,
    mut recheck_writer: MessageWriter<RecheckTileAdjacency>,
    mut card_at_gpos: ResMut<CardinalDirAtGpos>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_children: Query<&Tilemaps>,
    macro_chunk_ref_query: Query<&MacroChunkRef>,
    mut macro_chunk_tile_indices_query: Query<&mut MacroChunkU16IndexMatrix>,
    mut tilemap_query: Query<(&mut TileStorage, &HashIdToTexIndex)>,
    hash_id_query: Query<&common::HashId, common::AnyDisabling>,
    templ_ref_query: Query<&TemplEntiRef, (With<Tile>, common::AnyDisabling)>,
    tile_index_query: Query<&U16TileIndex, common::AnyDisabling>,
    dim_query: Query<&DimensionRef, (With<Tile>, common::AnyDisabling)>,
    gpos_query: Query<&GlobalTilePos, (With<Tile>, common::AnyDisabling)>,
    mut rechecks: Local<Vec<RecheckTileAdjacency>>,
) {
    for &SafeDespawn(tile_ent) in reader.read() {
        let (Ok(&dim), Ok(&gpos)) = (dim_query.get(tile_ent), gpos_query.get(tile_ent)) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        let Ok(templ_ref) = templ_ref_query.get(tile_ent) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        if let Ok(&hash_id) = hash_id_query.get(templ_ref.0) {
            card_at_gpos.0.remove(&(hash_id, gpos));
        }
        let Ok(&tile_index) = tile_index_query.get(templ_ref.0) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        let chunk_pos = gpos.to_chunkpos();
        let Some(&chunk_ent) = loaded_chunks.0.get(&(dim, chunk_pos)) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        let Ok(macro_chunk_ref) = macro_chunk_ref_query.get(chunk_ent) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        let Ok(mut macro_chunk_tile_indices) = macro_chunk_tile_indices_query.get_mut(macro_chunk_ref.0) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        let macro_chunk_pos = chunk_pos.to_macrochunk_pos();
        let _ = macro_chunk_tile_indices.remove_tile_index(macro_chunk_pos.to_chunkpos().to_tilepos(), gpos, tile_index);

        cmd.entity(tile_ent).try_despawn();
        rechecks.push(RecheckTileAdjacency { dim, gpos });
        RecheckTileAdjacency::append_all_adjacent_pos(&mut rechecks, dim, gpos);

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
