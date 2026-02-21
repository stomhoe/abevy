

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)]
use bevy::platform::collections::{HashSet, HashMap};

use ::being_shared::*;
use game_common::game_common_components::*;
use ::tilemap_shared::*;


#[allow(unused_parens, )]
#[derive(SystemParam)]
/// system which uses this must be put .in_set(PreChunkDespawnReaders)
pub struct BlockingTileParamSet<'w, 's> {
    tile_gathering_params: TileGatheringParamSet<'w, 's>,
    being_query: Query<'w, 's, (Has<WallPhaser>, )>,
    tile_instance_query: Query<'w, 's, (&'static EntityZeroRef, &'static GlobalTilePos, Option<&'static TileFlip>, Option<&'static CardinalDirection>), >,
    walk_speed: Query<'w, 's, &'static WalkSpeedMultIfOnTop, >,
    tile_collision_masks: Query<'w, 's, &'static TileCollisionMask, >,
    beings_at_gpos: Res<'w, BeingsAtGpos>,
}
#[allow(unused_parens, )]
impl<'w, 's> BlockingTileParamSet<'w, 's> {

    pub fn is_blocked_at(&self, to_drain: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity) -> bool {
        if self
            .beings_at_gpos
            .beings_at_pos(dim_ref, gpos)
            .iter()
            .any(|&ent| ent != being)
        {
            return true;
        }

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
            let Ok((ezero_ref, tile_origin, tile_flip, direction)) = self.tile_instance_query.get(tile_entity) else {
                continue;
            };
            all_tiles_failed = false;
            if self.walk_speed.get(ezero_ref.0).cloned().unwrap_or_default().is_extremely_low(){
                return true;
            }

            let blocks_here = if let Ok(mask) = self.tile_collision_masks.get(ezero_ref.0) {
                mask.is_solid_at_world_pos_with_flip(
                    *tile_origin,
                    gpos,
                    tile_flip.copied().unwrap_or_default(),
                    direction.copied().unwrap_or_default(),
                )
            } else {
                false
            };
            if blocks_here {
                return true;
            }
        }
        if all_tiles_failed {
            trace!("No tile found at position {:?} in dimension {:?} for movement blocking check.", gpos, dim_ref);
            return false;
        }
        false
    }
}
