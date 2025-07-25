#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use superstate::superstate_plugin;

use crate::game::{being::{being_components::ControlledBy, being_systems::*, class::ClassPlugin, gen_template::GenTemplatePlugin, race::RacePlugin, sprite::SpritePlugin}, IngameSystems};

pub mod being_components;

//mod being_events;
pub mod being_resources;
pub mod gen_template;
pub mod sprite;
pub mod being_utils;
pub mod race;
pub mod class;
pub mod movement;
pub mod modifier;

mod being_systems;



pub struct BeingsPlugin;
#[allow(unused_parens)]
impl Plugin for BeingsPlugin {
    fn build(&self, app: &mut App) {
        app
        
            .add_plugins((SpritePlugin, RacePlugin, ClassPlugin, GenTemplatePlugin )) 
            // .add_systems(OnEnter(AppState::StatefulGameSession), (

            // )) 
            .add_systems(Update, 
                
                 (on_control_change).in_set(IngameSystems),
                
            )
            .replicate::<ControlledBy>()
        ;
    }
}

