
use bevy::time::common_conditions::on_timer;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
use common::common_states::AssetsLoadingState;
use game_common::game_common::SimRunningSystems;
use sprite::SpriteSystems;
use ::sprite_animation_shared::*;


use crate::{sprite_animation_components::*, sprite_animation_events::MoveStateUpdated, sprite_animation_resources::*, sprite_animation_init_systems::*, sprite_animation_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteAnimationSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        SpritesheetAnimationPlugin, 
        RonAssetPlugin::<AnimationSerialization>::new(&["anim.ron"]),
    ))
    .add_systems(Update, (
           animate_sprite, 
           update_animstate_for_clients.run_if(in_state(ServerState::Running)),  
           client_receive_moving_anim.run_if(in_state(ClientState::Connected)),   
        ).in_set(SpriteAnimationSystems),

    )

    .add_systems(FixedUpdate, ((//está en fixed update para q no le afecte lo de SimRunningSystems
            init_animation_sheet_and_handle,
        ).in_set(SpriteAnimationSystems),
    ))

    .configure_sets(Update, ( SpriteAnimationSystems.in_set(SimRunningSystems),))
    
    .configure_sets(OnEnter(AssetsLoadingState::ReplicatedFinished), (        
        SpriteAnimationSystems.before(SpriteSystems)
    ))

    .add_systems(OnEnter(AssetsLoadingState::ReplicatedFinished), (
        (init_animations, ).chain()
    ).in_set(SpriteAnimationSystems)) 

    .add_mapped_server_message::<MoveStateUpdated>(Channel::Unordered)
    //.add_observer(client_receive_moving_anim)

    .replicate_once::<AnimationState>()
    .replicate_once::<MoveAnimActive>()
    .replicate::<AnimationMain>()
    .replicate::<AnimationSerialization>()
    .replicate::<AnimationsHolder>()
    .replicate_filtered::<ChildOf, With<AnimationMain>>()


    .register_type::<AnimationState>()
    .register_type::<MoveAnimActive>()
    .register_type::<AnimSerisHandles>()
    .register_type::<AnimationSerialization>()
    .register_type::<AnimationHandle>()
    
    .register_type::<AnimationLibrary>()
    .init_resource::<AnimationLibrary>()


    ;
}

