
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
use common::common_states::AssetsLoadingState;
use game_common::game_common::SimRunningSystems;


use crate::{sprite_animation_components::*, sprite_animation_events::MoveStateUpdated, sprite_animation_resources::*, sprite_animation_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AnimationSystems;

#[allow(unused_imports)] use {bevy::prelude::*, };

pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        SpritesheetAnimationPlugin::default(), 
        RonAssetPlugin::<AnimationSeri>::new(&["anim.ron"]),
    ))
    .add_systems(Update, (
        (update_animstate, animate_sprite, ).chain(),
        update_animstate_for_clients.run_if(server_running)
    ).in_set(AnimationSystems))

    .configure_sets(Update, (
        
        AnimationSystems.in_set(SimRunningSystems),
    ))

    .add_systems(OnEnter(AssetsLoadingState::LocalFinished), (
        init_animations,
    ).in_set(AnimationSystems)) 

    .add_mapped_server_trigger::<MoveStateUpdated>(Channel::Unordered)
    .add_observer(client_receive_moving_anim)

    .replicate_once::<AnimationState>()
    .replicate_once::<MoveAnimActive>()

    .register_type::<AnimationState>()
    .register_type::<AnimationIdPrefix>()
    .register_type::<MoveAnimActive>()
    .register_type::<AnimSerisHandles>()
    .register_type::<AnimationSeri>()

    ;
}

