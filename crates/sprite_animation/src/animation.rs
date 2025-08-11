
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::states::LocalAssetsLoadingState;
use game_common::game_common::SimRunningSystems;
use sprite_shared::animation_shared::*;


use crate::{animation_resources::*, animation_systems::*};

#[allow(unused_imports)] use {bevy::prelude::*, };





pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        SpritesheetAnimationPlugin::default(), 
        RonAssetPlugin::<AnimationSeri>::new(&["anim.ron"]),
    ))
    .add_systems(Update, (
        (update_animstate.before(animate_sprite), animate_sprite, ),
     
    ).in_set(AnimationSystems))

    .configure_sets(Update, (
        AnimationSystems.run_if(in_state(LocalAssetsLoadingState::Finished)),
        AnimationSystems.in_set(SimRunningSystems)
    ))

    .add_systems(OnEnter(LocalAssetsLoadingState::Finished), (
        init_animations,
    ).in_set(AnimationSystems)) 

 

    .register_type::<AnimationState>()
    .register_type::<MoveAnimActive>()



    ;
}

