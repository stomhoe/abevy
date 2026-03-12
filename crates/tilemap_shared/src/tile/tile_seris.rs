use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component, Deserialize, TypePath, Clone, Default, Debug)]
pub struct InteractionZoneSeri {
    #[serde(default)]
    pub offset_positions: Vec<(i8, i8)>,
    #[serde(default)]
    pub radius_offset: Vec<(f32, (f32, f32))>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TerrblParamsSeri {
    #[serde(default)]
    pub texture_path: String,
    #[serde(default)]
    pub priority: f32,
    #[serde(default = "default_terrbl_scale")]
    pub scale: f32,
    #[serde(default)]
    pub speed: f32,
    #[serde(default)]
    pub wavy_strength: f32,
    #[serde(default)]
    pub time_offset: f32,
    #[serde(default = "default_true")]
    pub blend_enabled: bool,
    #[serde(default = "default_tint")]
    pub tint: [u8; 4],
    #[serde(default = "default_tint_mask_target_sentinel")]
    pub tint_mask_target: [u8; 4],
}
impl Default for TerrblParamsSeri {
    fn default() -> Self {
        Self {
            texture_path: String::new(),
            priority: 0.0,
            scale: default_terrbl_scale(),
            speed: 0.0,
            wavy_strength: 0.0,
            time_offset: 0.0,
            blend_enabled: default_true(),
            tint: default_tint(),
            tint_mask_target: default_tint_mask_target_sentinel(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DeleteOtherTilesSeri {
    #[serde(default)]
    pub spared_z: Vec<f32>,
    #[serde(default)]
    pub targeted_z: Vec<f32>,
    #[serde(default)]
    pub spared_tags: Vec<String>,
    #[serde(default)]
    pub targeted_tags: Vec<String>,
    #[serde(default)]
    pub extra_radius: u32,
    #[serde(default)]
    pub priority: f32,
}

fn default_terrbl_scale() -> f32 { 1e-5 }
fn default_true() -> bool { true }
fn default_tint() -> [u8; 4] { [255, 255, 255, 255] }
fn default_tint_mask_target_sentinel() -> [u8; 4] { [255, 0, 255, 0] }
