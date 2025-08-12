use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;
use common::common_states::{AssetsLoadingState, };

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::tile::{
    tile_components::*, tile_resources::*, tile_systems::*, tiling_init_systems::*
};
mod tile_systems;
mod tiling_init_systems;
pub mod tile_components;
pub mod tile_resources;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;



#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, (
            update_tile_hash_value, update_tile_name, flip_tile_along_x,
            (add_tile_weighted_samplers_to_map, ).run_if(not(server_or_singleplayer)),
        ))

        .add_systems(OnEnter(AssetsLoadingState::LocalFinished), (
            init_shaders.before(add_shaders_to_map),
            add_shaders_to_map.before(init_tiles),
            init_tiles.before(add_tiles_to_map),
            add_tiles_to_map
        ).in_set(TilingSystems))
        .add_systems(OnEnter(AssetsLoadingState::ReplicatedFinished), (
            (init_tile_weighted_samplers.before(add_tile_weighted_samplers_to_map),
            add_tile_weighted_samplers_to_map).run_if(server_or_singleplayer),
        ).in_set(TilingSystems))

        .add_plugins((
            RonAssetPlugin::<ShaderRepeatTexSeri>::new(&["rep1shader.ron"]),
            RonAssetPlugin::<TileSeri>::new(&["tile.ron"]),
            RonAssetPlugin::<TileWeightedSamplerSeri>::new(&["sampler.ron"]),
        ))
        .register_type::<GlobalTilePos>()
        
        .add_server_trigger::<TilingEntityMap>(Channel::Unordered)
        .make_trigger_independent::<TilingEntityMap>()

        .register_type::<ShaderRepeatTexSerisHandles>()
        .register_type::<ShaderRepeatTexSeri>()
        .register_type::<TileSerisHandles>()
        .register_type::<TileSeri>()
        .register_type::<TileWeightedSamplerSerisHandles>()
        .register_type::<TileWeightedSamplerSeri>()
        .register_type::<TilingEntityMap>()
        .register_type::<TileShaderEntityMap>()
        .register_type::<TileShader>()
        .register_type::<TileShaderRef>()
        .register_type::<RepeatingTexture>()

    ;
}

