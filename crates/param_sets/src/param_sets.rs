

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
use tilemap::chunking::chunking_components::{Chunk, TerrGenState};


/// system which uses this must be put .in_set(PreChunkDespawnReaders)
#[allow(unused_parens, )]
#[derive(SystemParam)]
pub struct BlockingTileParamSet<'w, 's> {
    tile_gathering_params: TileGatheringParamSet<'w, 's>,
    wallphaser_query: Query<'w, 's, (), With<WallPhaser>>,
    will_despawn_query: Query<'w, 's, (), (With<Dead>, With<DespawnOnDeath>)>,
    tile_instance_query: Query<'w, 's, (&'static EntityZeroRef, &'static GlobalTilePos, Option<&'static TileFlip>, Option<&'static CardinalDirection>), (With<Tile>, Without<Being>)>,
    walk_speed: Query<'w, 's, &'static WalkSpeedMultIfOnTop, >,
    tile_tags: Query<'w, 's, &'static TagSet, (With<Tile>, Without<Being>)>,
    tile_collision_masks: Query<'w, 's, &'static TiledCollisionMask, >,
    terrgen_states: Query<'w, 's, &'static TerrGenState, With<Chunk>>,
    beings_at_gpos: Res<'w, BeingsAtGpos>,
}
#[allow(unused_parens, )]
impl<'w, 's> BlockingTileParamSet<'w, 's> {
    pub fn gather_tiles_at_to_drain(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.tile_gathering_params.gather_tiles_at_to_drain(dim_ref, gpos)
    }

    pub fn find_nearest_unblocked_gpos_in_chunk(
        &mut self,
        dim_ref: DimensionRef,
        anchor: GlobalTilePos,
        being: Entity,
        whitelisted_tags: &WhitelistedSpawnTileTags,
        blacklisted_tags: &BlacklistedSpawnTileTags,
    ) -> Option<GlobalTilePos> {
        let chunk_pos = anchor.to_chunkpos();
        let min_tile = chunk_pos.to_tilepos().0;
        let clamped_anchor = chunk_pos.clamp_gpos_to_chunk(anchor);
        let local_anchor = clamped_anchor.0 - min_tile;
        let max_radius = (ChunkPos::CHUNK_SIZE.x.max(ChunkPos::CHUNK_SIZE.y) as i32).saturating_sub(1);

        for radius in 0..=max_radius {
            let min_local_x = (local_anchor.x - radius).max(0);
            let max_local_x = (local_anchor.x + radius).min(ChunkPos::CHUNK_SIZE.x as i32 - 1);
            let min_local_y = (local_anchor.y - radius).max(0);
            let max_local_y = (local_anchor.y + radius).min(ChunkPos::CHUNK_SIZE.y as i32 - 1);

            for local_y in min_local_y..=max_local_y {
                for local_x in min_local_x..=max_local_x {
                    if radius != 0
                        && local_x != min_local_x
                        && local_x != max_local_x
                        && local_y != min_local_y
                        && local_y != max_local_y
                    {
                        continue;
                    }
                    let candidate = GlobalTilePos(min_tile + IVec2::new(local_x, local_y));
                    if !self.allows_spawn_at(
                        dim_ref,
                        candidate,
                        being,
                        whitelisted_tags,
                        blacklisted_tags,
                    ) {
                        continue;
                    }
                    return Some(candidate);
                }
            }
        }

        None
    }

    pub fn allows_spawn_at(
        &mut self,
        dim_ref: DimensionRef,
        gpos: GlobalTilePos,
        being: Entity,
        whitelisted_tags: &WhitelistedSpawnTileTags,
        blacklisted_tags: &BlacklistedSpawnTileTags,
    ) -> bool {
        if self
            .beings_at_gpos
            .beings_at_pos(dim_ref, gpos)
            .iter()
            .any(|&ent| ent != being)
        {
            return false;
        }

        let can_phase = self.wallphaser_query.get(being).is_ok();
        let tiles_at_pos = self.tile_gathering_params.gather_tiles_at_to_drain(dim_ref, gpos).to_vec();

        let mut all_tiles_failed = true;
        let mut has_whitelist_match = whitelisted_tags.0.is_empty();
        let mut has_blacklist_match = false;

        for tile_entity in tiles_at_pos {
            let Ok((ezero_ref, tile_origin, tile_flip, direction)) = self.tile_instance_query.get(tile_entity) else {
                continue;
            };
            all_tiles_failed = false;

            if let Ok(tile_tags) = self.tile_tags.get(ezero_ref.0) {
                if !whitelisted_tags.0.is_empty() && tile_tags.intersects(&whitelisted_tags.0) {
                    has_whitelist_match = true;
                }
                if !blacklisted_tags.0.is_empty() && tile_tags.intersects(&blacklisted_tags.0) {
                    has_blacklist_match = true;
                }
            }

            if can_phase {
                continue;
            }

            if self.walk_speed.get(ezero_ref.0).cloned().unwrap_or_default().is_extremely_low() {
                if self.will_despawn_query.get(tile_entity).is_err() {
                    return false;
                }
                continue;
            }

            let blocks_here = self
                .tile_collision_masks
                .get(ezero_ref.0)
                .map(|mask| {
                    mask.is_solid_at_world_pos_with_flip(
                        *tile_origin,
                        gpos,
                        tile_flip.copied().unwrap_or_default(),
                        direction.copied().unwrap_or_default(),
                    )
                })
                .unwrap_or(false);
            if blocks_here && self.will_despawn_query.get(tile_entity).is_err() {
                return false;
            }
        }

        if all_tiles_failed {
            trace!("No tile found at position {:?} in dimension {:?} for spawn validation.", gpos, dim_ref);
            return whitelisted_tags.0.is_empty();
        }
        if !whitelisted_tags.0.is_empty() {
            return has_whitelist_match;
        }
        !has_blacklist_match
    }

