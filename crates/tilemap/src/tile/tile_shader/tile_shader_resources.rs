#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use serde::{Deserialize, };

use crate::tile::tile_shader::tile_shader_components::TileShader;

#[derive(Deserialize, Asset, TypePath, Default)]
pub struct ShaderRepeatTexSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub mask_color: [f32; 4],
    pub tint_color: Option<[f32; 4]>,
}

#[derive(Deserialize, Asset, TypePath, Default)]
///dont delete, placeholder for putting a new shader in here
pub struct PlaceholderSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub voronoi_scale: f32,
    pub voronoi_scale_random: f32,
    pub voronoi_rotation: f32,
    pub mask_color: [f32; 4],
}


#[derive(Deserialize, Asset, TypePath, Default)]
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

#[derive(Deserialize, Asset, TypePath, Default)]
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
    ShaderRepeatTexSeri, "seri.tilemap.tile_shader.repeat_tex", "rep1shader.ron",
    //PlaceholderSeri, "ron/tilemap/tiling/shader/voroshu", "voroshu.ron",
    ShaderWavySeri, "seri.tilemap.tile_shader.wavy", "wavy.ron",
    ShaderRockyTerrainSeri, "seri.tilemap.tile_shader.rocky", "rocky.ron",
);
