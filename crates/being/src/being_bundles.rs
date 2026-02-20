#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashSet, MapEntities}, platform::collections::HashMap, prelude::*};


use tilemap_shared::DimensionRef;

use crate::being_components::*;
use ::being_shared::*;


#[derive(Bundle, Debug, )]
pub struct BeingBundle(
    pub Being,
    pub DimensionRef,
    pub Transform,
);
impl BeingBundle {
    pub fn new(
        being: Being,
        dimension_ref: DimensionRef,
        transform: Transform,
    ) -> Self {
        Self (
            being,
            dimension_ref,
            transform,
        )
    }
}
