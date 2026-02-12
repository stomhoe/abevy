use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::being_shared::*;

use common::{AppRegisterAndReplicateExt, common_states::AssetLoading};
use game_common::{
    HostSystems,
    game_common::{GameplaySystems, StatefulSessionSystems},
};
use sprite::AcSpriteSystems;


use crate::{
    being_components::*,
    being_inst_template::BeingInstTemplateSystems,
    being_systems::*,
    body::{self, BodySystems},
    race::RaceSystems,
};

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        crate::race::plugin,
        crate::sex::plugin,
        body::plugin,
        crate::being_inst_template::plugin,
    ))

    .add_systems(Update, (
        (
            (add_activates_chunks, cross_portal).in_set(HostSystems),
            on_control_change,
        ).in_set(GameplaySystems),
    ))

    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        RaceSystems.after(BodySystems),
        RaceSystems.after(AcSpriteSystems),
        BeingInstTemplateSystems.after(RaceSystems)
    ))
    .replicate::<Being>()
    .replicate::<ControlledBy>()
    .replicate::<Grounding>()
    .replicate::<FollowerOf>()
    .replicate::<CharacterCreatedBy>()
    .replicate::<PlayerDirectControllable>()
    .replicate::<BodyCollisionRadius>()


    .replicate::<Sentient>()
    .replicate::<MappedSpritesToSample>()
    .replicate::<ControlledByClient>()

    .replicate_filtered::<ChildOf, With<Being>>()




    ;
}
