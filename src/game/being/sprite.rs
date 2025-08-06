use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;


#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::{being::{being_components::Being, sprite::animation_resources::{AnimStateUpdated, AnimationSeri}}, ActiveGameSystems};
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
            .add_systems(FixedUpdate, (
                (animate_sprite, change_anim_state_string, apply_offsets, apply_scales, ).run_if(on_timer(Duration::from_millis(5))).in_set(ActiveGameSystems),

                ((
                    replace_string_ids_by_entities, add_spritechildren_and_comps, ).run_if(server_or_singleplayer), 
                    on_anim_state_change.run_if(server_running).run_if(on_timer(Duration::from_millis(30))),
                    become_child_of_sprite_with_category, insert_sprite_to_instance,
                ).run_if(
                    in_state(LocalAssetsLoadingState::Complete)
                )
            ).in_set(SpriteSystemsSet))
            
            .add_systems(OnEnter(LocalAssetsLoadingState::Complete), (
                init_sprite_cfgs.before(add_sprites_to_local_map,),
                add_sprites_to_local_map,
                init_animations,
            ).in_set(SpriteSystemsSet)) 

            .add_server_trigger::<SpriteCfgEntityMap>(Channel::Unordered)
            .add_server_trigger::<AnimStateUpdated>(Channel::Unreliable)
            
            .make_trigger_independent::<SpriteCfgEntityMap>()
            .add_observer(client_map_server_sprite_cfgs)
            .add_observer(on_receive_anim_state_from_server)
            //TODO TRIGGER PARA SYNQUEAR MAPAS

            .replicate_with((
                (RuleFns::<ChildOf>::default(), SendRate::EveryTick),
                (RuleFns::<SpriteHolderRef>::default(), SendRate::EveryTick),
                (RuleFns::<SpriteConfigRef>::default(), SendRate::EveryTick),
                (RuleFns::<AnimationState>::default(), SendRate::Periodic(64*5)),//NO TIENE Q SER FRECUENTE ESTE, ES RELIABLE. HACER OTRO NO RELIABLE
                //para late joiners usar Added<SpriteCfgsToBuild>
            ))

            // .replicate_with((
            //     (RuleFns::<SpriteCfgsToBuild>::default()),
            //     (RuleFns::<SpriteCfgsBuiltSoFar>::default(), SendRate::Once),//para late joiners usar Added<SpriteCfgsBuiltSoFar> 
            // ))
            .replicate::<Directionable>()


        ;
    }
}

