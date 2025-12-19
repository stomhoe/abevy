#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
use being_shared::Grounding;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::{game_common::{GameplaySystems, StatefulSessionSystems}, };

use crate::{being_resources::*, being_systems::*, being_components::*};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(Update, (
        (
            (host_add_activates_chunks, cross_portal).run_if(in_state(ClientState::Disconnected)),
            on_control_change,
        ).in_set(GameplaySystems),
        add_beings_to_holder,
    ))

    .add_systems(OnEnter(AssetLoading::LoadingReplicatedCollections), (
        (
           spawn_egui_being_holder
        ),
    ))
    
    .register_type::<Being>()
    .register_type::<DirControlledBy>()
    .register_type::<Grounding>()
    .register_type::<Controls>()
    .register_type::<EguiBeingHolderReference>()
    .register_type::<EguiBeingHolder>()

    .register_type::<FollowerOf>()
    .register_type::<Followers>()
    .register_type::<CharacterCreatedBy>()
    .register_type::<CreatedCharacters>()
    .register_type::<IsHumanControlled>()
    
    .replicate::<PlayerDirectControllable>()
    
    .replicate::<CharacterCreatedBy>()
    .replicate::<IsHumanControlled>()
    .replicate::<Being>()
    .replicate::<DirControlledBy>()
    .replicate::<EguiBeingHolderReference>()
    .replicate_once::<Grounding>()
    .replicate::<FollowerOf>()

    .replicate_filtered::<ChildOf, With<Being>>()
    .replicate_filtered::<Transform, With<Being>>()
    ;
}


