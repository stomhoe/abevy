use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use common::TILEMAP_SYSTEM;
use game_common::game_common_components::TemplEntiRef;
use game_common::Templ;

use ::tilemap_shared::*;
use crate::tile::tile_components::*;
use crate::tile::tile_resources::*;

#[allow(unused_parens, )]
pub fn track_spawned_tiles_for_ai_nav(
    query: Query<
        (
            &DimensionRef,
            &GlobalTilePos,
            Option<&TileRef>,
            Option<&TemplEntiRef>,
        ),
        (
            Added<Tile>,
            Without<Templ>,
            common::AnyDisabling,
        ),
    >,
    tile_map: Res<TileEntityMap>,
    interaction_zones_query: Query<&InteractionZones, common::AnyDisabling>,
    walk_speed_query: Query<&WalkSpeedMultIfOnTop, common::AnyDisabling>,
    mut ai_nav_tile_blocked_gpos_counts: ResMut<AiNavTileBlockedGposCounts>,
    mut nav_grid_dirty_writer: MessageWriter<AiNavGridDirtyDim>,
    // LEAVE THIS AS A HASHSET, DON'T TOUCH
    mut nav_grid_dirty_msgs: Local<HashSet<AiNavGridDirtyDim>>,
) {
    let iter = query.iter();
    nav_grid_dirty_msgs.reserve(iter.size_hint().1.unwrap_or(iter.size_hint().0));

    let mut tracked_tiles = 0usize;
    for (&dim_ref, &gpos, tile_ref, templ_enti_ref) in &query {
        let Some(templ_ent) = tile_ref
            .and_then(|tile_ref| tile_map.0.get_cloned(tile_ref.0).ok())
            .or_else(|| templ_enti_ref.map(|templ_enti_ref| templ_enti_ref.0))
        else {
            continue;
        };
        let interaction_zones = interaction_zones_query.get(templ_ent).ok();
        let is_low_speed = walk_speed_query
            .get(templ_ent)
            .ok()
            .is_some_and(|walk_speed| walk_speed.is_extremely_low());
        if !ai_nav_tile_blocked_gpos_counts.insert_blocked_positions(dim_ref, gpos, interaction_zones, is_low_speed) {
            continue;
        }
        tracked_tiles += 1;
        nav_grid_dirty_msgs.insert(AiNavGridDirtyDim { dim: dim_ref });
    }

    if tracked_tiles > 0 {
        trace!(target: TILEMAP_SYSTEM, "Tracked {} newly spawned tile blockers for AI nav", tracked_tiles);
    }

    nav_grid_dirty_writer.write_batch(nav_grid_dirty_msgs.drain());
}
