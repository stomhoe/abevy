#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use superstate::superstate_plugin;

use crate::game::{being::{being_components::*, being_systems::*, class::ClassPlugin, gen_template::GenTemplatePlugin, movement::MovementPlugin, race::RacePlugin, sprite::SpritePlugin}, modifier::ModifierPlugin, player::player_components::Controls, GameplaySystems};

pub mod being_components;

//mod being_events;
pub mod being_resources;
pub mod gen_template;
pub mod sprite;
pub mod being_utils;
pub mod body;
pub mod race;
pub mod class;
pub mod movement;  

mod being_systems;

pub struct BeingsPlugin;
#[allow(unused_parens)]
impl Plugin for BeingsPlugin {
    fn build(&self, app: &mut App) {
        app
        
            .add_plugins((SpritePlugin, RacePlugin, ClassPlugin, GenTemplatePlugin, MovementPlugin, ModifierPlugin, )) 
            // .add_systems(OnEnter(AppState::StatefulGameSession), (

            // )) 
            .add_systems(Update, 
                
                 (on_control_change).in_set(GameplaySystems),
                
            )
            .replicate::<Being>()
            .replicate::<CpuControlled>()
            .replicate::<ControlledBy>()
            .replicate::<ControlledBy>()
            .register_type::<ControlledBy>()
            .register_type::<Controls>()
            .register_type::<FollowerOf>()
            .register_type::<Followers>()

            //.replicate_bundle::<(Being, Transform)>()//PROVISORIO

        ;
    }
}

