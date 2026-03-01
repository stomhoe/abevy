use bevy::platform::collections::HashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Default)]
pub struct ShaderRepeatTexSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub mask_color: [f32; 4],
    pub tint_color: Option<[f32; 4]>,
    #[serde(default)]
    pub blend_blacklist: HashSet<String>,
    #[serde(default)]
    pub blend_whitelist: HashSet<String>,
}

#[derive(Deserialize, Asset, TypePath, Default)]
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
    pub img_path: String,
    pub mask_color: [f32; 4],
    pub scale: f32,
    pub time: f32,
    pub speed: f32,
    pub debug_mode: f32,
}
