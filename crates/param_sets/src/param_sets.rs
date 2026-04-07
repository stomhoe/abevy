

use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use common::common_components::HashId;
use common::common_tag_components::TagSet;
use common::log_targets::POSITION_SEARCH;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use ::being_shared::*;
use game_common::game_common_components::*;
use player_shared::player_components::*;
use ::tilemap_shared::{BlacklistedSpawnTileTagsRef, WhitelistedSpawnTileTagsRef};
use tilemap::chunking::*;
use tilemap::tile::*;

mod blocking_tile_param_set_collision;
mod blocking_tile_param_set_samples;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GposSearchConfig {
    pub start_ring_ix: u16,
    pub max_ring_ix: u16,
    pub radius_increase_per_ring: u16,
    pub sample_spacing: u16,
}
impl Default for GposSearchConfig {
    fn default() -> Self {
        Self::concentric(3, 2)
    }
}
impl GposSearchConfig {
    pub const fn concentric(radius_increase_per_ring: u16, sample_spacing: u16) -> Self {
        Self {
            start_ring_ix: 0,
            max_ring_ix: u16::MAX,
            radius_increase_per_ring,
            sample_spacing,
        }
    }

    pub const fn thorough() -> Self {
        Self {
            start_ring_ix: 0,
            max_ring_ix: u16::MAX,
            radius_increase_per_ring: 1,
            sample_spacing: 0,
        }
    }

    pub const fn wander() -> Self {
        Self {
            start_ring_ix: 2,
            max_ring_ix: u16::MAX,
            radius_increase_per_ring: 3,
            sample_spacing: 2,
        }
    }

    pub const fn with_start_ring_ix(mut self, start_ring_ix: u16) -> Self {
        self.start_ring_ix = start_ring_ix;
        self
    }

    pub const fn with_max_ring_ix(mut self, max_ring_ix: u16) -> Self {
        self.max_ring_ix = max_ring_ix;
        self
    }
}

#[derive(Clone, Copy, )]
struct CollisionTileSample {
    templ_ent: Entity,
    tile_origin: GlobalTilePos,
    direction: CardinalDirection,
    dead_despawning: bool,
}


/// system which uses this must be put .in_set(PreChunkDespawnReaders)
#[allow(unused_parens, )]
#[derive(SystemParam)]
pub struct BlockingTileParamSet<'w, 's> {
    tile_gathering_params: TileGatheringParamSet<'w, 's>,
    wallphaser_query: Query<'w, 's, (), (With<WallPhaser>, common::AnyDisabling)>,
    will_despawn_query: Query<'w, 's, (), (With<Dead>, With<DespawnOnDeath>, common::AnyDisabling)>,
    templ_ref_query: Query<'w, 's, &'static TemplEntiRef, common::AnyDisabling>,
    pub gpos_query: Query<'w, 's, &'static mut GlobalTilePos, common::AnyDisabling>,
    walk_speed: Query<'w, 's, &'static WalkSpeedMultIfOnTop, common::AnyDisabling>,
    race_ref_query: Query<'w, 's, &'static RaceRef, common::AnyDisabling>,
    bit_ref_query: Query<'w, 's, &'static BitRef, common::AnyDisabling>,
    controlled_by_query: Query<'w, 's, &'static ComputedBy, common::AnyDisabling>,
    host_player_query: Query<'w, 's, Entity, (With<Player>, With<HostPlayer>)>,
    tags: Query<'w, 's, &'static TagSet, >,
    interaction_zones: Query<'w, 's, &'static InteractionZones, common::AnyDisabling>,
    macro_chunk_tile_indices: Query<'w, 's, &'static MacroChunkU16IndexMatrix, common::AnyDisabling>,
    tile_indexing_query: Query<'w, 's, &'static TileU16IndexHashIdMapping, >,
    hash_id_query: Query<'w, 's, &'static HashId, common::AnyDisabling>,
    loaded_macro_chunks: Res<'w, LoadedMacroChunks>,
    tile_map: Res<'w, TileEntityMap>,
    card_at_gpos: Res<'w, CardinalDirAtGpos>,
    beings_at_gpos: Res<'w, BeingsAtGpos>,
    occupied_gposes: Local<'s, Vec<GlobalTilePos>>,
    collision_tile_samples: Local<'s, Vec<CollisionTileSample>>,
    gposes_set: Local<'s, HashSet<GlobalTilePos>>,
    allowed_candidates_preferred: Local<'s, Vec<GlobalTilePos>>,
    allowed_candidates_expanded: Local<'s, Vec<GlobalTilePos>>,
    allowed_candidates_combined: Local<'s, Vec<GlobalTilePos>>,
    island_open: Local<'s, Vec<GlobalTilePos>>,
    island_visited: Local<'s, HashSet<GlobalTilePos>>,
    island_set: Local<'s, HashSet<GlobalTilePos>>,
}
#[allow(unused_parens, )]
impl<'w, 's> BlockingTileParamSet<'w, 's> {
    pub fn get_being_bit_ref(&self, being: Entity) -> Option<&BitRef> {
        self.bit_ref_query.get(being).ok()
    }

