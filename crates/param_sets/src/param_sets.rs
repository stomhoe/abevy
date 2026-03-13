

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)]
use bevy::platform::collections::{HashSet, HashMap};
use common::common_tag_components::TagSet;

use ::being_shared::*;
use game_common::game_common_components::*;
use tilemap::tile::prelude::*;
use ::tilemap_shared::*;


/// system which uses this must be put .in_set(PreChunkDespawnReaders)
#[allow(unused_parens, )]
#[derive(SystemParam)]
pub struct BlockingTileParamSet<'w, 's> {
    tile_gathering_params: TileGatheringParamSet<'w, 's>,
    being_query: Query<'w, 's, (Has<WallPhaser>, )>,
    tiles_2b_despawned_query: Query<'w, 's, (), (With<Dead>, With<DespawnOnDeath>)>,
    tile_instance_query: Query<'w, 's, (&'static EntityZeroRef, &'static GlobalTilePos, Option<&'static TileFlip>, Option<&'static CardinalDirection>), With<Tile>>,
    walk_speed: Query<'w, 's, &'static WalkSpeedMultIfOnTop, >,
    tile_tags: Query<'w, 's, &'static TagSet, >,
    tile_collision_masks: Query<'w, 's, &'static TiledCollisionMask, >,
    beings_at_gpos: Res<'w, BeingsAtGpos>,
}
#[allow(unused_parens, )]
impl<'w, 's> BlockingTileParamSet<'w, 's> {
    pub fn has_tagset_at(&self, to_drain: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos, target_tags: &TagSet) -> bool {
        if target_tags.is_empty() {
            return false;
        }
        to_drain.clear();
        self.tile_gathering_params.gather_tiles_at(to_drain, dim_ref, gpos);
        for tile_entity in to_drain.drain(..) {
            let Ok((ezero_ref, ..)) = self.tile_instance_query.get(tile_entity) else {
                continue;
            };
            let Ok(tile_tags) = self.tile_tags.get(ezero_ref.0) else {
                continue;
            };
            if tile_tags.intersects(target_tags) {
                return true;
            }
        }
        false
    }

    pub fn is_blocked_at(&self, to_drain: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity) -> bool {
        self.is_blocked_at_impl_except_dead_despawning(to_drain, dim_ref, gpos, being, true)
    }

    pub fn is_blocked_at_tiles_only(&self, to_drain: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity) -> bool {
        self.is_blocked_at_impl_except_dead_despawning(to_drain, dim_ref, gpos, being, false)
    }

    fn is_blocked_at_impl_except_dead_despawning(&self, to_drain: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity, include_beings: bool) -> bool {
        if include_beings
            && self
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
            if self.walk_speed.get(ezero_ref.0).cloned().unwrap_or_default().is_extremely_low() {
                let Ok(_) = self.tiles_2b_despawned_query.get(tile_entity) else {
                    return true;
                };

                continue;
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
                let Ok(_) = self.tiles_2b_despawned_query.get(tile_entity) else {
                    return true;
                };
            }
        }
        if all_tiles_failed {
            trace!("No tile found at position {:?} in dimension {:?} for movement blocking check.", gpos, dim_ref);
            return false;
        }
        false
    }
}

#[derive(SystemParam)]
pub struct EntitiesAtGposParamSet<'w> {
    sprite_tiles_at_gpos: Res<'w, SpriteTilesAtGpos>,
    beings_at_gpos: Res<'w, BeingsAtGpos>,
    items_at_gpos: Res<'w, ItemsAtGpos>,
}

impl<'w> EntitiesAtGposParamSet<'w> {
    pub fn gather_entities_at(&self, out: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos) {
        out.extend(self.sprite_tiles_at_gpos.tiles_at_pos(dim_ref, gpos).iter().copied());
        out.extend(self.beings_at_gpos.beings_at_pos(dim_ref, gpos).iter().copied());
        out.extend(self.items_at_gpos.items_at_pos(dim_ref, gpos).iter().copied());
    }
}
