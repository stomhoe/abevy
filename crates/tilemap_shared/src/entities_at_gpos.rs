use bevy::ecs::entity::EntityHashMap;
#[allow(unused_imports, )]use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use smallvec::SmallVec;

use crate::{CardinalDirection, DimensionRef, GlobalTilePos, InteractionZones, EntiSmallVec};

type BeingOccupiedPositions = SmallVec<[GlobalTilePos; 4]>;

#[derive(Resource, Debug, Default)]
pub struct SpriteTilesAtGpos(pub HashMap<(DimensionRef, GlobalTilePos), EntiSmallVec>);
impl SpriteTilesAtGpos {
    pub fn tiles_at_pos(&self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.0.get(&(dim_ref, gpos)).map(|entities| entities.as_slice()).unwrap_or(&[])
    }
    fn spritetile_occupied_positions(
        gpos: GlobalTilePos,
        interaction_zones: Option<&InteractionZones>,
    ) -> Vec<GlobalTilePos> {
        let Some(interaction_zones) = interaction_zones else {
            return Vec::new();
        };
        let Some(zone) = interaction_zones.get_collision_mask() else {
            return Vec::new();
        };

        let mut positions = Vec::new();
        zone.gather_zone_positions(CardinalDirection::South, gpos.to_pixelpos(), &mut positions);
        positions
    }