    pub fn get_being_race_ref(&self, being: Entity) -> Option<&RaceRef> {
        self.race_ref_query.get(being).ok()
    }

    pub fn find_nearest_unblocked_gpos_in_chunk(
        &mut self,
        dim_ref: DimensionRef,
        anchor: GlobalTilePos,
        being: Entity,
        whitelisted_tags: &WhitelistedTags,
        blacklisted_tags: &BlacklistedTags,
    ) -> Option<GlobalTilePos> {
        self.find_nearest_unblocked_gpos_in_chunk_refs(
            dim_ref,
            anchor,
            being,
            &WhitelistedSpawnTileTagsRef(whitelisted_tags),
            &BlacklistedSpawnTileTagsRef(blacklisted_tags),
        )
    }

    pub fn find_nearest_unblocked_gpos_in_chunk_refs(
        &mut self,
        dim_ref: DimensionRef,
        anchor: GlobalTilePos,
        being: Entity,
        whitelisted_tags: &WhitelistedSpawnTileTagsRef<'_>,
        blacklisted_tags: &BlacklistedSpawnTileTagsRef<'_>,
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
                    if !self.allowed_at_refs(
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

    pub fn allowed_at(
        &mut self,
        dim_ref: DimensionRef,
        gpos: GlobalTilePos,
        entity: Entity,
        whitelisted_tags: &WhitelistedSpawnTileTags,
        blacklisted_tags: &BlacklistedSpawnTileTags,
    ) -> bool {
        self.allowed_at_refs(
            dim_ref,
            gpos,
            entity,
            &whitelisted_tags.as_ref(),
            &blacklisted_tags.as_ref(),
        )
    }

    pub fn allowed_at_refs(
        &mut self,
        dim_ref: DimensionRef,
        gpos: GlobalTilePos,
        entity: Entity,
        whitelisted_tags: &WhitelistedSpawnTileTagsRef<'_>,
        blacklisted_tags: &BlacklistedSpawnTileTagsRef<'_>,
    ) -> bool {
        let moving_anchor = gpos.to_pixelpos();
        self.occupied_gposes.clear();

        let (collision_zone, facing_dir) = blocking_tile_param_set_collision::resolve_collision_context_for_entity(self, entity);
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
            blocking_tile_param_set_samples::gather_collision_tile_samples(self, dim_ref, occupied_gpos);
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
        let (collision_zone, facing_dir) = blocking_tile_param_set_collision::resolve_collision_context_for_entity(self, being);
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
            blocking_tile_param_set_samples::gather_collision_tile_samples(self, dim_ref, occupied_gpos);
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
            if let Ok(controlled_by) = self.controlled_by_query.get(being) {
                if self.host_player_query.get(controlled_by.client_ent).is_ok() {
                    return false;
                }
            }
            return true;
        }
        false
    }

    pub fn find_closest_allowed_gpos(
        &mut self,
        dim_ref: DimensionRef,
        target_gpos: GlobalTilePos,
        being: Entity,
        search_config: GposSearchConfig,
        whitelisted_tags: &WhitelistedTags,
        blacklisted_tags: &BlacklistedTags,
    ) -> Option<GlobalTilePos> {
        self.find_closest_allowed_gpos_refs(
            dim_ref,
            target_gpos,
            being,
            search_config,
            &WhitelistedSpawnTileTagsRef(whitelisted_tags),
            &BlacklistedSpawnTileTagsRef(blacklisted_tags),
        )
    }

    pub fn find_closest_allowed_gpos_refs(
        &mut self,
        dim_ref: DimensionRef,
        target_gpos: GlobalTilePos,
        being: Entity,
        search_config: GposSearchConfig,
        whitelisted_tags: &WhitelistedSpawnTileTagsRef<'_>,
        blacklisted_tags: &BlacklistedSpawnTileTagsRef<'_>,
    ) -> Option<GlobalTilePos> {
        const MAX_TRIES: usize = 10000;
        let mut tries = 0usize;
        if self.allowed_at_refs(
            dim_ref,
            target_gpos,
            being,
            whitelisted_tags,
            blacklisted_tags,
        ) {
            return Some(target_gpos);
        }
        tries += 1;

        const MAX_RINGS: i32 = 256;
        use std::f32::consts::PI;
        self.gposes_set.clear();
        let start_ring_ix = search_config.start_ring_ix.max(1);
        let max_ring_ix = search_config.max_ring_ix.min(MAX_RINGS as u16);
        if start_ring_ix > max_ring_ix {
            error!(target: POSITION_SEARCH, "No valid tile found for {:?} in {:?} around {} after {} tries", being, dim_ref, target_gpos, tries);
            return None;
        }
        for ring_i in start_ring_ix..=max_ring_ix {
            let ring_radius = i32::from(ring_i)
                .saturating_mul(i32::from(search_config.radius_increase_per_ring.max(1)));
            let radius_f = ring_radius as f32;
            let circumference = 2.0 * PI * radius_f;
            let sample_spacing = f32::from(search_config.sample_spacing);
            let sample_count = if search_config.sample_spacing == 0 {
                circumference.ceil() as i32
            } else {
                (circumference / sample_spacing.max(0.0001)).ceil() as i32
            }
            .max(1);
            for sample_i in 0..sample_count {
                if tries >= MAX_TRIES {
                    error!(target: POSITION_SEARCH, "Failed to find allowed gpos for {:?} in {:?} around {} after {} tries", being, dim_ref, target_gpos, MAX_TRIES);
                    return None;
                }
                let angle = 2.0 * PI * sample_i as f32 / sample_count as f32;
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(
                    (radius_f * angle.cos()).round() as i32,
                    (radius_f * angle.sin()).round() as i32,
                ));
                if !self.gposes_set.insert(candidate) {
                    continue;
                }
                tries += 1;
                if self.allowed_at_refs(
                    dim_ref,
                    candidate,
                    being,
                    whitelisted_tags,
                    blacklisted_tags,
                ) {
                    return Some(candidate);
                }
            }
        }
        error!(target: POSITION_SEARCH, "No valid tile found for {:?} in {:?} around {} after {} tries", being, dim_ref, target_gpos, tries);
        None
    }

