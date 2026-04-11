use crate::{
    tile::{tile_components::*, tile_messages::*, tile_resources::*},
};
use bevy::{ecs::system::SystemParam, platform::collections::{HashMap, HashSet}};
use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapId, tiles::TileFlip};
use common::common_components::HashId;
use ::tilemap_shared::*;

#[allow(unused_parens)]
pub fn reckeck_adjacency_for(
    mut reader: MessageReader<GlobalTilePosChanged>,
    mut writer: MessageWriter<RecheckTileAdjacency>,
    tiles_query: Query<(&DimensionRef, &GlobalTilePos), (With<Tile>, common::AnyDisabling)>,
    mut msgs: Local<Vec<RecheckTileAdjacency>>,
) {
    for read in reader.read() {
        if let Some(old) = read.old {
            RecheckTileAdjacency::append_all_adjacent_pos(&mut msgs, old.dim, old.gpos, );
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
#[derive(SystemParam)]
pub struct TileAdjacencyRetextureLocals<'s> {
    pub adj_masks_by_hid: Local<'s, HashMap<HashId, AdjMask>>,
    pub tiles_to_recheck: Local<'s, Vec<Entity>>,
    pub north_adj_tiles_templs: Local<'s, Vec<Entity>>,
    pub south_adj_tiles_templs: Local<'s, Vec<Entity>>,
    pub west_adj_tiles_templs: Local<'s, Vec<Entity>>,
    pub east_adj_tiles_templs: Local<'s, Vec<Entity>>,
    pub northeast_adj_tiles_templs: Local<'s, Vec<Entity>>,
    pub northwest_adj_tiles_templs: Local<'s, Vec<Entity>>,
    pub southeast_adj_tiles_templs: Local<'s, Vec<Entity>>,
    pub southwest_adj_tiles_templs: Local<'s, Vec<Entity>>,
    pub unique_rechecks: Local<'s, HashSet<(DimensionRef, GlobalTilePos)>>,
}

#[allow(unused_parens)]
pub fn tile_adjacency_dependent_retexturing_system(
    mut reader: MessageReader<RecheckTileAdjacency>,
    mut tile_query: Query<(&TileRef, &DimensionRef, &GlobalTilePos, Option<&mut sprite_animation_shared::AnimExtraState>, Option<(&mut TileTextureIndex, &mut TileFlip, &TilemapId)>, ), ()>,
    templ_query: Query<&AdjRetexConfig, ()>,
    tile_map: Res<TileEntityMap>,
    params: TileGatheringParamSet,
    mut locals: TileAdjacencyRetextureLocals,
) {
    locals.unique_rechecks.clear();
    for msg in reader.read() {
        let key = (msg.dim, msg.gpos);
        if !locals.unique_rechecks.insert(key) {
            continue;
        }
        params.gather_tiles_extend(&mut *locals.tiles_to_recheck, msg.dim, msg.gpos);
        for tile_ent in locals.tiles_to_recheck.drain(..) {
            let Ok((templ_ref, &dim, &gpos, ..)) = tile_query.get(tile_ent) else {
                continue;
            };
            let Ok(templ_ent) = tile_map.0.get_cloned(templ_ref.0) else {
                continue;
            };
            let Ok(adj_retex_config) = templ_query.get(templ_ent) else {
                continue;
            };
            locals.adj_masks_by_hid.clear();
            params.gather_tiles_extend(&mut *locals.north_adj_tiles_templs, dim, gpos.adjacent_north());
            params.gather_tiles_extend(&mut *locals.south_adj_tiles_templs, dim, gpos.adjacent_south());
            params.gather_tiles_extend(&mut *locals.west_adj_tiles_templs, dim, gpos.adjacent_west());
            params.gather_tiles_extend(&mut *locals.east_adj_tiles_templs, dim, gpos.adjacent_east());
            params.gather_tiles_extend(&mut *locals.northeast_adj_tiles_templs, dim, gpos.adjacent_northeast());
            params.gather_tiles_extend(&mut *locals.northwest_adj_tiles_templs, dim, gpos.adjacent_northwest());
            params.gather_tiles_extend(&mut *locals.southeast_adj_tiles_templs, dim, gpos.adjacent_southeast());
            params.gather_tiles_extend(&mut *locals.southwest_adj_tiles_templs, dim, gpos.adjacent_southwest());
            let mut process_adjacent_tiles = |adj_mask: AdjMask, adj_tiles: &mut Vec<Entity>| {
                for adj_tile_ent in adj_tiles.drain(..) {
                    let Ok((templ_ref, ..)) = tile_query.get(adj_tile_ent) else {
                        continue;
                    };
                    locals.adj_masks_by_hid.entry(templ_ref.0).or_default().insert(adj_mask);
                }
            };
            process_adjacent_tiles(DiagonalCardinalDirection::North.adj_mask_bit(), &mut locals.north_adj_tiles_templs);
            process_adjacent_tiles(DiagonalCardinalDirection::South.adj_mask_bit(), &mut locals.south_adj_tiles_templs);
            process_adjacent_tiles(DiagonalCardinalDirection::West.adj_mask_bit(), &mut locals.west_adj_tiles_templs);
            process_adjacent_tiles(DiagonalCardinalDirection::East.adj_mask_bit(), &mut locals.east_adj_tiles_templs);
            process_adjacent_tiles(DiagonalCardinalDirection::NorthEast.adj_mask_bit(), &mut locals.northeast_adj_tiles_templs);
            process_adjacent_tiles(DiagonalCardinalDirection::NorthWest.adj_mask_bit(), &mut locals.northwest_adj_tiles_templs);
            process_adjacent_tiles(DiagonalCardinalDirection::SouthEast.adj_mask_bit(), &mut locals.southeast_adj_tiles_templs);
            process_adjacent_tiles(DiagonalCardinalDirection::SouthWest.adj_mask_bit(), &mut locals.southwest_adj_tiles_templs);

            let Some((hid_to_use, new_flip)) = adj_retex_config.get_tex_in_curr_adjacency_state(&locals.adj_masks_by_hid) else {
                continue;
            };
            let Ok((templ_ref, .., anim_state, tmap_tile_data)) = tile_query.get_mut(tile_ent) else {
                continue;
            };
            if let Some((mut tex_idx, mut flip, tmap_ent)) = tmap_tile_data {
                let Ok((_, hash2tex, _)) = params.tilemap_query.get(tmap_ent.0) else {
                    continue;
                };
                if let Ok(new_tex_idx) = hash2tex.get(templ_ref.0, hid_to_use) {
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
