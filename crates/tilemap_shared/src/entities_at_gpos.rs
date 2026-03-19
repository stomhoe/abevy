use bevy::ecs::entity::EntityHashMap;
#[allow(unused_imports, )]use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use smallvec::SmallVec;

use crate::{CardinalDirection, DimensionRef, GlobalTilePos, InteractionZones, SmallEntiArr};

type BeingOccupiedPositions = SmallVec<[GlobalTilePos; 16]>;

#[derive(Resource, Debug, Default)]
pub struct SpriteTilesAtGpos(pub HashMap<(DimensionRef, GlobalTilePos), SmallEntiArr>);
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
pub struct BeingsAtGpos {
    pub by_pos: HashMap<(DimensionRef, GlobalTilePos), SmallEntiArr>,
    by_ent: EntityHashMap<(DimensionRef, BeingOccupiedPositions)>,
}
impl BeingsAtGpos {
    pub fn get_beings_at_pos(&self, dim: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.by_pos
            .get(&(dim, gpos))
            .map(|entities| entities.as_slice())
            .unwrap_or(&[])
    }
    pub fn update_being_occupy(&mut self, being_ent: Entity, dim_ref: DimensionRef, colmask: &[GlobalTilePos]) {
        let Some((prev_dim_ref, prev_positions)) = self.by_ent.get(&being_ent) else {
            self.occupy_colmask(being_ent, dim_ref, colmask);
            return;
        };
        if *prev_dim_ref == dim_ref && prev_positions.as_slice() == colmask {
            return;
        }
        self.remove_being_ent_entries(being_ent);
        self.occupy_colmask(being_ent, dim_ref, colmask);
    }
    pub fn remove_being_ent_entries(&mut self, being_ent: Entity) {
        let Some((dim_ref, positions)) = self.by_ent.remove(&being_ent) else {
            return;
        };
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
            }
        }
    }



    fn occupy_colmask(&mut self, being_ent: Entity, dim_ref: DimensionRef, positions: &[GlobalTilePos]) {
        for gpos in positions.iter().copied() {
            self.by_pos.entry((dim_ref, gpos)).or_default().push(being_ent);
        }
        self.by_ent.insert(being_ent, (dim_ref, SmallVec::from_slice(positions)));
    }
}

#[derive(Resource, Debug, Default)]
pub struct ItemsAtGpos(pub HashMap<(DimensionRef, GlobalTilePos), SmallEntiArr>);
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
