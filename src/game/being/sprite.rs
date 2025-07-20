use bevy_common_assets::ron::RonAssetPlugin;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use {crate::{game::IngameSystems, AppState}};
use crate::game::being::sprite::{
   animation_constants::*, animation_resources::*, animation_systems::*, sprite_components::*, sprite_systems::*
   //animation_events::*,
};
mod animation_systems;
mod sprite_systems;
pub mod sprite_components;
pub mod animation_constants;
pub mod sprite_constants;
pub mod animation_resources;
//mod animation_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteSystems;

pub struct AnimationPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((RonAssetPlugin::<SpriteDataSeri>::new(&["sprite.ron"])))
            
            .add_systems(Update, (
                (animate_sprite, change_anim_state_string, apply_offsets_and_scales, add_spritechildren_and_comps).in_set(SpriteSystems).in_set(IngameSystems),
            ))
            
            .add_systems(OnEnter(AppState::StatefulGameSession), (
                init_animations,
                init_sprites,
            )) 
            .init_resource::<IdSpriteDataEntityMap>()
            
        ;
    }
}

