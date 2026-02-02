use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use crate::{chunking::chunking_spawn_systems::*, regioning::{dungeoning_systems::*, regioning_components::*, regioning_messages::*, regioning_resources::*, regioning_sgc_components::*, regioning_sgc_init_systems::*, regioning_systems::*}, terrain_gen::terrgen_systems::process_pending_ops_and_collect_tiles, tile::tile_systems::despawn_if_not_excepted, tilemap_systems::process_tiles_pre};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RegioningSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct StructureBuildingSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        RonAssetPlugin::<StructuredGenConfigSeri>::new(&["sgc.ron"]),
    ))
    .add_systems(Update, (
        (
            (
                drunkwalk_dungeon_building_system, 
                corridor_dungeon_building_system, 
                spiral_dungeon_building_system,
                archimedes_spiral_building_system,
                maze_dungeon_building_system,
            )
            .in_set(StructureBuildingSystems),
            offer_chunks_of_new_regions_to_dungeoning_systems, 
            claim_chunks_for_various_dungeon_types,
            read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems,
            failsafe_timeout_pending_chunks,
            add_planned_tiles_to_region,
            timeout_pending_offers,
            advance_i_on_claimlist_timeout,
            
            clonespawn_tiles_on_chunk_spawn
            .before(process_tiles_pre)//removing this breaks it
            ,
            
        ).in_set(RegioningSystems),

        despawn_empty_regions,
    ))
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (   
                init_structured_gen_configs,
            )
            .chain(),
        ).in_set(RegioningSystems))
    .add_observer(on_region_despawn_remove_from_loaded_regions)
    .add_observer(remove_sgc_from_map_on_despawn)

    .init_resource::<SgcEntityMap>()
    .init_resource::<LoadedRegions>()

    .register_type::<LoadedRegions>()
    .register_type::<SgcEntityMap>()
    .register_type::<WhitelistedFilterOf>()
    .register_type::<AcceptedFilters>()

    .register_type::<WhitelistedFilterOf>()
    .register_type::<StructuredGenConfig>()
    .register_type::<ClaimList>().register_type_data::<GridOfSgcs, InspectorEguiImpl>()
    .register_type::<GridOfSgcs>()
    .register_type::<CountsOfSgcs>()

    .replicate::<WhitelistedFilterOf>()
    .replicate::<StructuredGenConfig>()
    .replicate::<SgcsWeightedSampler>()
    .replicate::<EguiSgcHolder>()
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
pub mod regioning_messages;
pub mod regioning_sgc_components;
pub mod dungeoning;
mod dungeoning_utils;
mod regioning_systems;
mod regioning_sgc_init_systems;
mod dungeoning_systems;