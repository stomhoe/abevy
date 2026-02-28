use crate::{
    tile::{tile_components::*, tile_messages::*},
    tilemap_resources::*,
};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapId, tiles::TileFlip};
use common::common_components::HashId;
use game_common::game_common_components::*;
use ::tilemap_shared::*;

#[allow(unused_parens)]
pub fn reckeck_adjacency_for(
    mut reader: MessageReader<GlobalTilePosChanged>,
    mut writer: MessageWriter<RecheckTileAdjacency>,
    tiles_query: Query<(&DimensionRef, &GlobalTilePos), (With<Tile>, common::AnyDisabling)>,
    mut msgs: Local<Vec<RecheckTileAdjacency>>,
) {
    for read in reader.read() {
        if read.old_gpos != PrevGlobalTilePos::PLACEHOLDER_I32_MAX.0 {
            RecheckTileAdjacency::append_all_adjacent_pos(&mut msgs, read.old_dim, read.old_gpos, );
        }
        if let Ok((&new_dim, &new_gpos)) = tiles_query.get(read.entity) {
            msgs.push(RecheckTileAdjacency {
                dim: new_dim,
                gpos: new_gpos,
            });
            RecheckTileAdjacency::append_all_adjacent_pos(&mut msgs, new_dim, new_gpos, );
        }
    }
    writer.write_batch(msgs.drain(..));
}

#[allow(unused_parens)]
pub fn tile_adjacency_dependent_retexturing_system(
    mut reader: MessageReader<RecheckTileAdjacency>,
    mut tile_query: Query<(&EntityZeroRef, &DimensionRef, &GlobalTilePos, Option<&mut sprite_animation_shared::AnimExtraState>, Option<(&mut TileTextureIndex, &mut TileFlip, &TilemapId)>, ), ()>,
    ezero_query: Query<(&HashId, Option<&AdjRetexConfig>), ()>,
    hash2tex_query: Query<(&HashIdToTexIndex), ()>,
    params: TileGatheringParamSet,
    mut adj_masks_by_hid: Local<HashMap<HashId, AdjMask>>,
    mut north_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut south_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut west_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut east_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut northeast_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut northwest_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut southeast_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut southwest_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut unique_rechecks: Local<HashSet<(DimensionRef, GlobalTilePos)>>,
) {
    unique_rechecks.clear();
    for msg in reader.read() {
        let key = (msg.dim, msg.gpos);
        if !unique_rechecks.insert(key) {
            continue;
        }
        north_adj_tiles_ezeros.clear();
        params.gather_tiles_at(&mut *north_adj_tiles_ezeros, msg.dim, msg.gpos);
        let mut tiles_to_recheck: Vec<Entity> = north_adj_tiles_ezeros.drain(..).collect();

        for tile_ent in tiles_to_recheck.drain(..) {
            let Ok((ezero_ref, &dim, &gpos, ..)) = tile_query.get(tile_ent) else {
                continue;
            };
            let Ok((_, Some(adj_retex_config))) = ezero_query.get(ezero_ref.0) else {
                continue;
            };
            north_adj_tiles_ezeros.clear();
            south_adj_tiles_ezeros.clear();
            west_adj_tiles_ezeros.clear();
            east_adj_tiles_ezeros.clear();
            northeast_adj_tiles_ezeros.clear();
            northwest_adj_tiles_ezeros.clear();
            southeast_adj_tiles_ezeros.clear();
            southwest_adj_tiles_ezeros.clear();
            adj_masks_by_hid.clear();

            params.gather_tiles_at(&mut *north_adj_tiles_ezeros, dim, gpos.adjacent_north());
            params.gather_tiles_at(&mut *south_adj_tiles_ezeros, dim, gpos.adjacent_south());
            params.gather_tiles_at(&mut *west_adj_tiles_ezeros, dim, gpos.adjacent_west());
            params.gather_tiles_at(&mut *east_adj_tiles_ezeros, dim, gpos.adjacent_east());
            params.gather_tiles_at(&mut *northeast_adj_tiles_ezeros, dim, gpos.adjacent_northeast());
            params.gather_tiles_at(&mut *northwest_adj_tiles_ezeros, dim, gpos.adjacent_northwest());
            params.gather_tiles_at(&mut *southeast_adj_tiles_ezeros, dim, gpos.adjacent_southeast());
            params.gather_tiles_at(&mut *southwest_adj_tiles_ezeros, dim, gpos.adjacent_southwest());

            let mut process_adjacent_tiles = |adj_mask: AdjMask, adj_tiles: &mut Vec<Entity>| {
                for adj_tile_ent in adj_tiles.drain(..) {
                    let Ok((ezero_ref, ..)) = tile_query.get(adj_tile_ent) else {
                        continue;
                    };
                    let Ok((&hid, ..)) = ezero_query.get(ezero_ref.0) else {
                        continue;
                    };
                    adj_masks_by_hid.entry(hid).or_default().insert(adj_mask);
                }
            };
            process_adjacent_tiles(DiagonalCardinalDirection::North.adj_mask_bit(), &mut north_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::South.adj_mask_bit(), &mut south_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::West.adj_mask_bit(), &mut west_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::East.adj_mask_bit(), &mut east_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::NorthEast.adj_mask_bit(), &mut northeast_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::NorthWest.adj_mask_bit(), &mut northwest_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::SouthEast.adj_mask_bit(), &mut southeast_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::SouthWest.adj_mask_bit(), &mut southwest_adj_tiles_ezeros);

            let Some((hid_to_use, new_flip)) = adj_retex_config.get_tex_in_curr_adjacency_state(&adj_masks_by_hid) else {
                continue;
            };
            let Ok((ezero_ref, .., anim_state, tmap_tile_data)) = tile_query.get_mut(tile_ent) else {
                continue;
            };
            let Ok((&tile_hid, ..)) = ezero_query.get(ezero_ref.0) else {
                continue;
            };
            if let Some((mut tex_idx, mut flip, tmap_ent)) = tmap_tile_data {
                let Ok(hash2tex) = hash2tex_query.get(tmap_ent.0) else {
                    continue;
                };
                if let Ok(new_tex_idx) = hash2tex.get(tile_hid, hid_to_use) {
                    *tex_idx = new_tex_idx;
                }
                if let Some(new_flip) = new_flip {
                    *flip = new_flip;
                }
            } else if let Some(mut anim_state) = anim_state {
                anim_state.0 = hid_to_use;
            }
        }
    }
}
