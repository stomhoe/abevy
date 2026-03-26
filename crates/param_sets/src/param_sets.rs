

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use common::common_components::HashId;
use common::common_tag_components::TagSet;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use ::being_shared::*;
use game_common::game_common_components::*;
use tilemap::chunking::*;
use tilemap::tile::*;

#[derive(Clone, Copy, )]
struct CollisionTileSample {
    templ_ent: Entity,
    tile_origin: GlobalTilePos,
    direction: CardinalDirection,
    dead_despawning: bool,
}

fn resolve_tile_direction(
    hash_id_query: &Query<&HashId, common::AnyDisabling>,
    card_at_gpos: &Res<CardinalDirAtGpos>,
    templ_ent: Entity,
    gpos: GlobalTilePos,
    fallback: CardinalDirection,
) -> CardinalDirection {
    let Ok(hash_id) = hash_id_query.get(templ_ent) else {
        return fallback;
    };
    card_at_gpos
        .0
        .get(&(*hash_id, gpos))
        .copied()
        .unwrap_or(fallback)
}


/// system which uses this must be put .in_set(PreChunkDespawnReaders)
#[allow(unused_parens, )]
#[derive(SystemParam)]
pub struct BlockingTileParamSet<'w, 's> {
    tile_gathering_params: TileGatheringParamSet<'w, 's>,
    wallphaser_query: Query<'w, 's, (), With<WallPhaser>>,
    will_despawn_query: Query<'w, 's, (), (With<Dead>, With<DespawnOnDeath>, common::AnyDisabling)>,
    templ_ref_query: Query<'w, 's, &'static TemplEntiRef, ()>,
    pub gpos_query: Query<'w, 's, &'static mut GlobalTilePos, common::AnyDisabling>,
    walk_speed: Query<'w, 's, &'static WalkSpeedMultIfOnTop, common::AnyDisabling>,
    race_ref_query: Query<'w, 's, &'static RaceRef, common::AnyDisabling>,
    bit_ref_query: Query<'w, 's, &'static BitRef, common::AnyDisabling>,
    tags: Query<'w, 's, &'static TagSet, >,
    interaction_zones: Query<'w, 's, &'static InteractionZones, common::AnyDisabling>,
    macro_chunk_tile_indices: Query<'w, 's, &'static MacroChunkTileIndices, common::AnyDisabling>,
    tile_indexing_query: Query<'w, 's, &'static TileIndexing, >,
    hash_id_query: Query<'w, 's, &'static HashId, common::AnyDisabling>,
    loaded_macro_chunks: Res<'w, LoadedMacroChunks>,
    tile_map: Res<'w, TileEntityMap>,
    card_at_gpos: Res<'w, CardinalDirAtGpos>,
    beings_at_gpos: Res<'w, BeingsAtGpos>,
    occupied_gposes: Local<'s, Vec<GlobalTilePos>>,
    collision_tile_samples: Local<'s, Vec<CollisionTileSample>>,
}
#[allow(unused_parens, )]
impl<'w, 's> BlockingTileParamSet<'w, 's> {
    pub fn get_being_bit_ref(&self, being: Entity) -> Option<&BitRef> {
        self.bit_ref_query.get(being).ok()
    }

    pub fn get_being_race_ref(&self, being: Entity) -> Option<&RaceRef> {
        self.race_ref_query.get(being).ok()
    }

