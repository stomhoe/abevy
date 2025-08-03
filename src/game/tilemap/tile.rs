use bevy_common_assets::ron::RonAssetPlugin;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::{AssetLoadingState};
use crate::game::tilemap::tile::{
    tile_systems::*,
    tile_resources::*,
};
mod tile_systems;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_constants;
pub mod tile_utils;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TileSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Tile)) del módulo tilemap !!
pub struct TilePlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                update_tile_hash_value, update_tile_name, flip_tile_along_x
            ))
            .add_systems(OnEnter(AssetLoadingState::Complete), (
                init_shaders.before(init_tiles),
                init_tiles.before(init_tile_weighted_samplers),
                init_tile_weighted_samplers
            ).in_set(TileSystems))
            .add_plugins((
                RonAssetPlugin::<ShaderRepeatTexSeri>::new(&["rep1shader.ron"]),
                RonAssetPlugin::<TileSeri>::new(&["tile.ron"]),
                RonAssetPlugin::<TileWeightedSamplerSeri>::new(&["sampler.ron"]),
            ))
            .init_resource::<TilingEntityMap>()
            .init_resource::<TileShaderEntityMap>()

        ;
    }
}

