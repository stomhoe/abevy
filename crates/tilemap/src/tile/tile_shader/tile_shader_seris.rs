use bevy::platform::collections::HashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Default)]
pub struct ShaderTerrblSeri {
    pub id: String,
    pub img_path: String,
    #[serde(default = "default_scale")]
    pub scale: f32,
    #[serde(default = "default_mask_color")]
    pub mask_color: [f32; 4],
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub wavy_strength: f32,
    #[serde(default)]
    pub blend_blacklist: HashSet<String>,
    #[serde(default)]
    pub blend_whitelist: HashSet<String>,
}

fn default_scale() -> f32 {
    1e-5
}

fn default_mask_color() -> [f32; 4] {
    [255.0, 0.0, 0.0, 255.0]
}
