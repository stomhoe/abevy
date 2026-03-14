use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::being_shared::*;
use movement::MovementSystems;
use tilemap_shared::{BeingsAtGpos, GlobalTilePos, PreChunkDespawnSystems};

use common::{AppRegisterAndReplicateExt, common_states::AssetLoading};
use game_common::{
    HostSystems,
    game_common::{GameplaySystems, StatefulSessionSystems},
};
use sprite_systems::AcSpriteSystems;


use crate::{
    being_build_systems::{build_beings_from_refs, sample_sprite_normal_variations, sync_hitbox_receiver_from_sources, sync_melee_interaction_zone_from_sources},
    being_components::*,
    being_control_systems::*,
    being_inst_template::BeingInstTemplateSystems,
    being_messages::*,
    being_portal_systems::*,
    being_behavior_systems::*,
    being_systems::*,
    body::{self, prelude::*, BodySystems},
    pack::PackSystems,
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
        crate::pack::plugin,
    ))
    .init_resource::<BeingsAtGpos>()
    .init_resource::<AiNavGrids>()

    .add_systems(Update, (
        (
            build_beings_from_refs,
            sync_melee_interaction_zone_from_sources,
            sync_hitbox_receiver_from_sources,
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
                sync_ai_nav_grids,
                update_predator_chase_targets,
                chase_behavior,
                wander_behavior,
            ),
        ).in_set(GameplaySystems),
    ))
    .add_systems(
        FixedUpdate,
        (
            apply_melee_attack.in_set(HostSystems),
            validate_added_beings_have_position_and_transform.before(sync_occupancy_for_beings_at_gpos_res),
            beings_sync_transform_to_added_gpos.before(sync_occupancy_for_beings_at_gpos_res),
            sync_occupancy_for_beings_at_gpos_res.before(MovementSystems),
        )
            .in_set(GameplaySystems),
    )
    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        RaceSystems.after(tilemap::terrain::biome::BiomeSystems),
        BeingInstTemplateSystems.after(tilemap::terrain::biome::BiomeSystems),
        PackSystems.after(tilemap::terrain::biome::BiomeSystems),
        RaceSystems.after(BodySystems),
        RaceSystems.after(AcSpriteSystems),
        BeingInstTemplateSystems.after(RaceSystems),
        PackSystems.after(RaceSystems),
        PackSystems.after(BeingInstTemplateSystems),
    ))
    .replicate::<Being>()
    .replicate::<ComputedBy>()
    .replicate::<Grounding>()
    .replicate::<FollowerOf>()
    .replicate::<CharacterCreatedBy>()
    .replicate::<PlayerDirectControllable>()
    .replicate::<BodyCollisionRadius>()
    .replicate::<ToChase>()
    .replicate_filtered::<GlobalTilePos, Without<Being>>()


    .replicate::<Sentient>()
    .replicate::<HumanControlled>()

    .replicate_filtered::<ChildOf, With<Being>>()




    ;
}
