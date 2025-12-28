

use common::common_states::*;
use bevy_asset_loader::prelude::*;
use dimension::dimension_resources::DimensionSerisHandles;
use game_common::{GameplaySystems, color_sampler_resources::ColorWeightedSamplerHandles};
use sprite::sprite_resources::*;
use sprite_animation::sprite_animation_resources::AnimSerisHandles;
use tilemap::{regioning::regioning_resources::*, terrain_gen::terrgen_resources::*, tile::{tile_resources::*, tile_sampler_resources::*, tile_shader_resources::*}};

use crate::asset_loading_systems::*;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};



#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AssetHotReloading;


#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            reload_assets_ingame,
        ))
        .add_systems(OnEnter(AssetLoading::SpawnLocalEntities), 
            moveon_to_replicated.in_set(AssetHotReloading)
        )
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            on_assets_loaded.in_set(AssetHotReloading)
        ))

        .configure_sets(OnEnter(AssetLoading::SpawnLocalEntities), (
            AssetHotReloading.run_if(in_state(TerrainHotReloading::DespawnAll))
        ))

        .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            AssetHotReloading.run_if(in_state(TerrainHotReloading::DespawnAll)).after(GameplaySystems)
        ))

        .add_loading_state(
            LoadingState::new(AssetLoading::LocalInProcess).continue_to_state(AssetLoading::SpawnLocalEntities)
        )
        .add_loading_state(
            LoadingState::new(AssetLoading::LoadingReplicatedCollections).continue_to_state(AssetLoading::SpawnReplicatedEntities)
            .load_collection::<ShaderRepeatTexSerisHandles>()
            .load_collection::<ShaderVoronoiSerisHandles>()
            .load_collection::<TileSerisHandles>()
            .load_collection::<AnimSerisHandles>()
            .load_collection::<SpriteSerisHandles>()
            .load_collection::<ColorWeightedSamplerHandles>()
            .load_collection::<TileWeightedSamplerHandles>()
            .load_collection::<NoiseSerisHandles>()
            .load_collection::<OpListSerisHandles>()
            .load_collection::<DimensionSerisHandles>()
            .load_collection::<StructureSerisHandles>()


        )

    ;
}

