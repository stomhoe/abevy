use bevy::prelude::*;
#[allow(unused_imports, )]
use ::common::*;
use serde::{Deserialize, Serialize};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PreChunkDespawnSystems;

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct GlobalGenSettings {

    pub seed: i32,
    pub world_freq: f32,
    pub tectonic_frequency: f32,
}
const DONT_TOUCH: f32 = 1000.;
impl Default for GlobalGenSettings {
    fn default() -> Self {
        Self {
            seed: 0,
            world_freq: 20.
            /DONT_TOUCH,
            tectonic_frequency: 20.
            /DONT_TOUCH,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Default)]
pub enum SnapTransformToGpos {
    OnChange,
    #[default]
    OnAdd,
}

#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, )]
#[relationship(relationship_target = Tilemaps)]
pub struct TilemapOf {
    #[relationship]
    pub chunk: Entity,
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = TilemapOf)]
pub struct Tilemaps(Vec<Entity>);
