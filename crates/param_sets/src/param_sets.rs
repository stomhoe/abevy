

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::platform::collections::{HashSet, HashMap};

use ::being_shared::*;
use ::tilemap_shared::*;


#[allow(unused_parens, )]
#[derive(SystemParam)]
pub struct BlockingTileParamSet<'w, 's> {
    tile_gathering_params: TileGatheringParamSet<'w, 's>,
    being_query: Query<'w, 's, (Has<WallPhaser>, )>,
    blocking_tiles: Query<'w, 's, &'static WalkSpeedMultIfOnTop, >,
}
#[allow(unused_parens, )]
impl<'w, 's> BlockingTileParamSet<'w, 's> {

    pub fn is_blocked_at(&self, to_drain: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity) -> bool {
        let can_phase = if let Ok((can_phase, ..)) = self.being_query.get(being) {
            can_phase
        } else {
            false
        };
        if can_phase {
            return false;
        }
        to_drain.clear();
        self.tile_gathering_params.gather_tiles_at(to_drain, dim_ref, gpos);

        let mut all_tiles_failed = true;
        for tile_entity in to_drain.drain(..) {
            if let Ok(walk_speed) = self.blocking_tiles.get(tile_entity) {
                all_tiles_failed = false;
                if walk_speed.0 == 0.0 {
                    return true;
                }
            }
        }
        if all_tiles_failed {
            trace!("No tile found at position {:?} in dimension {:?} for movement blocking check.", gpos, dim_ref);
            return false;
        }
        false
    }
}
