use bevy::prelude::*;
use being_shared::Grounding;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::{HostSystems, game_common::{GameplaySystems, StatefulSessionSystems} };

use crate::{being_resources::*, being_systems::*, being_components::*};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(Update, (
        (
            (add_activates_chunks, cross_portal).in_set(HostSystems),
            on_control_change,
        ).in_set(GameplaySystems),
    ))

    /*
    .add_systems(OnEnter(AssetLoading::LoadingAssetsIntoHandles), (
        (

        ),
    ))
     */
    
    .register_type::<Being>()
    .register_type::<ControlledBy>()
    .register_type::<Grounding>()
    .register_type::<Controls>()

    .register_type::<FollowerOf>()
    .register_type::<Followers>()
    .register_type::<CharacterCreatedBy>()
    .register_type::<CreatedCharacters>()
    .register_type::<IsHumanControlled>()
    
    .replicate::<PlayerDirectControllable>()
    
    .replicate::<CharacterCreatedBy>()
    .replicate::<IsHumanControlled>()
    .replicate::<Being>()
    .replicate::<ControlledBy>()
    .replicate_once::<Grounding>()
    .replicate::<FollowerOf>()

    .replicate_filtered::<ChildOf, With<Being>>()
    .replicate_filtered::<Transform, With<Being>>()
    ;
}