    fn resolve_collision_context_for_entity<'a>(&'a self, entity: Entity) -> (Cow<'a, InteractionZone>, CardinalDirection) {
        let collision_zone = self
            .interaction_zones
            .get(entity)
            .ok()
            .and_then(|zones| zones.get_collision_mask())
            .map(Cow::Borrowed)
            .or_else(|| {
                self.bit_ref_query.get(entity).ok()
                    .and_then(|bit_ref| self.interaction_zones.get(bit_ref.0).ok())
                    .and_then(|zones| zones.get_collision_mask())
                    .map(Cow::Borrowed)
            })
            .or_else(|| {
                self.race_ref_query.get(entity).ok()
                    .and_then(|race_ref| self.interaction_zones.get(race_ref.0).ok())
                    .and_then(|zones| zones.get_collision_mask())
                    .map(Cow::Borrowed)
            })
            .unwrap_or_else(|| Cow::Owned(InteractionZone::collision_default_zone()));
        let facing_dir = self.tile_gathering_params.cardinal_direction_query.get(entity).cloned().unwrap_or_default();
        (collision_zone, facing_dir)
    }

    pub fn find_nearest_unblocked_gpos_in_chunk(
        &mut self,
        dim_ref: DimensionRef,
        anchor: GlobalTilePos,
        being: Entity,
        whitelisted_tags: &WhitelistedTags,
        blacklisted_tags: &BlacklistedTags,
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
                    if !self.allowed_at(
                        dim_ref,
                        candidate,
                        being,
                        &WhitelistedSpawnTileTags(whitelisted_tags.clone()),
                        &BlacklistedSpawnTileTags(blacklisted_tags.clone()),
                    ) {
                        continue;
                    }
                    return Some(candidate);
                }
            }
        }

        None
    }

    pub fn allowed_at(
        &mut self,
        dim_ref: DimensionRef,
        gpos: GlobalTilePos,
        entity: Entity,
        whitelisted_tags: &WhitelistedSpawnTileTags,
        blacklisted_tags: &BlacklistedSpawnTileTags,
    ) -> bool {
        let moving_anchor = gpos.to_pixelpos();
        self.occupied_gposes.clear();

        let (collision_zone, facing_dir) = self.resolve_collision_context_for_entity(entity);
        let default_collision_zone;
        let collision_zone_ptr = match collision_zone {
            Cow::Borrowed(collision_zone) => collision_zone as *const InteractionZone,
            Cow::Owned(collision_zone) => {
                default_collision_zone = collision_zone;
                &default_collision_zone as *const InteractionZone
            }
        };
        let collision_zone = unsafe { &*collision_zone_ptr };

        collision_zone.gather_zone_positions(facing_dir, moving_anchor, &mut self.occupied_gposes);
        if self.occupied_gposes.is_empty() {
            self.occupied_gposes.push(gpos);
        }
        let can_phase = self.wallphaser_query.get(entity).is_ok();

        let mut all_tiles_failed = true;
        let mut has_whitelist_match = whitelisted_tags.0.is_empty();
        let mut has_blacklist_match = false;
        let occupied_gposes_len = self.occupied_gposes.len();
        let occupied_gposes_ptr = self.occupied_gposes.as_ptr();
        for occupied_idx in 0..occupied_gposes_len {
            let occupied_gpos = unsafe { *occupied_gposes_ptr.add(occupied_idx) };
            for &other_being in self.beings_at_gpos.get_beings_at_pos(dim_ref, occupied_gpos).iter() {
                if other_being == entity {
                    continue;
                }
                let target_zones = self.interaction_zones.get(other_being).ok();
                let coli_zone = target_zones
                    .and_then(|zones| zones.get_collision_mask().cloned())
                    .unwrap_or_else(InteractionZone::collision_default_zone);
                let target_direction = self.tile_gathering_params.cardinal_direction_query.get(other_being)
                    .cloned()
                    .unwrap_or_default();
                let Ok(gpos) = self.gpos_query.get(other_being) else {
                    continue;
                };
                let target_anchor: Vec2 = gpos.to_pixelpos();
                let intersects = coli_zone.intersects_zone(
                    target_direction,
                    target_anchor,
                    collision_zone,
                    facing_dir,
                    moving_anchor,
                );
                if intersects {
                    return false;
                }
            }
            self.gather_collision_tile_samples(dim_ref, occupied_gpos);
            let tile_samples_len = self.collision_tile_samples.len();
            let tile_samples_ptr = self.collision_tile_samples.as_ptr();
            for tile_sample_idx in 0..tile_samples_len {
                let sample = unsafe { *tile_samples_ptr.add(tile_sample_idx) };
                all_tiles_failed = false;
                if let Ok(tile_tags) = self.tags.get(sample.templ_ent) {
                    if !whitelisted_tags.0.is_empty() && tile_tags.intersects(&whitelisted_tags.0.0) {
                        has_whitelist_match = true;
                    }
                    if !blacklisted_tags.0.is_empty() && tile_tags.intersects(&blacklisted_tags.0.0) {
                        has_blacklist_match = true;
                    }
                }
                if can_phase {
                    continue;
                }
                if sample.dead_despawning {
                    continue;
                }
                if self.walk_speed.get(sample.templ_ent).cloned().unwrap_or_default().is_extremely_low() {
                    return false;
                }
                let Ok(interaction_zones) = self.interaction_zones.get(sample.templ_ent) else {
                    continue;
                };
                let blocks_here = interaction_zones.interaction_zones_intersect(
                    InteractionZones::COLLISION,
                    collision_zone,
                    sample.direction,
                    sample.tile_origin.to_pixelpos(),
                    facing_dir,
                    moving_anchor,
                );
                if blocks_here {
                    return false;
                }
            }
            self.collision_tile_samples.clear();
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

    pub fn is_blocked_at(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity, ) -> bool {
        self.is_blocked_at_impl_except_dead_despawning(dim_ref, gpos, being, true, )
    }

    pub fn is_blocked_at_tiles_only(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity, ) -> bool {
        self.is_blocked_at_impl_except_dead_despawning(dim_ref, gpos, being, false, )
    }

    fn is_blocked_at_impl_except_dead_despawning(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, being: Entity, include_beings: bool) -> bool {
        let moving_anchor = gpos.to_pixelpos();
        let (collision_zone, facing_dir) = self.resolve_collision_context_for_entity(being);
        let default_collision_zone;
        let collision_zone_ptr = match collision_zone {
            Cow::Borrowed(collision_zone) => collision_zone as *const InteractionZone,
            Cow::Owned(collision_zone) => {
                default_collision_zone = collision_zone;
                &default_collision_zone as *const InteractionZone
            }
        };
        let collision_zone = unsafe { &*collision_zone_ptr };

        self.occupied_gposes.clear();
        collision_zone.gather_zone_positions(facing_dir, moving_anchor, &mut self.occupied_gposes);
        if self.occupied_gposes.is_empty() {
            self.occupied_gposes.push(gpos);
        }
        let occupied_gposes = self.occupied_gposes.clone();
        if include_beings {
            for occupied_gpos in occupied_gposes.iter().copied() {
                for &other_being in self.beings_at_gpos.get_beings_at_pos(dim_ref, occupied_gpos).iter() {
                    if other_being == being {
                        continue;
                    }
                    let target_zones = self.interaction_zones.get(other_being).ok();
                    let target_zone = target_zones
                        .and_then(|zones| zones.get_collision_mask().cloned())
                        .unwrap_or_else(InteractionZone::collision_default_zone);
                    let Ok(target_direction) = self.tile_gathering_params.cardinal_direction_query.get_mut(other_being) else {
                        return true;
                    };
                    let Ok(gpos) = self.gpos_query.get(other_being) else {
                        return true;
                    };
                    let target_anchor = gpos.to_pixelpos();
                    let target_direction = *target_direction;

                    let intersects = target_zone.intersects_zone(
                        target_direction,
                        target_anchor,
                        collision_zone,
                        facing_dir,
                        moving_anchor,
                    );
                    if intersects {
                        return true;
                    }
                }
            }
        }

        let can_phase = self.wallphaser_query.get(being).is_ok();
        if can_phase {
            return false;
        }

        let mut all_tiles_failed = true;
        for occupied_gpos in occupied_gposes.iter().copied() {
            self.gather_collision_tile_samples(dim_ref, occupied_gpos);
            let tile_samples = self.collision_tile_samples.drain(..).collect::<Vec<_>>();
            for sample in tile_samples {
                all_tiles_failed = false;
                if sample.dead_despawning {
                    continue;
                }
                if self.walk_speed.get(sample.templ_ent).cloned().unwrap_or_default().is_extremely_low() {
                    return true;
                }
                let Ok(interaction_zones) = self.interaction_zones.get(sample.templ_ent) else {
                    continue;
                };
                let blocks_here = interaction_zones.interaction_zones_intersect(
                    InteractionZones::COLLISION,
                    collision_zone,
                    sample.direction,
                    sample.tile_origin.to_pixelpos(),
                    facing_dir,
                    moving_anchor,
                );
                if blocks_here {
                    return true;
                }
            }
        }
        if all_tiles_failed {
            trace!("No tile found at position {:?} in dimension {:?} for movement blocking check.", gpos, dim_ref);
            return true;
        }
        false
    }

    fn gather_tile_templs_at<'a>(
        &self,
        dim_ref: DimensionRef,
        gpos: GlobalTilePos,
        out: &'a mut Vec<Entity>,
    ) -> &'a [Entity] {
        out.clear();
        let macro_chunk_pos = gpos.to_chunkpos().to_macrochunk_pos();
        let Some(&macro_chunk_ent) = self.loaded_macro_chunks.0.get(&(dim_ref, macro_chunk_pos)) else {
            return out.as_slice();
        };
        let Ok(macro_chunk_tile_indices) = self.macro_chunk_tile_indices.get(macro_chunk_ent) else {
            return out.as_slice();
        };
        let Ok(tile_indexing) = self.tile_indexing_query.single() else {
            return out.as_slice();
        };
        let macro_chunk_anchor = macro_chunk_pos.to_chunkpos().to_tilepos();
        let Some(tile_indices) = macro_chunk_tile_indices.tile_indices_at_gpos(macro_chunk_anchor, gpos) else {
            return out.as_slice();
        };
        out.reserve(tile_indices.len());
        for &tile_index in tile_indices.iter() {
            let Some(tile_hash_id) = tile_indexing.hash_id_for_index(tile_index) else {
                continue;
            };
            let Ok(tile_templ_ent) = self.tile_map.0.get_cloned(tile_hash_id) else {
                continue;
            };
            out.push(tile_templ_ent);
        }
        out.as_slice()
    }

    fn gather_collision_tile_samples(
        &mut self,
        dim_ref: DimensionRef,
        occupied_gpos: GlobalTilePos,
    ) {
        self.collision_tile_samples.clear();
        self.tile_gathering_params.gather_tiles_at_to_drain(dim_ref, occupied_gpos);
        if !self.tile_gathering_params.to_drain.is_empty() {
            let tile_entities_len = self.tile_gathering_params.to_drain.len();
            let tile_entities_ptr = self.tile_gathering_params.to_drain.as_ptr();
            self.collision_tile_samples.reserve(tile_entities_len);
            for tile_entity_idx in 0..tile_entities_len {
                let tile_entity = unsafe { *tile_entities_ptr.add(tile_entity_idx) };
                let Ok(templ_ref) = self.templ_ref_query.get(tile_entity) else {
                    continue;
                };
                let Ok(tile_origin) = self.gpos_query.get(tile_entity) else {
                    continue;
                };
                let fallback_direction = self
                    .tile_gathering_params
                    .cardinal_direction_query
                    .get(tile_entity)
                    .cloned()
                    .unwrap_or_default();
                self.collision_tile_samples.push(CollisionTileSample {
                    templ_ent: templ_ref.0,
                    tile_origin: *tile_origin,
                    direction: resolve_tile_direction(&self.hash_id_query, &self.card_at_gpos, templ_ref.0, *tile_origin, fallback_direction),
                    dead_despawning: self.will_despawn_query.get(tile_entity).is_ok(),
                });
            }
            self.tile_gathering_params.to_drain.clear();
        } else {
            let mut fallback_templs = Vec::new();
            let templ_ents = self.gather_tile_templs_at(dim_ref, occupied_gpos, &mut fallback_templs);
            let templ_ents_len = templ_ents.len();
            let templ_ents_ptr = templ_ents.as_ptr();
            self.collision_tile_samples.reserve(templ_ents_len);
            for templ_ent_idx in 0..templ_ents_len {
                let templ_ent = unsafe { *templ_ents_ptr.add(templ_ent_idx) };
                self.collision_tile_samples.push(CollisionTileSample {
                    templ_ent,
                    tile_origin: occupied_gpos,
                    direction: resolve_tile_direction(&self.hash_id_query, &self.card_at_gpos, templ_ent, occupied_gpos, CardinalDirection::default()),
                    dead_despawning: false,
                });
            }
        }
    }

    pub fn find_closest_allowe_gpos(
        &mut self,
        dim_ref: DimensionRef,
        target_gpos: GlobalTilePos,
        being: Entity,
        whitelisted_tags: &WhitelistedTags,
        blacklisted_tags: &BlacklistedTags,
    ) -> Option<GlobalTilePos> {
        if self.allowed_at(
            dim_ref,
            target_gpos,
            being,
            &WhitelistedSpawnTileTags(whitelisted_tags.clone()),
            &BlacklistedSpawnTileTags(blacklisted_tags.clone()),
        ) {
            return Some(target_gpos);
        }

        const MAX_SPIRAL_RADIUS: i32 = 256;
        for radius in 1..=MAX_SPIRAL_RADIUS {
            let min = -radius;
            let max = radius;
            for x in min..=max {
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(x, min));
                if self.allowed_at(
                    dim_ref,
                    candidate,
                    being,
                    &WhitelistedSpawnTileTags(whitelisted_tags.clone()),
                    &BlacklistedSpawnTileTags(blacklisted_tags.clone()),
                ) {
                    return Some(candidate);
                }
            }
            for y in (min + 1)..=max {
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(max, y));
                if self.allowed_at(
                    dim_ref,
                    candidate,
                    being,
                    &WhitelistedSpawnTileTags(whitelisted_tags.clone()),
                    &BlacklistedSpawnTileTags(blacklisted_tags.clone()),
                ) {
                    return Some(candidate);
                }
            }
            for x in (min..max).rev() {
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(x, max));
                if self.allowed_at(
                    dim_ref,
                    candidate,
                    being,
                    &WhitelistedSpawnTileTags(whitelisted_tags.clone()),
                    &BlacklistedSpawnTileTags(blacklisted_tags.clone()),
                ) {
                    return Some(candidate);
                }
            }
            for y in ((min + 1)..max).rev() {
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(min, y));
                if self.allowed_at(
                    dim_ref,
                    candidate,
                    being,
                    &WhitelistedSpawnTileTags(whitelisted_tags.clone()),
                    &BlacklistedSpawnTileTags(blacklisted_tags.clone()),
                ) {
                    return Some(candidate);
                }
            }
        }
        error!(target: "asd", "No valid tile found for {:?} in {:?} around {}", being, dim_ref, target_gpos);
        None
    }
}

impl<'w, 's> Deref for BlockingTileParamSet<'w, 's> {
    type Target = TileGatheringParamSet<'w, 's>;

    fn deref(&self) -> &Self::Target {
        &self.tile_gathering_params
    }
}

impl<'w, 's> DerefMut for BlockingTileParamSet<'w, 's> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tile_gathering_params
    }
}

#[derive(SystemParam)]
pub struct EntitiesAtGposParamSet<'w> {
    beings_at_gpos: Res<'w, BeingsAtGpos>,
    items_at_gpos: Res<'w, ItemsAtGpos>,
}

impl<'w> EntitiesAtGposParamSet<'w> {
    pub fn gather_entities_at(&self, out: &mut Vec<Entity>, dim_ref: DimensionRef, gpos: GlobalTilePos) {
        out.extend(self.beings_at_gpos.get_beings_at_pos(dim_ref, gpos).iter().copied());
        out.extend(self.items_at_gpos.items_at_pos(dim_ref, gpos).iter().copied());
    }
}
