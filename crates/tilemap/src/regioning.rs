use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::*;
use tilemap_shared::{AcGlobalGenSettings, ChunkPos};
use crate::{chunking_systems::*, regioning::{regioning_components::*, regioning_init_systems::*, regioning_resources::*, regioning_systems::*}, tilemap_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RegioningSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        RonAssetPlugin::<StructuredGenConfigSeri>::new(&["structure.ron"]),
    ))
    .add_systems(Update, (
        (
            plan_structures_for_new_region,
        ).in_set(RegioningSystems).run_if(in_state(TerrainHotReloading::KeepAlive))
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

    .init_resource::<LoadedRegions>()
    .register_type::<WhitelistedFilterOf>()
    .replicate::<WhitelistedFilterOf>()
;
}

pub mod regioning_components;
pub mod regioning_resources;
mod regioning_systems;
mod regioning_init_systems;