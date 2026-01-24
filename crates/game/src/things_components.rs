

use bevy::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = Inventory)]
pub struct HeldIn {
    #[relationship] #[entities]
    pub holder: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = HeldIn)]
pub struct Inventory(
    Vec<Entity>
);



#[derive(Component, Default)]
pub enum Handling {
    #[default]
    OneHanded,
    TwoHanded,
    AnyHanded,
}