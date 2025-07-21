use bevy_common_assets::ron::RonAssetPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use {crate::{game::IngameSystems, AppState}};
use crate::game::{being::sprite::{
   animation_constants::*, sprite_resources::*, animation_systems::*, sprite_components::*, sprite_systems::*
   //animation_events::*,
}, AssetLoadingState};
mod animation_systems;
mod sprite_systems;
pub mod sprite_components;
pub mod animation_constants;
pub mod sprite_constants;
pub mod sprite_resources;
//mod animation_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteSystemsSet;

pub struct SpritePlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((SpritesheetAnimationPlugin::default(), RonAssetPlugin::<SpriteDataSeri>::new(&["spritedata.ron"])))
            
            .add_systems(Update, (
                (animate_sprite, change_anim_state_string, apply_offsets_and_scales, turn_ids_into_entities, add_spritechildren_and_comps).in_set(SpriteSystemsSet).in_set(IngameSystems),
            ))
            
            .add_systems(OnEnter(AssetLoadingState::Complete), (
                init_animations,
                init_sprites,
                replace_string_ids_by_entities.after(init_sprites),
            ).in_set(SpriteSystemsSet)) 
            .init_resource::<SpriteDataIdEntityMap>()
            
        ;
    }
}

