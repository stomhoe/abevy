use bevy::prelude::*;
use superstate::superstate_plugin;

use crate::game::{being::{being_systems::*, class::ClassPlugin, race::RacePlugin}, IngameSystems};

// Module being
pub mod being_components;
mod being_systems;

//mod being_events;
pub mod being_resources;

mod race;
pub mod sprite;
mod class;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSystems;

pub struct BeingsPlugin;
#[allow(unused_parens)]
impl Plugin for BeingsPlugin {
    fn build(&self, app: &mut App) {
        app
        
            .add_plugins((RacePlugin, ClassPlugin, )) 
            // .add_systems(OnEnter(AppState::StatefulGameSession), (

            // )) 
            .add_systems(Update, (handle_movement, ).in_set(MovementSystems).in_set(IngameSystems))
        ;
    }
}

