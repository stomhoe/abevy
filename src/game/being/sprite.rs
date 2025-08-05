use bevy_common_assets::ron::RonAssetPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;


#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::{being::sprite::animation_resources::AnimationSeri, ActiveGameSystems};
use crate::game::{being::sprite::{
   animation_constants::*, sprite_resources::*, animation_systems::*, sprite_components::*, sprite_systems::*,
   sprite_init_systems::*,
   //animation_events::*,
}, LocalAssetsLoadingState};
mod animation_systems;
mod sprite_systems;
mod sprite_init_systems;
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
                RonAssetPlugin::<SpriteConfigSeri>::new(&["sprite.ron"]),
                RonAssetPlugin::<AnimationSeri>::new(&["anim.ron"]),
            ))
            .add_systems(Update, (
                (animate_sprite, change_anim_state_string, apply_offsets, apply_scales, ).in_set(ActiveGameSystems),

                (replace_string_ids_by_entities.run_if(server_or_singleplayer), become_child_of_sprite_with_category, add_spritechildren_and_comps).run_if(
                    in_state(LocalAssetsLoadingState::Complete)
                )
            ).in_set(SpriteSystemsSet))
            
            .add_systems(OnEnter(LocalAssetsLoadingState::Complete), (
                init_sprite_cfgs.before(add_sprites_to_local_map,),
                add_sprites_to_local_map,
                init_animations,
            ).in_set(SpriteSystemsSet)) 

            //TODO TRIGGER PARA SYNQUEAR MAPAS
            .replicate::<SpriteCfgsToBuild>()
            .replicate::<Directionable>()
            .replicate_once::<SpriteCfgsBuiltSoFar>()//para late joiners usar Added<SpriteCfgsBuiltSoFar> 


        ;
    }
}

