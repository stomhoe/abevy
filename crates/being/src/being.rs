use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::being_shared::*;
use tilemap_shared::{BeingsAtGpos, GlobalTilePos};

use common::common_states::AssetLoading;
use game_common::{
    HostSystems,
    game_common::GameplaySystems,
};
use sprite_systems::AcSpriteSystems;
use tilemap::chunking::{despawn_chunks, rem_outofrange_chunks_from_activators, BeingChunkDespawned};


use crate::{
    being_behavior_systems::*, being_build_systems::{build_beings_from_refs, sample_sprite_normal_variations, sync_hitbox_receiver_from_sources, sync_melee_interaction_zone_from_sources}, being_on_chunk_despawn_systems::*, being_components::*, being_control_systems::*, being_inst_template::BeingInstTemplateSystems, being_portal_systems::*, being_systems::*, body::{self, BodySystems}, nav::{being_nav_systems::*, being_nav_wander_systems::*, being_nav_chase_systems::*}, pack::PackSystems, prelude::*, race::RaceSystems
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
    .init_resource::<ChaserNavPlans>()

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
                rebuild_chaser_nav_plans,
                chase_behavior,
                on_chunk_with_beings_attempt_unload
                    .in_set(HostSystems)
                    .before(retain_chunks_for_player_faction_chasers)
                    .run_if(on_message::<BeingChunkDespawned>),
                extend_retained_chasepaths_for_moving_player_prey,
                retain_chunks_for_player_faction_chasers
                    .after(rem_outofrange_chunks_from_activators)
                    .before(despawn_chunks),
                wander_behavior,
            ),
        ).in_set(GameplaySystems),
    ))
    .add_systems(
        Update,
        (
            apply_melee_attack.in_set(HostSystems),
            validate_added_beings_have_position_and_transform,

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
    .replicate::<Chaser>()
    .replicate_filtered::<GlobalTilePos, Without<Being>>()


    .replicate::<Sentient>()
    .replicate::<HumanControlled>()

    .replicate_filtered::<ChildOf, With<Being>>()




    ;
}
