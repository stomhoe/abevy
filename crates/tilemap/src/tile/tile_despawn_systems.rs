use crate::{
    chunking::MacroChunkU16IndexMatrix,
    tile::{tile_components::*, tile_delete_others_systems::*, tile_messages::*, tile_resources::*},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use common::{AnyDisabling, common_tag_components::TagSet};
use game_common::game_common_components::*;
use ::tilemap_shared::{*, DeleteOtherTilesInSamePos};

pub use crate::tile::tile_delete_others_systems::{process_tile_despawns_from_templ, };

#[derive(SystemParam)]
#[allow(unused_parens, )]
pub struct SafeDespawnTileQueries<'w, 's> {
    pub loaded_chunks: Res<'w, LoadedChunks>,
    pub chunk_children: Query<'w, 's, &'static Tilemaps>,
    pub macro_chunk_ref_query: Query<'w, 's, &'static MacroChunkRef>,
    pub macro_chunk_tile_indices_query: Query<'w, 's, &'static mut MacroChunkU16IndexMatrix>,
    pub tilemap_query: Query<'w, 's, (&'static mut TileStorage, &'static HashIdToTexIndex)>,
    pub tile_map: Res<'w, TileEntityMap>,
    pub templ_ref_query: Query<'w, 's, &'static TileRef, (With<Tile>, common::AnyDisabling)>,
    pub tile_index_query: Query<'w, 's, &'static U16TileIndex, common::AnyDisabling>,
    pub dim_query: Query<'w, 's, &'static DimensionRef, (With<Tile>, common::AnyDisabling)>,
    pub gpos_query: Query<'w, 's, &'static GlobalTilePos, (With<Tile>, common::AnyDisabling)>,
    pub walk_speed_query: Query<'w, 's, &'static WalkSpeedMultIfOnTop, common::AnyDisabling>,
    pub interaction_zones_query: Query<'w, 's, &'static InteractionZones, common::AnyDisabling>,
    pub sprite_tile_query: Query<'w, 's, (), With<SpriteTile>>,
    pub ai_nav_blocked_gpos_counts: ResMut<'w, AiNavBlockedGposCounts>,
}

#[derive(SystemParam)]
#[allow(unused_parens, )]
pub struct SafeDespawnTileLocals<'s> {
    pub rechecks: Local<'s, Vec<RecheckTileAdjacency>>,
}

pub fn on_spritetile_despawn(
    trig: On<Despawn, (Tile, Transform, SpriteTile)>,
    query: Query<(&DimensionRef, &GlobalTilePos, &TileRef), (Without<TilemapId>, Without<TilePos>, Without<Templ>, AnyDisabling)>,
    tile_map: Res<TileEntityMap>,
    interaction_zones_query: Query<&InteractionZones, common::AnyDisabling>,
    walk_speed_query: Query<&WalkSpeedMultIfOnTop, common::AnyDisabling>,
    mut ai_nav_blocked_gpos_counts: ResMut<AiNavBlockedGposCounts>,
    mut spritetiles_at_gpos: ResMut<SpriteTilesAtGpos>,
) {
    let Ok((&dim_ref, &gpos, templ_ref)) = query.get(trig.entity) else {
        return;
    };
    let interaction_zones = tile_map
        .0
        .get_cloned(templ_ref.0)
        .ok()
        .and_then(|templ_ent| interaction_zones_query.get(templ_ent).ok());
    let is_low_speed = tile_map
        .0
        .get_cloned(templ_ref.0)
        .ok()
        .and_then(|templ_ent| walk_speed_query.get(templ_ent).ok())
        .is_some_and(|walk_speed| walk_speed.is_extremely_low());
    ai_nav_blocked_gpos_counts.remove_blocked_positions(dim_ref, gpos, interaction_zones, is_low_speed);
    spritetiles_at_gpos.remove_tile(dim_ref, gpos, trig.entity, interaction_zones);
}

pub fn despawn_other_tiles_in_same_pos_if_not_excepted_from_added_delete_other_tiles(
    query: Query<
        (Entity, &DimensionRef, &GlobalTilePos, &TileRef, &DeleteOtherTilesInSamePos, Option<&TagSet>),
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
        (Entity, &DimensionRef, &GlobalTilePos, &TileRef),
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
    mut queries: SafeDespawnTileQueries,
    mut locals: SafeDespawnTileLocals,
) {
    let SafeDespawnTileQueries {
        loaded_chunks,
        chunk_children,
        macro_chunk_ref_query,
        macro_chunk_tile_indices_query,
        tilemap_query,
        tile_map,
        templ_ref_query,
        tile_index_query,
        dim_query,
        gpos_query,
        walk_speed_query,
        interaction_zones_query,
        sprite_tile_query,
        ai_nav_blocked_gpos_counts,
    } = &mut queries;
    let SafeDespawnTileLocals { rechecks } = &mut locals;

    for &SafeDespawn(tile_ent) in reader.read() {
        let (Ok(&dim), Ok(&gpos)) = (dim_query.get(tile_ent), gpos_query.get(tile_ent)) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        let Ok(templ_ref) = templ_ref_query.get(tile_ent) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        let interaction_zones = tile_map
            .0
            .get_cloned(templ_ref.0)
            .ok()
            .and_then(|templ_ent| interaction_zones_query.get(templ_ent).ok());
        let is_low_speed = tile_map
            .0
            .get_cloned(templ_ref.0)
            .ok()
            .and_then(|templ_ent| walk_speed_query.get(templ_ent).ok())
            .is_some_and(|walk_speed| walk_speed.is_extremely_low());
        if sprite_tile_query.get(tile_ent).is_err() {
            ai_nav_blocked_gpos_counts.remove_blocked_positions(dim, gpos, interaction_zones, is_low_speed);
        }
        card_at_gpos.0.remove(&(templ_ref.0, gpos));
        let Ok(templ_ent) = tile_map.0.get_cloned(templ_ref.0) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };
        let Ok(&tile_index) = tile_index_query.get(templ_ent) else {
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
        RecheckTileAdjacency::append_all_adjacent_pos(rechecks, dim, gpos);

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
