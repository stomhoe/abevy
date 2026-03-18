use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;

use crate::{DimensionRef, GlobalTilePos, ReturnedVec, SizeInTiles};

#[derive(Resource, Debug, Default)]
pub struct SpriteTilesAtGpos(pub HashMap<(DimensionRef, GlobalTilePos), ReturnedVec>);
impl SpriteTilesAtGpos {
    pub fn tiles_at_pos(&self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.0.get(&(dim_ref, gpos)).map(|entities| entities.as_slice()).unwrap_or(&[])
    }
    pub fn remove_tile(&mut self, dim_ref: DimensionRef, gpos: GlobalTilePos, tile_ent: Entity, size: SizeInTiles) {
        let size = size.inner();
        for y in 0..size.y {
            for x in 0..size.x {
                let x = i64::from(gpos.0.x) + i64::from(x);
                let y = i64::from(gpos.0.y) + i64::from(y);
                if x < i64::from(i32::MIN)
                    || x > i64::from(i32::MAX)
                    || y < i64::from(i32::MIN)
                    || y > i64::from(i32::MAX)
                {
                    continue;
                }
                let curr_gpos = GlobalTilePos(IVec2::new(x as i32, y as i32));
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
    }
    pub fn reserve_capacity(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
    pub fn insert(&mut self, entity: Entity, dimension_ref: DimensionRef, gpos: GlobalTilePos, size: SizeInTiles) {
        let size = size.inner();
        for y in 0..size.y {
            for x in 0..size.x {
                let x = i64::from(gpos.0.x) + i64::from(x);
                let y = i64::from(gpos.0.y) + i64::from(y);
                if x < i64::from(i32::MIN)
                    || x > i64::from(i32::MAX)
                    || y < i64::from(i32::MIN)
                    || y > i64::from(i32::MAX)
                {
                    continue;
                }
                let curr_gpos = GlobalTilePos(IVec2::new(x as i32, y as i32));
                self.0.entry((dimension_ref, curr_gpos)).or_default().push(entity);
            }
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct BeingsAtGpos {
    pub by_pos: HashMap<(DimensionRef, GlobalTilePos), ReturnedVec>,
    by_ent: EntityHashMap<(DimensionRef, HashSet<GlobalTilePos>)>,
}
impl BeingsAtGpos {
    pub fn beings_at_pos(&self, dim_ref: DimensionRef, gpos: GlobalTilePos) -> &[Entity] {
        self.by_pos
            .get(&(dim_ref, gpos))
            .map(|entities| entities.as_slice())
            .unwrap_or(&[])
    }

    pub fn remove_being(&mut self, being_ent: Entity) {
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

    pub fn set_being_positions(
        &mut self,
        being_ent: Entity,
        dim_ref: DimensionRef,
        positions: impl IntoIterator<Item = GlobalTilePos>,
    ) {
        let positions: HashSet<GlobalTilePos> = positions.into_iter().collect();
        let Some((prev_dim_ref, prev_positions)) = self.by_ent.get(&being_ent) else {
            self.insert_being_positions(being_ent, dim_ref, positions);
            return;
        };
        if *prev_dim_ref == dim_ref && *prev_positions == positions {
            return;
        }
        self.remove_being(being_ent);
        self.insert_being_positions(being_ent, dim_ref, positions);
    }

    pub fn insert_being_positions(
        &mut self,
        being_ent: Entity,
        dim_ref: DimensionRef,
        positions: HashSet<GlobalTilePos>,
    ) {
        for gpos in positions.iter().copied() {
            self.by_pos.entry((dim_ref, gpos)).or_default().push(being_ent);
        }
        self.by_ent.insert(being_ent, (dim_ref, positions));
    }
}

#[derive(Resource, Debug, Default)]
pub struct ItemsAtGpos(pub HashMap<(DimensionRef, GlobalTilePos), ReturnedVec>);
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
