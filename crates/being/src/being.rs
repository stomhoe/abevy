use bevy::{prelude::*, };
use being_shared::{Grounding, Sentient};
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::{HostSystems, game_common::{GameplaySystems, StatefulSessionSystems} };
use sprite::AcSpriteSystems;

use crate::{being_components::*, being_resources::*, being_systems::*, being_inst_template::BeingInstTemplateSystems, race::RaceSystems};

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        crate::race::plugin,
        crate::sex::plugin,
    ))

    .add_systems(Update, (
        (
            (add_activates_chunks, cross_portal).in_set(HostSystems),
            on_control_change,
        ).in_set(GameplaySystems),
    ))

    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        RaceSystems.after(AcSpriteSystems),
        BeingInstTemplateSystems.after(RaceSystems)
    ))


    
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
    .replicate::<Grounding>()
    .replicate::<FollowerOf>()
    .replicate::<Sentient>()

    .replicate_filtered::<ChildOf, With<Being>>()
    .replicate_filtered::<Transform, With<Being>>()
    ;
}


