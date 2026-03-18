use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PreChunkDespawnSystems;

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
#[require(Replicated, Prefix::trunc("GlobalGenSettings"), AssetScoped, HotReload)]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct SnapTransformToGpos;

#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, )]
#[relationship(relationship_target = Tilemaps)]
pub struct TilemapOf {
    #[relationship]
    pub chunk: Entity,
}
impl TilemapOf {
    pub fn new(chunk: Entity) -> Self {
        Self { chunk }
    }
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = TilemapOf)]
pub struct Tilemaps(Vec<Entity>);
