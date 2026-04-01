use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use param_sets::BlockingTileParamSet;
use tilemap_shared::{DimensionRef, GlobalTilePos};

pub fn find_nearest_overlap_resolution_gpos(
    blocking_tiles: &mut BlockingTileParamSet,
    reserved_positions: &HashSet<(DimensionRef, GlobalTilePos)>,
    dim_ref: DimensionRef,
    anchor: GlobalTilePos,
    being_ent: Entity,
) -> Option<GlobalTilePos> {
    if !reserved_positions.contains(&(dim_ref, anchor))
        && !blocking_tiles.is_blocked_at_tiles_only(dim_ref, anchor, being_ent)
    {
        return Some(anchor);
    }

    const MAX_SEARCH_RADIUS: i32 = 256;
    for radius in 1..=MAX_SEARCH_RADIUS {
        let min_x = anchor.0.x - radius;
        let max_x = anchor.0.x + radius;
        let min_y = anchor.0.y - radius;
        let max_y = anchor.0.y + radius;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x != min_x && x != max_x && y != min_y && y != max_y {
                    continue;
                }
                let candidate = GlobalTilePos(IVec2::new(x, y));
                if reserved_positions.contains(&(dim_ref, candidate)) {
                    continue;
                }
                if blocking_tiles.is_blocked_at_tiles_only(dim_ref, candidate, being_ent) {
                    continue;
                }
                return Some(candidate);
            }
        }
    }

    None
}
