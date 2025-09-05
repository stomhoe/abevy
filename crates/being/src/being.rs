#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
use bevy_replicon::prelude::*;
use common::common_states::{AppState, GamePhase};
use game_common::{game_common::{GameplaySystems, StatefulSessionSystems}, };

use crate::{being_resources::*, being_systems::*, being_components::*};



#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_systems(Update, (
        (
            host_add_activates_chunks.run_if(server_or_singleplayer),
            on_control_change,
            cross_portal,
        ).in_set(GameplaySystems),
    ))


    .replicate::<PlayerDirectControllable>()
   
    .replicate::<CharacterCreatedBy>()
    .replicate::<HumanControlled>()
    .replicate::<Being>()
    .replicate::<DirControlledBy>()
    .replicate::<BeingAltitude>()
    .replicate::<FollowerOf>()

    .register_type::<DirControlledBy>()
    .register_type::<BeingAltitude>()
    .register_type::<Controls>()

    .register_type::<FollowerOf>()
    .register_type::<Followers>()
    .register_type::<CharacterCreatedBy>()
    .register_type::<CreatedCharacters>()
    .register_type::<HumanControlled>()

    .replicate_with((
    RuleFns::<Being>::default(),
    RuleFns::<ChildOf>::default(),
    (RuleFns::<Transform>::default(), SendRate::Periodic((64*3))),
    (RuleFns::<GlobalTransform>::default(), SendRate::Once),
    ))
    

    ;
}


