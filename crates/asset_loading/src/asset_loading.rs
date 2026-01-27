

use bevy_replicon::prelude::ClientState;
use color_sample::color_sample_resources::ColorWeightedSamplerHandles;
use common::common_states::*;
use bevy_asset_loader::prelude::*;
use dimension::dimension_resources::DimensionSerisHandles;
use game_common::{GameplaySystems, };
use sprite::sprite_resources::*;
use sprite_animation::sprite_animation_resources::AnimSerisHandles;
use tilemap::{regioning::regioning_resources::*, terrain_gen::terrgen_resources::*, tile::{tile_resources::*, tile_sampler_resources::*, tile_shader::tile_shader_resources::*}};

use crate::asset_loading_systems::*;

#[allow(unused_imports)] use {bevy::prelude::*, };



#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AssetHotReloading;


#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .init_state::<AssetLoading>()
        .init_state::<AssetHotReloadState>()

        .add_systems(Update, (
            reload_assets_while_ingame,
        ))
        .add_systems(OnExit(AppState::StatefulGameSession), 
            despawn_asset_scoped_entities
        )
        .add_systems(OnEnter(ClientState::Connecting), 
            despawn_asset_scoped_entities
        )
        .add_systems(OnEnter(AssetLoading::LoadingAssetsIntoHandles), 
            despawn_asset_scoped_entities_except_spared
        )
        .add_systems(OnEnter(AssetLoading::NotStarted), 
            despawn_asset_scoped_entities
        )
        // Don't use OnExit(AssetLoading::SpawnReplicatedEntities) because clients aren't in that state

        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            on_assets_loaded.in_set(AssetHotReloading)
        ))

        .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            AssetHotReloading.run_if(in_state(AssetHotReloadState::Ongoing)).after(GameplaySystems)
        ))
        .add_loading_state(
            LoadingState::new(AssetLoading::LoadingAssetsIntoHandles).continue_to_state(AssetLoading::SpawnReplicatedEntities)
            .load_collection::<ShaderRepeatTexSerisHandles>()
            .load_collection::<ShaderVoroshuSerisHandles>()
            .load_collection::<ShaderWavySerisHandles>()
            .load_collection::<ShaderRockyTerrainSerisHandles>()
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

