#[allow(unused_imports)] use bevy::prelude::*;
pub use bevy_ecs_tilemap::tiles::*;
use common::{common_components::*, };

use crate::tile::{self, tile_shader::*};
use serde::{Deserialize, Serialize};

pub type TileShaderRef = tile::tile_shader::tile_shader_resources::TileShaderRef;
impl Default for TileShaderRef { fn default() -> Self { Self(Entity::PLACEHOLDER) } }

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
#[require(AssetScoped, Prefix::trunc("TileShader"), Replicated)]
pub enum TileShader{
    TexRepeat(MonoRepeatTextureOverlayMat),
    TwoTexRepeat(TwoOverlaysExample),
    Voronoi(VoronoiTextureOverlayMat),
    Wavy(WavyMat),
    RockyTerrain(RockyTerrainMat),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}
impl TileShader {
    pub fn set_image_handle(&mut self, handle: Handle<Image>) {
        match self {
            TileShader::TexRepeat(mat) => { mat.texture_overlay = handle; }
            TileShader::TwoTexRepeat(mat) => { mat.texture_overlay = handle.clone(); mat.texture_overlay_2 = handle; }
            TileShader::Voronoi(mat) => { mat.texture_overlay = handle; }
            TileShader::Wavy(mat) => { mat.texture_overlay = handle; }
            TileShader::RockyTerrain(_) => { } // Procedural shader, no image needed

        }
    }
}