    pub fn find_allowed_gposes_in_area(
        &mut self,
        dim_ref: DimensionRef,
        target_gpos: GlobalTilePos,
        needed_count: usize,
        preferred_radius_tiles: Option<u16>,
        hard_max_radius_tiles: Option<u16>,
        gather_all_gpos_within_same_allowed_island: bool,
        only_same_island: bool,
        being: Entity,
        whitelisted_tags: &WhitelistedTags,
        blacklisted_tags: &BlacklistedTags,
        out: &mut Vec<GlobalTilePos>,
    ) {
        self.find_allowed_gposes_in_area_refs(
            dim_ref,
            target_gpos,
            needed_count,
            preferred_radius_tiles,
            hard_max_radius_tiles,
            gather_all_gpos_within_same_allowed_island,
            only_same_island,
            being,
            &WhitelistedSpawnTileTagsRef(whitelisted_tags),
            &BlacklistedSpawnTileTagsRef(blacklisted_tags),
            out,
        );
    }

    pub fn find_allowed_gposes_in_area_refs(
        &mut self,
        dim_ref: DimensionRef,
        target_gpos: GlobalTilePos,
        needed_count: usize,
        preferred_radius_tiles: Option<u16>,
        hard_max_radius_tiles: Option<u16>,
        gather_all_gpos_within_same_allowed_island: bool,
        only_same_island: bool,
        being: Entity,
        whitelisted_tags: &WhitelistedSpawnTileTagsRef<'_>,
        blacklisted_tags: &BlacklistedSpawnTileTagsRef<'_>,
        out: &mut Vec<GlobalTilePos>,
    ) {
        out.clear();
        if needed_count == 0 {
            return;
        }
        out.reserve(needed_count);
        let preferred_radius = i32::from(preferred_radius_tiles.unwrap_or(16));
        let mut hard_max_radius = i32::from(hard_max_radius_tiles.unwrap_or_else(|| preferred_radius_tiles.unwrap_or(16)));
        if hard_max_radius < preferred_radius {
            hard_max_radius = preferred_radius;
        }
        let min_x = target_gpos.0.x - hard_max_radius;
        let max_x = target_gpos.0.x + hard_max_radius;
        let min_y = target_gpos.0.y - hard_max_radius;
        let max_y = target_gpos.0.y + hard_max_radius;
        let preferred_radius_sqr = preferred_radius.saturating_mul(preferred_radius);
        let hard_max_radius_sqr = hard_max_radius.saturating_mul(hard_max_radius);
        let is_within_radius = |candidate: GlobalTilePos, radius_sqr: i32| {
            let delta = candidate.0 - target_gpos.0;
            delta.x.saturating_mul(delta.x) + delta.y.saturating_mul(delta.y) <= radius_sqr
        };

        self.allowed_candidates_preferred.clear();
        self.allowed_candidates_expanded.clear();
        self.allowed_candidates_combined.clear();
        self.island_open.clear();
        self.island_visited.clear();
        self.island_set.clear();
        self.gposes_set.clear();
        for radius in 0..=hard_max_radius {
            let min = -radius;
            let max = radius;
            for x in min..=max {
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(x, min));
                if candidate.0.x < min_x || candidate.0.x > max_x || candidate.0.y < min_y || candidate.0.y > max_y {
                    continue;
                }
                if !is_within_radius(candidate, hard_max_radius_sqr) {
                    continue;
                }
                if !self.allowed_at_refs(dim_ref, candidate, being, whitelisted_tags, blacklisted_tags) {
                    continue;
                }
                if !self.gposes_set.insert(candidate) {
                    continue;
                }
                if is_within_radius(candidate, preferred_radius_sqr) {
                    self.allowed_candidates_preferred.push(candidate);
                } else {
                    self.allowed_candidates_expanded.push(candidate);
                }
            }
            for y in (min + 1)..=max {
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(max, y));
                if candidate.0.x < min_x || candidate.0.x > max_x || candidate.0.y < min_y || candidate.0.y > max_y {
                    continue;
                }
                if !is_within_radius(candidate, hard_max_radius_sqr) {
                    continue;
                }
                if !self.allowed_at_refs(dim_ref, candidate, being, whitelisted_tags, blacklisted_tags) {
                    continue;
                }
                if !self.gposes_set.insert(candidate) {
                    continue;
                }
                if is_within_radius(candidate, preferred_radius_sqr) {
                    self.allowed_candidates_preferred.push(candidate);
                } else {
                    self.allowed_candidates_expanded.push(candidate);
                }
            }
            for x in (min..max).rev() {
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(x, max));
                if candidate.0.x < min_x || candidate.0.x > max_x || candidate.0.y < min_y || candidate.0.y > max_y {
                    continue;
                }
                if !is_within_radius(candidate, hard_max_radius_sqr) {
                    continue;
                }
                if !self.allowed_at_refs(dim_ref, candidate, being, whitelisted_tags, blacklisted_tags) {
                    continue;
                }
                if !self.gposes_set.insert(candidate) {
                    continue;
                }
                if is_within_radius(candidate, preferred_radius_sqr) {
                    self.allowed_candidates_preferred.push(candidate);
                } else {
                    self.allowed_candidates_expanded.push(candidate);
                }
            }
            for y in ((min + 1)..max).rev() {
                let candidate = GlobalTilePos(target_gpos.0 + IVec2::new(min, y));
                if candidate.0.x < min_x || candidate.0.x > max_x || candidate.0.y < min_y || candidate.0.y > max_y {
                    continue;
                }
                if !is_within_radius(candidate, hard_max_radius_sqr) {
                    continue;
                }
                if !self.allowed_at_refs(dim_ref, candidate, being, whitelisted_tags, blacklisted_tags) {
                    continue;
                }
                if !self.gposes_set.insert(candidate) {
                    continue;
                }
                if is_within_radius(candidate, preferred_radius_sqr) {
                    self.allowed_candidates_preferred.push(candidate);
                } else {
                    self.allowed_candidates_expanded.push(candidate);
                }
            }
        }

        if !gather_all_gpos_within_same_allowed_island {
            self.allowed_candidates_combined.reserve(
                self.allowed_candidates_preferred.len() + self.allowed_candidates_expanded.len(),
            );
            self.allowed_candidates_combined.extend(self.allowed_candidates_preferred.iter().copied());
            self.allowed_candidates_combined.extend(self.allowed_candidates_expanded.iter().copied());
        } else {
            let seed = self.allowed_candidates_preferred
                .first()
                .copied()
                .or_else(|| self.allowed_candidates_expanded.first().copied());
            let Some(seed) = seed else {
                return;
            };
            self.island_open.reserve(needed_count.max(16));
            self.island_open.push(seed);
            self.island_visited.insert(seed);
            let mut open_idx = 0usize;
            while open_idx < self.island_open.len() {
                let current = self.island_open[open_idx];
                open_idx += 1;
                if !self.island_set.insert(current) {
                    continue;
                }
                self.allowed_candidates_combined.push(current);
                for delta in [
                    IVec2::new(1, 0),
                    IVec2::new(-1, 0),
                    IVec2::new(0, 1),
                    IVec2::new(0, -1),
                ] {
                    let neighbor = GlobalTilePos(current.0 + delta);
                    if neighbor.0.x < min_x || neighbor.0.x > max_x || neighbor.0.y < min_y || neighbor.0.y > max_y {
                        continue;
                    }
                    if !self.island_visited.insert(neighbor) || !self.gposes_set.contains(&neighbor) {
                        continue;
                    }
                    self.island_open.push(neighbor);
                }
            }

            if !only_same_island && self.allowed_candidates_combined.len() < needed_count {
                for candidate in self.allowed_candidates_preferred.iter().copied() {
                    if self.island_set.insert(candidate) {
                        self.allowed_candidates_combined.push(candidate);
                    }
                }
                for candidate in self.allowed_candidates_expanded.iter().copied() {
                    if self.island_set.insert(candidate) {
                        self.allowed_candidates_combined.push(candidate);
                    }
                }
            }
        }

        if self.allowed_candidates_preferred.is_empty() && self.allowed_candidates_expanded.is_empty() {
            return;
        }
        if gather_all_gpos_within_same_allowed_island && self.allowed_candidates_combined.is_empty() {
            return;
        }

        let sqr_dist = |a: GlobalTilePos, b: GlobalTilePos| {
            let delta = a.0 - b.0;
            delta.x * delta.x + delta.y * delta.y
        };
        self.gposes_set.clear();
        if gather_all_gpos_within_same_allowed_island {
            self.allowed_candidates_preferred.retain(|candidate| {
                self.allowed_candidates_combined.contains(candidate)
            });
            self.allowed_candidates_expanded.retain(|candidate| {
                self.allowed_candidates_combined.contains(candidate)
            });
        }

        if !self.allowed_candidates_preferred.is_empty() {
            let remaining = &mut self.allowed_candidates_preferred;
            let mut first_choice = None;
            let mut first_choice_dist = -1;
            for (idx, candidate) in remaining.iter().enumerate() {
                let candidate_dist = sqr_dist(*candidate, target_gpos);
                if candidate_dist > first_choice_dist {
                    first_choice = Some(idx);
                    first_choice_dist = candidate_dist;
                }
            }
            if let Some(first_choice_idx) = first_choice {
                let next = remaining.swap_remove(first_choice_idx);
                out.push(next);
                self.gposes_set.insert(next);
            }

            while out.len() < needed_count && !remaining.is_empty() {
                let mut best_idx = None;
                let mut best_min_dist = -1;
                let mut best_anchor_dist = -1;
                for (idx, candidate) in remaining.iter().enumerate() {
                    if self.gposes_set.contains(candidate) {
                        continue;
                    }
                    let mut min_dist = i32::MAX;
                    for picked in out.iter() {
                        let dist = sqr_dist(*candidate, *picked);
                        if dist < min_dist {
                            min_dist = dist;
                        }
                    }
                    let anchor_dist = sqr_dist(*candidate, target_gpos);
                    if min_dist > best_min_dist || (min_dist == best_min_dist && anchor_dist > best_anchor_dist) {
                        best_idx = Some(idx);
                        best_min_dist = min_dist;
                        best_anchor_dist = anchor_dist;
                    }
                }
                let Some(best_idx) = best_idx else {
                    break;
                };
                let next = remaining.swap_remove(best_idx);
                out.push(next);
                self.gposes_set.insert(next);
            }
        }

        if out.len() < needed_count && !self.allowed_candidates_expanded.is_empty() {
            self.allowed_candidates_expanded.sort_by_key(|candidate| sqr_dist(*candidate, target_gpos));
            for candidate in self.allowed_candidates_expanded.iter().copied() {
                if self.gposes_set.contains(&candidate) {
                    continue;
                }
                out.push(candidate);
                self.gposes_set.insert(candidate);
                if out.len() >= needed_count {
                    break;
                }
            }
        }
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
