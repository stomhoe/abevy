use bevy_common_assets::ron::RonAssetPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;


#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::ActiveGameSystems;
use crate::game::{being::sprite::{
   animation_constants::*, sprite_resources::*, animation_systems::*, sprite_components::*, sprite_systems::*
   //animation_events::*,
}, LocalAssetsLoadingState};
mod animation_systems;
mod sprite_systems;
pub mod sprite_components;
pub mod animation_constants;
pub mod sprite_resources;
pub mod animation_resources;
//mod animation_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteSystemsSet;

pub struct SpritePlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                SpritesheetAnimationPlugin::default(), 
                RonAssetPlugin::<SpriteDataSeri>::new(&["spridat.ron"]),
                RonAssetPlugin::<AnimationSeri>::new(&["anim.ron"]),
            ))
            .add_systems(Update, (
                (animate_sprite, change_anim_state_string, apply_offsets, apply_scales, add_spritechildren_and_comps).in_set(SpriteSystemsSet).in_set(ActiveGameSystems),

                (replace_string_ids_by_entities, become_child_of_sprite_with_category).in_set(SpriteSystemsSet).run_if(
                    in_state(LocalAssetsLoadingState::Complete)
                )
            ))
            
            .add_systems(OnEnter(LocalAssetsLoadingState::Complete), (
                init_sprites,
                init_animations,
            ).in_set(SpriteSystemsSet)) 
            .replicate::<SpriteDatasChildrenStringIds>()
            .replicate::<Directionable>()


        ;
    }
}

