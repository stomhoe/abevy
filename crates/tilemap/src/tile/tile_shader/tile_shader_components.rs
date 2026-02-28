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
    TexRepeat(RepeatTexMat),
    Wavy(WavyMat),
    TerrainBlending(TerrainBlendingMat),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}
impl TileShader {
    pub fn set_image_handle(&mut self, handle: Handle<Image>) {
        match self {
            TileShader::TexRepeat(mat) => { mat.texture_overlay = handle; }
            TileShader::Wavy(mat) => { mat.texture_overlay = handle; }
            TileShader::TerrainBlending(mat) => { mat.texture_a = handle.clone(); mat.texture_b = handle; }

        }
    }
    pub fn set_multiple_image_handles(&mut self, handles: Vec<Handle<Image>>) {
        let Some(first) = handles.first().cloned() else {
            return;
        };
        match self {
            TileShader::TerrainBlending(mat) => {
                mat.texture_a = first.clone();
                mat.texture_b = handles.get(1).cloned().unwrap_or(first);
            }
            _ => {
                self.set_image_handle(first);
            }
        }
    }
}
