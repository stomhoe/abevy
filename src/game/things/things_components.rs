

use bevy::prelude::*;
use superstate::{SuperstateInfo};

#[derive(Component)]
#[relationship(relationship_target = Inventory)]
pub struct HeldIn {
    #[relationship] pub holder: Entity,
}

#[derive(Component)]
#[relationship_target(relationship = HeldIn)]
pub struct Inventory(Vec<Entity>);



#[derive(Component, Default)]
#[require(SuperstateInfo<Handling>)]
struct Handling; 

#[derive(Component)]
#[require(Handling)]
struct TwoHanded; 

#[derive(Component)]
#[require(Handling)]
struct OneHanded; 

#[derive(Component)]
#[require(Handling)]
struct AnyHanded; 