#[allow(unused_imports)] use bevy::prelude::*;
pub use bevy_ecs_tilemap::tiles::*;
use common::{common_components::*, };

use crate::tile::{self, tile_shader::*};
use serde::{Deserialize, Serialize};

pub type TileShaderRef = tile::tile_shader::tile_shader_resources::TileShaderRef;
impl Default for TileShaderRef { fn default() -> Self { Self(HashId::default()) } }

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(AssetScoped, Prefix::trunc("TileShader"), Replicated)]
pub enum TileShader{
    TerrBlend(TerrBlendMat),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}
