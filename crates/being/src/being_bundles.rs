#[allow(unused_imports)]
use bevy::{
    ecs::entity::{EntityHashSet, MapEntities},
    platform::collections::HashMap,
    prelude::*,
};

use movement::movement_components::GridLockedMovement;
use tilemap_shared::{DimensionRef, GlobalTilePos};

use crate::being_components::*;

#[derive(Bundle, Debug)]
pub struct PlayerStartBeingBundle(pub Being, pub Transform);
impl PlayerStartBeingBundle {
    pub fn new(transform: Transform) -> Self {
        Self(Being::default(), transform)
    }
}

#[derive(Bundle, Debug)]
pub struct BeingBundle(
    pub Being,
    pub DimensionRef,
    pub Transform,
    pub tilemap_shared::GlobalTilePos,
    pub GridLockedMovement,
);
impl BeingBundle {
    pub fn new(dimension_ref: DimensionRef, tile_pos: GlobalTilePos) -> Self {
        Self(
            Being::default(),
            dimension_ref,
            Transform::from_translation(tile_pos.to_translation(0.0)),
            tile_pos,
            GridLockedMovement {
                visual_origin_tile: tile_pos.0,
                ..default()
            },
        )
    }
}
