use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::prelude::*;
use common::common_states::{LocalAssetsLoadingState, ReplicatedAssetsLoadingState};

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::tile::{
    tile_components::GlobalTilePos, tile_resources::*, tile_systems::*, tiling_init_systems::*
};
mod tile_systems;
mod tiling_init_systems;
pub mod tile_components;
pub mod tile_resources;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilingSystems;

pub struct TilePlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                update_tile_hash_value, update_tile_name, flip_tile_along_x,
                (add_tile_weighted_samplers_to_map, ).run_if(not(server_or_singleplayer)),
            ))
  
            .add_systems(OnEnter(LocalAssetsLoadingState::Finished), (
                init_shaders.before(add_shaders_to_map),
                add_shaders_to_map.before(init_tiles),
                init_tiles.before(add_tiles_to_map),
                add_tiles_to_map
            ).in_set(TilingSystems))
            .add_systems(OnEnter(ReplicatedAssetsLoadingState::Finished), (
                (init_tile_weighted_samplers.before(add_tile_weighted_samplers_to_map),
                add_tile_weighted_samplers_to_map).run_if(server_or_singleplayer),
            ).in_set(TilingSystems))

            .add_plugins((
                RonAssetPlugin::<ShaderRepeatTexSeri>::new(&["rep1shader.ron"]),
                RonAssetPlugin::<TileSeri>::new(&["tile.ron"]),
                RonAssetPlugin::<TileWeightedSamplerSeri>::new(&["sampler.ron"]),
            ))
            .register_type::<TilePos>()
            .register_type::<GlobalTilePos>()
            
            

        ;
    }
}

