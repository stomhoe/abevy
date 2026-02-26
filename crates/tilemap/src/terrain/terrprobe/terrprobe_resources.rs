use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

use crate::terrain::terrprobe::terrprobe_components::TerrProbeTempl;

common::define_entity_map_systems!(
    TerrProbeTempl,
    (),
    Tpt,
    "terrprobe",
    "terrprobe",
    TerrProbeTempl,
    common::common_components::StrId,
    TerrainProbeSeri, "seri.tilemap.terrprobe", "tpt.ron",
);

#[derive(Deserialize, Asset, TypePath)]
pub struct TerrainProbeSeri {
    pub id: String,
    #[serde(default)]
    pub opfilter_id: String,
    #[serde(default)]
    pub opfilter_tags: HashSet<String>,
    #[serde(default)]
    pub opfilter_var_name: String,
    #[serde(default = "default_min_val")]
    pub opfilter_min_val: f32,
    #[serde(default = "default_max_val")]
    pub opfilter_max_val: f32,
    #[serde(default)]
    pub structuregen_whitelist: HashSet<String>,
    #[serde(default)]
    pub structuregen_blacklist: HashSet<String>,
    #[serde(default)]
    pub required_tile_tags: HashSet<String>,
    pub probe_pattern: String,
    #[serde(default = "default_concentric_sample_spacing")]
    #[serde(alias = "conc_sample_spacing")]
    pub concentric_sample_spacing: f32,
    #[serde(default = "default_step_size")]
    pub step_size: u16,
    #[serde(default = "default_region_multiplier")]
    pub region_multiplier: f32,
    #[serde(default = "default_max_batches")]
    pub max_batches: u16,
    #[serde(default = "default_iterations_per_batch")]
    pub iterations_per_batch: u16,
    #[serde(default = "default_max_emitted_results")]
    pub max_emitted_results: u32,
    #[serde(default = "default_min_result_distance")]
    pub min_result_distance: u16,
    #[serde(default = "default_collect")]
    pub collect: bool,
}

fn default_step_size() -> u16 { 1 }
fn default_region_multiplier() -> f32 { 1.0 }
fn default_concentric_sample_spacing() -> f32 { 30.0 }
fn default_min_val() -> f32 { f32::NEG_INFINITY }
fn default_max_val() -> f32 { f32::INFINITY }
fn default_max_batches() -> u16 { 1000 }
fn default_iterations_per_batch() -> u16 { 10000 }
fn default_max_emitted_results() -> u32 { 1 }
fn default_min_result_distance() -> u16 { 0 }
fn default_collect() -> bool { false }
