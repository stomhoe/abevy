#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use serde::{Deserialize, Serialize};

use crate::tile::tile_shader::tile_shader_components::TileShader;

#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderRepeatTexSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub mask_color: [f32; 4],
}

#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderVoronoiShuffleSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub voronoi_scale: f32,
    pub voronoi_scale_random: f32,
    pub voronoi_rotation: f32,
    pub mask_color: [f32; 4],
}


#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderWavySeri {
    pub id: String,
    // Path to overlay texture (use "placeholder" if unknown)
    pub img_path: String,
    pub mask_color: [f32; 4],
    pub scale: f32,
    pub time: f32,
    pub speed: f32,
    pub debug_mode: f32,
}

#[derive(Deserialize, Asset, Reflect, Default)]
pub struct ShaderRockyTerrainSeri {
    pub id: String,
    pub roughness: f32,
    pub scale: f32,
    pub height_scale: f32,
    pub color_base: [f32; 4],
    pub color_shadow: [f32; 4],
}

common::define_entity_map_systems!(
    TileShader,
    ShaderRepeatTexSeri, "ron/tilemap/tiling/shader/rep1", "rep1shader.ron",
    ShaderVoronoiShuffleSeri, "ron/tilemap/tiling/shader/voroshu", "voroshu.ron",
    ShaderWavySeri, "ron/tilemap/tiling/shader/wavy", "wavy.ron",
    ShaderRockyTerrainSeri, "ron/tilemap/tiling/shader/rocky", "rocky.ron",
);
