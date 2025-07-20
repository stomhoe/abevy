use bevy::prelude::*;
use superstate::superstate_plugin;

use crate::game::{being::{being_systems::*, class::ClassPlugin, gen_template::GenTemplatePlugin, race::RacePlugin}, IngameSystems};

// Module being
pub mod being_components;

//mod being_events;
pub mod being_resources;
pub mod gen_template;
pub mod sprite;
pub mod being_utils;

mod being_systems;
mod race;
mod class;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSystems;

pub struct BeingsPlugin;
#[allow(unused_parens)]
impl Plugin for BeingsPlugin {
    fn build(&self, app: &mut App) {
        app
        
            .add_plugins((RacePlugin, ClassPlugin, GenTemplatePlugin )) 
            // .add_systems(OnEnter(AppState::StatefulGameSession), (

            // )) 
            .add_systems(Update, (handle_movement, ).in_set(MovementSystems).in_set(IngameSystems))
        ;
    }
}

