use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;


#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::{being::sprite::{animation_components::{AnimationState, MoveAnimActive}, animation_resources::{AnimationSeri, MoveStateUpdated}}, ActiveGameSystems};
use crate::game::{being::sprite::{
   sprite_resources::*, animation_systems::*, sprite_components::*, sprite_systems::*,
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
pub mod animation_components;
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
                (update_animstate.before(animate_sprite), animate_sprite, apply_offsets, apply_scales, ).in_set(ActiveGameSystems),

                ((
                    replace_string_ids_by_entities, add_spritechildren_and_comps, ).run_if(server_or_singleplayer), 
                    update_animstate_for_clients.run_if(server_running),
                    become_child_of_sprite_with_category, insert_sprite_to_instance,
                ).run_if(
                    in_state(LocalAssetsLoadingState::Finished)
                )
            ).in_set(SpriteSystemsSet))
            
            .add_systems(OnEnter(LocalAssetsLoadingState::Finished), (
                init_sprite_cfgs.before(add_sprites_to_local_map,),
                add_sprites_to_local_map,
                init_animations,
            ).in_set(SpriteSystemsSet)) 

            .add_server_trigger::<SpriteCfgEntityMap>(Channel::Unordered)
            .add_mapped_server_trigger::<MoveStateUpdated>(Channel::Ordered)
            
            .make_trigger_independent::<SpriteCfgEntityMap>()
            .add_observer(client_map_server_sprite_cfgs)
            .add_observer(on_receive_moving_anim_from_server)
            //TODO TRIGGER PARA SYNQUEAR MAPAS

            .replicate_with((
                (RuleFns::<ChildOf>::default(), SendRate::EveryTick),
                (RuleFns::<SpriteHolderRef>::default(), SendRate::EveryTick),
                (RuleFns::<SpriteConfigRef>::default(), SendRate::EveryTick),
                //para late joiners usar Added<SpriteCfgsToBuild>
            ))
            .replicate_with((
                (RuleFns::<AnimationState>::default(), SendRate::Periodic(64*7)),//NO TIENE Q SER FRECUENTE ESTE, ES RELIABLE. HACER OTRO NO RELIABLE
            ))
            .replicate_with((
                (RuleFns::<MoveAnimActive>::default(), SendRate::Once),//NO TIENE Q SER FRECUENTE ESTE, ES RELIABLE. HACER OTRO NO RELIABLE
            ))

            // .replicate_with((
            //     (RuleFns::<SpriteCfgsToBuild>::default()),
            //     (RuleFns::<SpriteCfgsBuiltSoFar>::default(), SendRate::Once),//para late joiners usar Added<SpriteCfgsBuiltSoFar> 
            // ))
            .replicate::<Directionable>()
            .register_type::<SpriteCfgEntityMap>()
            .register_type::<SpriteHolderRef>()
            .register_type::<AnimationState>()
            .register_type::<MoveAnimActive>()



        ;
    }
}

