

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone)]
#[relationship(relationship_target = Inventory)]
pub struct HeldIn {
    #[relationship] #[entities]
    pub holder: Entity,
}

#[derive(Component, Clone)]
#[relationship_target(relationship = HeldIn)]
pub struct Inventory(
    Vec<Entity>
);

#[derive(Component, Default, Clone)]
pub enum Handling {
    #[default]
    OneHanded,
    TwoHanded,
    AnyHanded,
}
