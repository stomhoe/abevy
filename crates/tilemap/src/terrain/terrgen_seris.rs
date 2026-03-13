use bevy::prelude::*;
use common::log_targets::TERRGEN_INIT;
use serde::Deserialize;
use tilemap_shared::GlobalGenSettings;

#[derive(Deserialize, Asset, TypePath)]
pub struct FnlSeri {
    pub id: String,
    #[serde(default = "default_frequency")]
    pub frequency: f32,
    #[serde(default)]
    pub tect: bool,
    #[serde(default = "default_noise_type")]
    pub noise_type: String,
    #[serde(default = "default_fractal_type")]
    pub fractal_type: String,
    #[serde(default = "default_octaves")]
    pub octaves: u8,
    #[serde(default = "default_lacunarity")]
    pub lacunarity: f32,
    #[serde(default = "default_gain")]
    pub gain: f32,
    #[serde(default)]
    pub weighted_strength: f32,
    #[serde(default = "default_ping_pong_strength")]
    pub ping_pong_strength: f32,
    #[serde(default = "default_cellular_distance_function")]
    pub cellular_distance_function: String,
    #[serde(default = "default_cellular_return_type")]
    pub cellular_return_type: String,
    #[serde(default = "default_cellular_jitter")]
    pub cellular_jitter: f32,
    #[serde(default = "default_domain_warp_type")]
    pub domain_warp_type: String,
    #[serde(default = "default_domain_warp_amp")]
    pub domain_warp_amp: f32,
}

#[derive(Deserialize, Asset, TypePath)]
pub struct DungeonSeri {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Deserialize, Asset, TypePath, Clone, Debug)]
pub struct TerrgenSettingsSeri {
    #[serde(default)]
    pub seed: i32,
    #[serde(default = "default_global_world_freq")]
    pub world_freq: f32,
    #[serde(default = "default_global_tectonic_frequency")]
    pub tectonic_frequency: f32,
    #[serde(default = "default_global_structure_build_timeout_secs")]
    pub structure_build_timeout_secs: f64,
}

impl TerrgenSettingsSeri {
    pub fn to_terrgen_settings(&self) -> GlobalGenSettings {
        GlobalGenSettings {
            seed: self.seed,
            world_freq: self.world_freq,
            tectonic_frequency: self.tectonic_frequency,
            structure_build_timeout_secs: self.structure_build_timeout_secs,
            ..Default::default()
        }
    }
}

fn default_global_world_freq() -> f32 { 0.02 }
fn default_global_tectonic_frequency() -> f32 { 0.02 }
fn default_global_structure_build_timeout_secs() -> f64 { 4.0 }
fn default_frequency() -> f32 { 0.01 }
fn default_noise_type() -> String { "OpenSimplex2".to_string() }
fn default_fractal_type() -> String { "None".to_string() }
fn default_octaves() -> u8 { 3 }
fn default_lacunarity() -> f32 { 2.0 }
fn default_gain() -> f32 { 0.5 }
fn default_ping_pong_strength() -> f32 { 2.0 }
fn default_cellular_distance_function() -> String { "EuclideanSq".to_string() }
fn default_cellular_return_type() -> String { "Distance".to_string() }
fn default_cellular_jitter() -> f32 { 1.0 }
fn default_domain_warp_type() -> String { "OpenSimplex2".to_string() }
fn default_domain_warp_amp() -> f32 { 1.0 }

pub fn load_terrgen_settings_seri_defs() -> Vec<TerrgenSettingsSeri> {
    let db = match common::def_db::DefDatabase::<TerrgenSettingsSeri>::load_from_assets_dir_with_type(
        stringify!(TerrgenSettingsSeri),
        &["terrgen.settings.ron"],
        |_| "terrgen_settings",
    ) {
        Ok(db) => db,
        Err(err) => {
            error!(
                target: TERRGEN_INIT,
                "Failed loading TerrgenSettingsSeri defs: {err:#}"
            );
            return Vec::new();
        }
    };
    db.into_records().into_iter().map(|r| r.value).collect()
}
