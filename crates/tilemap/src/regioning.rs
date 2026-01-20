use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use crate::regioning::{dungeoning_systems::*, regioning_components::*, regioning_init_systems::*, regioning_messages::*, regioning_resources::*, regioning_systems::*};

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
            (offer_chunks_of_new_regions, read_chunk_claims_for_region_and_emit_build_orders,),
            claim_chunks_for_various_dungeon_types, 
            failsafe_timeout_pending_chunks,
            add_planed_tiles_to_region,
            (drunkwalk_dungeon_building_system, advanced_dungeon_building_system).in_set(StructureBuildingSystems),
            clonespawn_tiles_on_chunk_spawn,//tiene q hacerse despues de los building systems            , // ensure missing compliances don't block region planning indefinitely            track_when_region_is_ready_for_spawning,
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
    .register_type::<LoadedRegions>()
    .register_type::<StructuredGenConfigEntityMap>()
    .register_type::<WhitelistedFilterOf>()
    .register_type::<AcceptedFilters>()

    .register_type::<WhitelistedFilterOf>()
    .register_type::<StructuredGenConfig>()
    .register_type::<RegionStructures>()
    .register_type::<RegionPlannedTiles>()

    .replicate::<WhitelistedFilterOf>()
    .replicate::<StructuredGenConfig>()
    .replicate::<StructuredGenCfgsWeightedMap>()
    .replicate_once::<Region>()
    
    .add_message::<OfferChunk>()
    .add_message::<ClaimedChunks>()
    .add_message::<StructureBuildOrder>()
    .add_message::<StructureBuildCompliance>()
    
    .init_resource::<LoadedRegions>()

;
}

pub mod regioning_components;
pub mod regioning_resources;
pub mod regioning_messages;
mod regioning_systems;
mod regioning_init_systems;
mod dungeoning_systems;