use bevy::prelude::*;

use crate::game::{beings::{beings_systems::*, classes::ClassesPlugin, races::RacesPlugin}, game_systems::*, IngameSystems};

// Module being
pub mod beings_components;
mod beings_systems;
//mod being_events;
pub mod beings_resources;

mod races;
mod classes;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSystems;

pub struct BeingsPlugin;
#[allow(unused_parens)]
impl Plugin for BeingsPlugin {
    fn build(&self, app: &mut App) {
        app
        
            .add_plugins((RacesPlugin, ClassesPlugin)) 
            .add_systems(Update, (handle_movement, ).in_set(MovementSystems).in_set(IngameSystems))
        ;
    }
}