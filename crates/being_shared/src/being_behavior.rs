use bevy::{ecs::entity::MapEntities, platform::collections::HashSet, prelude::*};
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, PartialEq, MapEntities)]
#[relationship(relationship_target = HuntedBy)]
pub struct HostileChase {
    #[relationship]
    #[entities]
    pub prey: Entity,
    #[serde(default)]
    pub retaliating: bool,
    #[serde(default)]
    pub retaliation_stop_distance_tiles: f32,
}

impl HostileChase {
    pub fn new(prey: Entity) -> Self {
        Self {
            prey,
            retaliating: false,
            retaliation_stop_distance_tiles: 0.0,
        }
    }

    pub fn with_retaliation(prey: Entity, retaliation_stop_distance_tiles: f32) -> Self {
        Self {
            prey,
            retaliating: true,
            retaliation_stop_distance_tiles: retaliation_stop_distance_tiles.max(0.0),
        }
    }
}

#[derive(Component, Debug)]
#[relationship_target(relationship = HostileChase)]
pub struct HuntedBy(Vec<Entity>);
