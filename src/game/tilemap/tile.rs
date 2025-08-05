use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::*;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::{tilemap::tile::tile_components::TileWeightedSampler, LocalAssetsLoadingState, ReplicatedAssetsLoadingState};
use crate::game::tilemap::tile::{
    tile_systems::*,
    tiling_init_systems::*,
    tile_resources::*,
};
mod tile_systems;
mod tiling_init_systems;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_utils;
pub mod tile_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TileSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TilingInitSystems {
    Local,
    Replicated,
}


//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Tile)) del módulo tilemap !!
pub struct TilePlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                update_tile_hash_value, update_tile_name, flip_tile_along_x,
                (add_tile_weighted_samplers_to_map, ).run_if(not(server_or_singleplayer)),
            ))
  
            .add_systems(OnEnter(LocalAssetsLoadingState::Complete), (
                init_shaders.before(add_shaders_to_map),
                add_shaders_to_map.before(init_tiles),
                init_tiles.before(add_tiles_to_map),
                add_tiles_to_map
            ).in_set(TilingInitSystems::Local))
            .add_systems(OnEnter(ReplicatedAssetsLoadingState::Complete), (
                (init_tile_weighted_samplers.before(add_tile_weighted_samplers_to_map),
                add_tile_weighted_samplers_to_map).run_if(server_or_singleplayer),
            ).in_set(TilingInitSystems::Replicated))

            .add_plugins((
                RonAssetPlugin::<ShaderRepeatTexSeri>::new(&["rep1shader.ron"]),
                RonAssetPlugin::<TileSeri>::new(&["tile.ron"]),
                RonAssetPlugin::<TileWeightedSamplerSeri>::new(&["sampler.ron"]),
            ))
            //.add_client_trigger::<TilingEntityMap>(Channel::Ordered)
            .add_server_trigger::<TilingEntityMap>(Channel::Ordered)
            
            .make_trigger_independent::<TilingEntityMap>()
            .add_observer(client_map_server_tiling)
            
            .replicate::<TileWeightedSampler>()
        ;
    }
}

