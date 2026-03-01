use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use crate::{ regioning::{dungeoning_systems::*, natural::*, regioning_components::*, regioning_messages::*, regioning_resources::*, regioning_sgc_components::*, regioning_sgc_init_systems::*, regioning_systems::*, regioning_sgc_components::StructuredGenConfig}, };


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RegioningSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct StructureBuildingSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        plugin_structured_gen_config,
    ))
    .add_systems(Update, (
        (
            (
                drunkwalk_dungeon_building_system,
                corridor_dungeon_building_system,
                spiral_dungeon_building_system,
                archimedes_spiral_building_system,
                maze_dungeon_building_system,
                river_structure_building_system,
            )
            .in_set(StructureBuildingSystems),
            offer_chunks_of_new_regions_to_dungeoning_systems,
            (
                claim_chunks_for_various_dungeon_types,
                claim_chunks_for_river_structures,
            ),
            read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems,
            failsafe_timeout_pending_chunks,
            add_planned_tiles_to_region,
            timeout_pending_offers,
            advance_i_on_claimlist_timeout,
            clonespawn_tiles_on_chunk_spawn
            .before(crate::tilemap_systems::process_tiles_pre)//removing this breaks it
            ,
        ).in_set(RegioningSystems),
        despawn_empty_regions,
    ))
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities), (
            init_structured_gen_configs, map_structured_gen_config_id_to_entity
        ).in_set(RegioningSystems))
    .add_observer(on_region_despawn_remove_from_loaded_regions)

    .init_resource::<LoadedRegions>()
    .init_resource::<Prioritized>()
    .init_resource::<PrioritizedPerRegion>()
    .init_resource::<RiverPlans>()
    .init_resource::<RiverDebugData>()

    .replicate::<WhitelistedFilterOf>()
    .replicate::<StructuredGenConfig>()
    .replicate::<SgcsWeightedSampler>()
    .replicate_once::<Region>()

    .add_message::<OfferChunk>()
    .add_message::<ChunksClaim>()
    .add_message::<SgcPrepareTilesOrder>()
    .add_message::<StructureBuildCompliance>()
    .add_message::<RecheckRegion>()



;
}

pub mod regioning_components;
pub mod regioning_resources;
pub mod regioning_seris;
pub mod regioning_messages;
pub mod regioning_sgc_components;
pub mod dungeoning;
pub mod natural;
mod dungeoning_utils;
mod regioning_systems;
mod regioning_sgc_init_systems;
mod dungeoning_systems;
