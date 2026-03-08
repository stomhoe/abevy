use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::being_shared::*;
use movement::MovementSystems;
use tilemap_shared::{BeingsAtGpos, PreChunkDespawnSystems};

use common::{AppRegisterAndReplicateExt, common_states::AssetLoading};
use game_common::{
    HostSystems,
    game_common::{GameplaySystems, StatefulSessionSystems},
};
use sprite::AcSpriteSystems;


use crate::{
    being_build_systems::{build_beings_from_refs, sample_sprite_normal_variations, sync_melee_interaction_zone_from_sources},
    being_components::*,
    being_inst_template::BeingInstTemplateSystems,
    being_behavior_systems::*,
    being_systems::*,
    body::{self, prelude::*, BodySystems},
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
    .add_observer(apply_melee_attack)

    .add_systems(Update, (
        (
            build_beings_from_refs,
            sync_melee_interaction_zone_from_sources,
            sample_sprite_normal_variations,
        ).chain().in_set(HostSystems),
        (
            (
                add_activates_chunks,
                assign_uncontrolled_beings_to_host,
                cross_portal,
            ).in_set(HostSystems),
            on_control_change,
            (
                sync_predator_config_from_sources,
                add_predator_behavior_components,
                tick_hunger,
                sync_ai_nav_grids,//.in_set(PreChunkDespawnReaders),
                update_predator_chase_targets,
                chase_behavior,
                wander_behavior,
            ),
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
    .replicate::<ToChase>()


    .replicate::<Sentient>()
    .replicate::<PlayerControlled>()

    .replicate_filtered::<ChildOf, With<Being>>()




    ;
}
