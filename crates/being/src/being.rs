use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::being_shared::*;
use movement::MovementSystems;
use tilemap_shared::BeingsAtGpos;

use common::{AppRegisterAndReplicateExt, common_states::AssetLoading};
use game_common::{
    HostSystems,
    game_common::{GameplaySystems, StatefulSessionSystems},
};
use sprite::AcSpriteSystems;


use crate::{
    being_components::*,
    being_inst_template::BeingInstTemplateSystems,
    being_behavior_systems::*,
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
    .init_resource::<BeingsAtGpos>()
    .init_resource::<AiNavGrids>()

    .add_systems(Update, (
        (
            (add_activates_chunks, cross_portal).in_set(HostSystems),
            on_control_change,
            (
                sync_predator_config_from_sources,
                add_predator_behavior_components,
                tick_hunger,
                sync_ai_nav_grids.before(tilemap::chunking::despawn_chunks),
                predator_hunt_behavior,
            ).chain(),
        ).in_set(GameplaySystems),
    ))
    .add_systems(
        FixedUpdate,
        sync_beings_at_gpos
            .in_set(GameplaySystems)
            .before(MovementSystems),
    )
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
    .replicate::<ControlledByClient>()

    .replicate_filtered::<ChildOf, With<Being>>()




    ;
}
