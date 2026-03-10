#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashSet, MapEntities}, platform::collections::HashMap, prelude::*};


use tilemap_shared::DimensionRef;
use movement::movement_components::GridLockedMovement;

use crate::being_components::*;


#[derive(Bundle, Debug, )]
pub struct PlayerStartBeingBundle(
    pub Being,
    pub Transform,
);
impl PlayerStartBeingBundle {
    pub fn new(transform: Transform) -> Self {
        Self(
            Being::default(),
            transform,
        )
    }
}

#[derive(Bundle, Debug, )]
pub struct BeingBundle(
    pub Being,
    pub DimensionRef,
    pub Transform,
    pub GridLockedMovement,
);
impl BeingBundle {
    pub fn new(dimension_ref: DimensionRef, transform: Transform) -> Self {
        Self(
            Being::default(),
            dimension_ref,
            transform,
            GridLockedMovement::default(),
        )
    }
}