    pub fn has_tagset_at(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, target_tags: &TagSet) -> bool {
        if target_tags.is_empty() {
            return false;
        }
        let tiles_at_pos = self.tile_gathering_params.gather_tiles_at_to_drain(dim_ref, gpos).to_vec();
        for tile_entity in tiles_at_pos {
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

    pub fn is_blocked_at(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity) -> bool {
        self.is_blocked_at_impl_except_dead_despawning(dim_ref, gpos, being, true)
    }

    pub fn is_blocked_at_tiles_only(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity) -> bool {
        self.is_blocked_at_impl_except_dead_despawning(dim_ref, gpos, being, false)
    }

    fn is_blocked_at_impl_except_dead_despawning(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity, include_beings: bool) -> bool {
        if include_beings
            && self
                .beings_at_gpos
                .beings_at_pos(dim_ref, gpos)
                .iter()
                .any(|&ent| ent != being)
        {
            return true;
        }

        let can_phase = self.wallphaser_query.get(being).is_ok();
        if can_phase {
            return false;
        }
        
        let tiles_at_pos = self.tile_gathering_params.gather_tiles_at_to_drain(dim_ref, gpos).to_vec();

        let mut all_tiles_failed = true;
        for tile_entity in tiles_at_pos {
            let Ok((ezero_ref, tile_origin, tile_flip, direction)) = self.tile_instance_query.get(tile_entity) else {
                continue;
            };
            all_tiles_failed = false;
            if self.walk_speed.get(ezero_ref.0).cloned().unwrap_or_default().is_extremely_low() {
                let Ok(_) = self.will_despawn_query.get(tile_entity) else {
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
                let Ok(_) = self.will_despawn_query.get(tile_entity) else {
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

    pub fn find_closest_spawn_suitable_gpos_across_loaded_chunks(
        &mut self,
        loaded_chunks: &LoadedChunks,
        dim_ref: DimensionRef,
        target_gpos: GlobalTilePos,
        being: Entity,
        whitelisted_tags: &WhitelistedSpawnTileTags,
        blacklisted_tags: &BlacklistedSpawnTileTags,
        max_chunk_manhattan: i32,
    ) -> Option<GlobalTilePos> {
        let home_chunk = target_gpos.to_chunkpos();

        // collect chunks in this dimension
        let mut nearby_chunks: Vec<(ChunkPos, Entity)> = Vec::new();
        for (&(dref, chunk_pos), &chunk_ent) in loaded_chunks.0.iter() {
            if dref != dim_ref {
                continue;
            }
            let manhattan = (chunk_pos.0.x - home_chunk.0.x).abs() + (chunk_pos.0.y - home_chunk.0.y).abs();
            if manhattan as i32 <= max_chunk_manhattan {
                nearby_chunks.push((chunk_pos, chunk_ent));
            }
        }

        // sort by Manhattan distance ascending
        nearby_chunks.sort_by_key(|(cp, _)| (cp.0.x - home_chunk.0.x).abs() + (cp.0.y - home_chunk.0.y).abs());

        let mut index = 0usize;
        while index < nearby_chunks.len() {
            let (chunk_pos, chunk_ent) = nearby_chunks[index];
            let distance = (chunk_pos.0.x - home_chunk.0.x).abs() + (chunk_pos.0.y - home_chunk.0.y).abs();
            // if we've moved to a new distance level and it's already > max, stop
            if distance as i32 > max_chunk_manhattan {
                break;
            }

            let anchor = chunk_pos.clamp_gpos_to_chunk(target_gpos);
            // check terrgen readiness for this chunk
            if let Ok(terr) = self.terrgen_states.get(chunk_ent) {
                if !terr.is_ready() {
                    index += 1;
                    continue;
                }
            } else {
                index += 1;
                continue;
            }

            if let Some(found_gpos) = self.find_nearest_unblocked_gpos_in_chunk(
                dim_ref,
                anchor,
                being,
                whitelisted_tags,
                blacklisted_tags,
            ) {
                return Some(found_gpos);
            }

            index += 1;
        }

        None
    }
}

#[derive(SystemParam)]
pub struct EntitiesAtGposParamSet<'w> {
    beings_at_gpos: Res<'w, BeingsAtGpos>,
    items_at_gpos: Res<'w, ItemsAtGpos>,
}

impl<'w> EntitiesAtGposParamSet<'w> {
    pub fn gather_entities_at(&self, out: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos) {
        out.extend(self.beings_at_gpos.beings_at_pos(dim_ref, gpos).iter().copied());
        out.extend(self.items_at_gpos.items_at_pos(dim_ref, gpos).iter().copied());
    }
}
