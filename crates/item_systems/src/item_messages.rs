use bevy::prelude::*;
use game_common::game_common_components::EntityZeroRef;
use tilemap_shared::{DimensionRef, GlobalTilePos};

#[derive(Message, Debug, Copy, Clone)]
pub struct GenerateItem {
    pub ezero_ref: EntityZeroRef,
    pub dim_ref: DimensionRef,
    pub dest: GenerateItemDest,
}

#[derive(Debug, Copy, Clone)]
pub enum GenerateItemDest {
    Entity(Entity),
    Gpos(GlobalTilePos),
}

impl GenerateItem {
    pub fn on_ground(ezero_ref: EntityZeroRef, dim_ref: DimensionRef, gpos: GlobalTilePos) -> Self {
        Self {
            ezero_ref,
            dim_ref,
            dest: GenerateItemDest::Gpos(gpos),
        }
    }

    pub fn on_entity(ezero_ref: EntityZeroRef, dim_ref: DimensionRef, dest_entity: Entity) -> Self {
        Self {
            ezero_ref,
            dim_ref,
            dest: GenerateItemDest::Entity(dest_entity),
        }
    }
}