    pub fn remove_tile(
        &mut self,
        dim_ref: DimensionRef,
        gpos: GlobalTilePos,
        tile_ent: Entity,
        interaction_zones: Option<&InteractionZones>,
    ) {
        for curr_gpos in Self::spritetile_occupied_positions(gpos, interaction_zones) {
            let key = (dim_ref, curr_gpos);
            let mut should_remove = false;
            if let Some(entities) = self.0.get_mut(&key) {
                let Some(idx) = entities.iter().position(|&e| e == tile_ent) else {
                    continue;
                };
                entities.swap_remove(idx);
                should_remove = entities.is_empty();
            }
            if should_remove {
                self.0.remove(&key);
            }
        }
    }
    pub fn reserve_capacity(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
    pub fn insert(
        &mut self,
        entity: Entity,
        dimension_ref: DimensionRef,
        gpos: GlobalTilePos,
        interaction_zones: Option<&InteractionZones>,
    ) {
        for curr_gpos in Self::spritetile_occupied_positions(gpos, interaction_zones) {
            self.0.entry((dimension_ref, curr_gpos)).or_default().push(entity);
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct AiNavTileBlockedGposCounts(pub HashMap<(DimensionRef, GlobalTilePos), u16>);
impl AiNavTileBlockedGposCounts {
    pub fn is_blocked(&self, dim: DimensionRef, gpos: GlobalTilePos) -> bool {
        self.0.get(&(dim, gpos)).copied().unwrap_or(0) > 0
    }

    pub fn reserve_capacity(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    fn blocked_positions_for_tile(
        tile_gpos: GlobalTilePos,
        interaction_zones: Option<&InteractionZones>,
        is_low_speed: bool,
    ) -> Vec<GlobalTilePos> {
        let mut blocked_positions = Vec::new();
        if let Some(interaction_zones) = interaction_zones {
            if let Some(collision_mask) = interaction_zones.get_collision_mask() {
                collision_mask.gather_zone_positions(
                    CardinalDirection::South,
                    tile_gpos.to_pixelpos(),
                    &mut blocked_positions,
                );
            }
        }
        if blocked_positions.is_empty() && is_low_speed {
            blocked_positions.push(tile_gpos);
        }
        if blocked_positions.is_empty() {
            return blocked_positions;
        }
        blocked_positions.sort_unstable_by_key(|pos| (pos.0.x, pos.0.y));
        blocked_positions.dedup();
        blocked_positions
    }

    pub fn insert_blocked_positions(
        &mut self,
        dim: DimensionRef,
        tile_gpos: GlobalTilePos,
        interaction_zones: Option<&InteractionZones>,
        is_low_speed: bool,
    ) -> bool {
        let blocked_positions = Self::blocked_positions_for_tile(tile_gpos, interaction_zones, is_low_speed);
        if blocked_positions.is_empty() {
            return false;
        }
        for blocked_gpos in blocked_positions {
            *self.0.entry((dim, blocked_gpos)).or_insert(0) += 1;
        }
        true
    }

    pub fn remove_blocked_positions(
        &mut self,
        dim: DimensionRef,
        tile_gpos: GlobalTilePos,
        interaction_zones: Option<&InteractionZones>,
        is_low_speed: bool,
    ) -> bool {
        let blocked_positions = Self::blocked_positions_for_tile(tile_gpos, interaction_zones, is_low_speed);
        if blocked_positions.is_empty() {
            return false;
        }
        let mut removed_any = false;
        for blocked_gpos in blocked_positions {
            let Some(count) = self.0.get_mut(&(dim, blocked_gpos)) else {
                continue;
            };
            *count = count.saturating_sub(1);
            removed_any = true;
            if *count == 0 {
                self.0.remove(&(dim, blocked_gpos));
            }
        }
        removed_any
    }

    pub fn blocked_tiles_for_dim(
        &self,
        dim: DimensionRef,
        min_tile: IVec2,
        width: u32,
        height: u32,
    ) -> Vec<UVec2> {
        let mut blocked_tiles = Vec::with_capacity(self.0.len());
        let max_x = min_tile.x + width as i32 - 1;
        let max_y = min_tile.y + height as i32 - 1;
        for (&(blocked_dim, blocked_gpos), _) in self.0.iter() {
            if blocked_dim != dim {
                continue;
            }
            let local = blocked_gpos.0 - min_tile;
            if local.x < 0 || local.y < 0 || blocked_gpos.0.x > max_x || blocked_gpos.0.y > max_y {
                continue;
            }
            blocked_tiles.push(UVec2::new(local.x as u32, local.y as u32));
        }
        blocked_tiles
    }
}

#[derive(Resource, Debug, Default)]
pub struct BeingsAtGpos {
    pub by_pos: HashMap<(DimensionRef, GlobalTilePos), EntiSmallVec>,
    pub by_dim: HashMap<DimensionRef, HashSet<GlobalTilePos>>,
    by_ent: EntityHashMap<(DimensionRef, BeingOccupiedPositions)>,
}
impl BeingsAtGpos {
    pub fn get_beings_at_pos(&self, dim: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.by_pos
            .get(&(dim, gpos))
            .map(|entities| entities.as_slice())
            .unwrap_or(&[])
    }
    pub fn occupied_positions_in_dim(&self, dim: DimensionRef) -> Option<&HashSet<GlobalTilePos>> {
        self.by_dim.get(&dim)
    }
    pub fn update_being_occupy(
        &mut self,
        being_ent: Entity,
        dim_ref: DimensionRef,
        colmask: &[GlobalTilePos],
    ) -> Option<(DimensionRef, Vec<GlobalTilePos>)> {
        let Some((prev_dim_ref, prev_positions)) = self.by_ent.get(&being_ent) else {
            self.occupy_colmask(being_ent, dim_ref, colmask);
            return None;
        };
        if *prev_dim_ref == dim_ref && prev_positions.as_slice() == colmask {
            return None;
        }
        let previous = self.remove_being_ent_entries(being_ent);
        self.occupy_colmask(being_ent, dim_ref, colmask);
        previous
    }
    pub fn remove_being_ent_entries(&mut self, being_ent: Entity) -> Option<(DimensionRef, Vec<GlobalTilePos>)> {
        let Some((dim_ref, positions)) = self.by_ent.remove(&being_ent) else {
            return None;
        };
        let removed_positions = positions.as_slice().to_vec();
        for gpos in positions {
            let key = (dim_ref, gpos);
            let Some(entities) = self.by_pos.get_mut(&key) else {
                continue;
            };
            let Some(idx) = entities.iter().position(|&e| e == being_ent) else {
                continue;
            };
            entities.swap_remove(idx);
            if entities.is_empty() {
                self.by_pos.remove(&key);
                let mut should_remove_dim_entry = false;
                if let Some(occupied_positions) = self.by_dim.get_mut(&dim_ref) {
                    occupied_positions.remove(&gpos);
                    should_remove_dim_entry = occupied_positions.is_empty();
                }
                if should_remove_dim_entry {
                    self.by_dim.remove(&dim_ref);
                }
            }
        }
        Some((dim_ref, removed_positions))
    }



    fn occupy_colmask(&mut self, being_ent: Entity, dim_ref: DimensionRef, positions: &[GlobalTilePos]) {
        for gpos in positions.iter().copied() {
            self.by_pos.entry((dim_ref, gpos)).or_default().push(being_ent);
            self.by_dim.entry(dim_ref).or_default().insert(gpos);
        }
        self.by_ent.insert(being_ent, (dim_ref, SmallVec::from_slice(positions)));
    }
}

#[derive(Resource, Debug, Default)]
pub struct ItemsAtGpos(pub HashMap<(DimensionRef, GlobalTilePos), EntiSmallVec>);
impl ItemsAtGpos {
    pub fn items_at_pos(&self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.0.get(&(dim_ref, gpos)).map(|entities| entities.as_slice()).unwrap_or(&[])
    }
    pub fn remove_item(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, item_ent: Entity) {
        let key = (dim_ref, gpos);
        let Some(entities) = self.0.get_mut(&key) else {
            return;
        };
        let Some(idx) = entities.iter().position(|&e| e == item_ent) else {
            return;
        };
        entities.swap_remove(idx);
        if entities.is_empty() {
            self.0.remove(&key);
        }
    }
    pub fn insert_item(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, item_ent: Entity) {
        self.0
            .entry((dim_ref, gpos))
            .or_default()
            .push(item_ent);
    }
}
