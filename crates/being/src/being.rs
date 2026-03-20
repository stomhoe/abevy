use bevy::prelude::*;
use bevy_replicon::prelude::*;
use ::being_shared::{*, UnloadBeing};
use tilemap_shared::{BeingsAtGpos, ChunkWithBeingsWantsDespawn, GlobalTilePos};

use common::common_states::AssetLoading;
use game_common::{
    HostSystems,
    game_common::GameplaySystems,
};
use sprite_systems::AcSpriteSystems;
use crate::being_melee_systems::apply_melee_attack;
use crate::being_messages::MakeChunkSnapshotForChaser;
use crate::being_on_chunk_despawn_systems::{freeze_being, on_chunk_with_beings_attempt_unload};
use crate::nav::{AiNavGrids, ChaserNavPlans};

use crate::{
    being_behavior_systems::*,
    being_build_systems::{build_beings_from_refs, sample_sprite_normal_size_variations, },
    being_components::*,
    being_control_systems::*,
    being_inst_template::BeingInstTemplateSystems,
    being_portal_systems::*,
    being_systems::*,
    body::{self, BodySystems},
    nav,
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
        nav::plugin,
        crate::being_inst_template::plugin,
        crate::pack::plugin,
    ))
    .init_resource::<BeingsAtGpos>()
    .init_resource::<AiNavGrids>()
    .init_resource::<ChaserNavPlans>()

    .add_systems(Update, (
        (
            build_beings_from_refs,
            sample_sprite_normal_size_variations,
        ).chain().in_set(HostSystems),
        (
            add_activates_chunks,
            sync_player_being_chunk_ranges,
            assign_uncomputed_beings_to_host,
            cross_portal,
            freeze_being.run_if(on_message::<UnloadBeing>),
            on_chunk_with_beings_attempt_unload
                .in_set(tilemap_shared::PreChunkDespawnSystems)
                .run_if(on_message::<ChunkWithBeingsWantsDespawn>),
        ).in_set(HostSystems),
    ))
    .add_systems(Update, (
        on_control_change,
        sync_predator_config_from_sources,
        add_predator_behavior_components,
        tick_hunger,
        update_predator_chase_targets,
    ).chain())
    .add_systems(
        Update,
        (
            apply_melee_attack.in_set(HostSystems),
            validate_added_beings_have_gpos,

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
    .replicate::<DirectControllable>()
    .replicate::<Chasing>()
    .replicate_filtered::<GlobalTilePos, Without<Being>>()

    .replicate::<PackMemberRank>()

    .replicate::<Sentient>()
    .replicate::<HumanControlled>()

    .replicate_filtered::<ChildOf, With<Being>>()

    .add_message::<MakeChunkSnapshotForChaser>()
    .add_message::<UnloadBeing>()



    ;
}
