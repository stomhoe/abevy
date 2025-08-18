

use bevy_replicon::prelude::*;
use common::common_states::*;
use bevy_asset_loader::prelude::*;
use dimension::dimension_resources::DimensionSerisHandles;
use sprite::sprite_resources::*;
use sprite_animation::sprite_animation_resources::AnimSerisHandles;
use tilemap::{terrain_gen::terrgen_resources::*, tile::{tile_resources::*, tile_samplers_resources::TileWeightedSamplerHandles}};

use crate::asset_loading_systems::*;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};






#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            reload_assets_ingame,
        ))
        .add_systems(OnEnter(AssetsLoadingState::LocalFinished), 
            moveon_to_replicated.run_if(in_state(TerrainGenHotLoading::DespawnAll))
        )
        .add_systems(OnEnter(AssetsLoadingState::ReplicatedFinished), (
            on_assets_loaded,
        ).run_if(in_state(TerrainGenHotLoading::DespawnAll))
        )
        .add_loading_state(
            LoadingState::new(AssetsLoadingState::LocalInProcess).continue_to_state(AssetsLoadingState::LocalFinished)
            .load_collection::<ShaderRepeatTexSerisHandles>()
            .load_collection::<TileSerisHandles>()
            .load_collection::<AnimSerisHandles>()
            .load_collection::<SpriteSerisHandles>()
        )
        .add_loading_state(
            LoadingState::new(AssetsLoadingState::ReplicatedInProcess).continue_to_state(AssetsLoadingState::ReplicatedFinished)
            .load_collection::<TileWeightedSamplerHandles>()
            .load_collection::<NoiseSerisHandles>()
            .load_collection::<OpListSerisHandles>()
            .load_collection::<DimensionSerisHandles>()

        )

    ;
}

