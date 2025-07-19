#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use {crate::{game::IngameSystems, AppState}};
use crate::game::beings::animation::{
   animation_systems::*,
   animation_components::*,
   animation_constants::*,
   //animation_resources::*,
   //animation_events::*,
};
mod animation_systems;
pub mod animation_components;
pub mod animation_constants;
//mod animation_resources;
//mod animation_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AnimationSystems;

pub struct AnimationPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app

            //.init_resource::<RESOURCE_NAME>()


            .add_systems(Update, (
                (animate_sprite,change_anim_state_sid).in_set(AnimationSystems).in_set(IngameSystems),
            ))

            .add_systems(OnEnter(AppState::StatefulGameSession), (
                init_animations,
                init_sprites,
            )) 
            
        ;
    }
}

