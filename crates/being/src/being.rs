use std::time::Duration;

use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::time::common_conditions::on_timer;
use bevy_replicon::prelude::*;
use ::being_shared::body_energy::*;
use ::being_shared::*;
use faction::faction_resources::FactionRef;
use ::game_common::*;
use tilemap::terrain::terrgen_messages::ChunkTerrainBuilt;

use common::common_states::AssetLoading;
use sprite_systems::AcSpriteSystems;
use crate::being_melee_systems::*;
use crate::being_messages::*;
use crate::being_cleanup_systems::*;
use crate::being_on_chunk_despawn_systems::*;
use crate::being_enable_systems::*;
use crate::squad_build_systems::*;
use crate::being_nav::*;
use crate::being_simulation_systems::*;
use crate::being_hunt_systems::*;
use crate::being_control_systems::*;
use crate::being_build_systems::*;
use crate::being_portal_systems::*;
use ::tilemap_shared::*;

use crate::{
    being_inst_template::BeingInstTemplateSystems,
    being_resources::*,
    being_portal_resources::*,
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
    .init_resource::<BeingEntityMap>()

    .add_systems(Update, (
        (
            build_beings_from_refs,
            sample_sprite_normal_size_variations,
            assign_being_hash_ids,
        ).chain().in_set(HostSystems),
        (
            add_activates_chunks,
            activate_beings_in_first_time_loaded_chunks.run_if(on_message::<ChunkTerrainBuilt>),
            sync_being_chunk_ranges_to_resource.run_if(resource_changed::<LoadChunksAround>),
            assign_uncomputed_beings_to_host,
            sync_group_members_from_member_of,
            assign_member_ranks_on_joined_squad,
            refresh_leader_on_member_rank_change,
            rebuild_portal_crossing_index,
            cull_loaded_beings_far_from_humans.run_if(on_timer(Duration::from_secs(10))),
            faithful_sim_being.run_if(on_message::<FaithfulSimBeing>),
            unfreeze_beings_on_chunk_load.run_if(on_message::<ChunkLoaded>),
            insert_macrochunk_nav_islands.run_if(on_message::<NewMacrochunkLoaded>),
            on_chunk_with_beings_attempt_unload
                .in_set(tilemap_shared::PreChunkDespawnSystems)
                .run_if(on_message::<ChunkWithBeingsWantsDespawn>),

            instance_pack_entities.run_if(on_message::<InstantiateTemplPackEntity>),
        ).in_set(HostSystems).in_set(StatefulSessionSystems),
    ))
    .add_observer(cleanup_being_from_BeingsInCpos_on_despawn)
    .add_observer(remove_being_hash_id_from_map_on_despawn)
    .add_systems(
        Update,
        (
            on_control_change,
            sync_predator_squad_marker,
            validate_added_beings_have_gpos.in_set(HostSystems).in_set(GameplaySystems),
        ),
    )
    .add_systems(
        Update,
        (
            add_melee_target_comp_to_ai_controlled,
            update_squad_weight_sum.before(update_predator_hunting_targets),
            update_predator_hunting_targets,
            make_hunted_be_melee_targets.after(update_predator_hunting_targets),
            sync_chasing_to_hunt.after(update_predator_hunting_targets),
        ).in_set(SimRunningSystems),
    )
    .add_systems(
        Update,
        (
            (
                emit_ai_melee_attack_requests.run_if(on_timer(Duration::from_millis(30))),
                apply_melee_attack,
            ).in_set(HostSystems).in_set(SimRunningSystems),
        ),
    )
    .add_systems(
        Update,
        cross_portal.in_set(HostSystems),
    )
    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        RaceSystems.after(tilemap::terrain::biome::BiomeSystems),
        RaceSystems.after(BodySystems),
        RaceSystems.after(AcSpriteSystems),
        BeingInstTemplateSystems.after(RaceSystems),
        PackSystems.after(BeingInstTemplateSystems),
    ))
    .replicate::<Being>()
    .replicate::<ComputedBy>()
    .replicate::<Grounding>()
    .replicate::<FollowerOf>()
    .replicate::<CharacterCreatedBy>()
    .replicate::<DirectControllable>()
    .replicate::<Chasing>()
    .replicate::<FightOrFlightConfig>()
    .replicate::<FightingStyle>()
    .replicate::<WanderState>()
    .replicate::<Fleeing>()
    .replicate::<DoAvoidBlacklistedSpawnTilesForWander>()
    .replicate::<LodLevel>()
    .replicate_filtered::<GlobalTilePos, Without<Being>>()
    .replicate::<BgSimulatedIn>()
    .replicate::<BodyWeightSum>()
    .replicate::<BodyCondition>()
    .replicate::<BodyStrengthScale>()

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
