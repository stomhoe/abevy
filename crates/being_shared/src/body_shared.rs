use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize, MapEntities)]
#[relationship(relationship_target = HeldBody)]
pub struct BodyOf {
    #[relationship]
    #[entities]
    pub being: Entity,
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = BodyOf)]
pub struct HeldBody(Entity);
impl HeldBody {
    pub fn entity(&self) -> Entity {
        self.0
    }
}
