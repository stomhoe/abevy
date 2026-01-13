use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use crate::{regioning::{regioning_components::*, regioning_init_systems::*, regioning_messages::*, regioning_resources::*, regioning_systems::*}, };

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RegioningSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        RonAssetPlugin::<StructuredGenConfigSeri>::new(&["strgencfg.ron"]),
    ))
    .add_systems(Update, (
        (
            (offer_chunks_of_new_region, read_chunk_claims_for_region_and_emit_build_orders,),
            example_emit_claims_system, example_building_system,
            clonespawn_structure_tile_on_chunk_spawn,

        ).in_set(RegioningSystems)
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
    .register_type::<TilesToSpawnPerChunk>()

    .replicate::<WhitelistedFilterOf>()
    .replicate::<StructuredGenConfig>()
    .replicate::<StructuredGenCfgsWeightedMap>()
    .replicate_once::<Region>()
    
    .add_message::<OfferChunk>()
    .add_message::<ClaimedChunks>()
    .add_message::<StructureBuildOrder>()
    
    .init_resource::<LoadedRegions>()

;
}

pub mod regioning_components;
pub mod regioning_resources;
pub mod regioning_messages;
mod regioning_systems;
mod regioning_init_systems;