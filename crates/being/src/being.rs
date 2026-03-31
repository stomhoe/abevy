use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_replicon::prelude::*;
use ::being_shared::*;
use faction::faction_resources::FactionRef;
use tilemap::terrain::terrgen_messages::ChunkTerrainBuilt;
use tilemap_shared::{BeingsAtGpos, ChunkLoaded, ChunkWithBeingsWantsDespawn, GlobalTilePos};

use common::common_states::AssetLoading;
use game_common::{
    HostSystems,
    game_common::GameplaySystems,
};
use sprite_systems::AcSpriteSystems;
use crate::being_melee_systems::apply_melee_attack;
use crate::being_messages::{MakeChunkSnapshotForChaser, NavOrder};
use crate::being_cleanup_systems::cull_loaded_beings_far_from_humans;
use crate::being_on_chunk_despawn_systems::{faithful_sim_being, on_chunk_with_beings_attempt_unload};
use crate::being_enable_systems::unfreeze_beings_on_chunk_load;
use crate::squad_build_systems::instance_pack_entities;
use crate::being_nav::{AiNavGrids, ChaserNavPlans, SharedChaseFlowFields};
use crate::being_simulation_systems::insert_macrochunk_nav_islands;
use tilemap_shared::NewMacrochunkLoaded;

use crate::{
    being_hunt_systems::*,
    being_build_systems::*,
    being_enable_systems::*,
    being_control_systems::*,
    being_inst_template::BeingInstTemplateSystems,
    being_portal_resources::*,
    being_portal_systems::*,
    being_systems::*,
    body::{self, BodySystems},
    being_nav,
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
        being_nav::plugin,
        crate::being_inst_template::plugin,
        crate::pack::plugin,
    ))
    .init_resource::<BeingsAtGpos>()
    .init_resource::<AiNavGrids>()
    .init_resource::<SharedChaseFlowFields>()
    .init_resource::<ChaserNavPlans>()
    .init_resource::<FrozenBgSimulatedBeingsMap>()
    .init_resource::<PortalCrossingIndex>()
    .init_resource::<BeingsToEnableOnChunkLoad>()
    .init_resource::<NextPendingNaturalSpawnGroupId>()

    .add_systems(Update, (
        (
            build_beings_from_refs,
            sample_sprite_normal_size_variations,
        ).chain().in_set(HostSystems),
        (
            add_activates_chunks,
            activate_beings_in_first_time_loaded_chunks.run_if(on_message::<ChunkTerrainBuilt>),
            sync_player_being_chunk_ranges,
            assign_uncomputed_beings_to_host,
            sync_group_members_from_member_of,
            assign_member_ranks_on_joined_squad.before(refresh_leader_on_member_rank_change),
            rebuild_portal_crossing_index,
            refresh_leader_on_member_rank_change,
            cross_portal,
            cull_loaded_beings_far_from_humans.run_if(on_timer(Duration::from_secs(10))),
            faithful_sim_being.run_if(on_message::<FaithfulSimBeing>),
            unfreeze_beings_on_chunk_load.run_if(on_message::<ChunkLoaded>),
            insert_macrochunk_nav_islands.run_if(on_message::<NewMacrochunkLoaded>),
            on_chunk_with_beings_attempt_unload
                .in_set(tilemap_shared::PreChunkDespawnSystems)
                .run_if(on_message::<ChunkWithBeingsWantsDespawn>),
            instance_pack_entities.run_if(on_message::<InstantiateTemplPackEntity>),
        ).in_set(HostSystems),
    ))
    .add_systems(Update, (
        on_control_change,
        sync_predator_squad_marker,
        tick_hunger,
        update_predator_hunting_targets,
        sync_chasing_to_hunt.after(update_predator_hunting_targets),
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
    .replicate_once::<WanderState>()
    .replicate::<Fleeing>()
    .replicate::<DoAvoidBlacklistedSpawnTilesForWander>()
    .replicate::<GoTo>()
    .replicate_filtered::<GlobalTilePos, Without<Being>>()
    .replicate::<being_shared::BgSimulatedIn>()

    .replicate::<MemberRanks>()
    .replicate::<Predator>()


    .replicate::<HumanControlled>()
    .replicate::<PreventsChunkUnloading>()
    .replicate::<Hunting>()
    .replicate::<LedBy>()
    .replicate::<JoinedGroups>()
    .register_type::<JoinedGroups>()
    .register_type::<WanderState>()
    .replicate::<FactionRef>()
    .replicate::<SquadMemberOf>()
    .replicate::<crate::pack::pack_components::SquadAvgCenterPerDim>()



    .replicate_filtered::<ChildOf, With<Being>>()

    .add_message::<MakeChunkSnapshotForChaser>()
    .add_message::<NavOrder>()
    .add_message::<FaithfulSimBeing>()
    .add_message::<InstantiateTemplPackEntity>()



    ;
}
/*
 * can you make refresh_terrbl_tilemaps more change-detection-driven or messagereader-driven ?
 */
