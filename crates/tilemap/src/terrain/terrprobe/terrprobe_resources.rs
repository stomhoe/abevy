use bevy::prelude::*;
use serde::Deserialize;

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
    pub opfilter_id: String,
    pub probe_pattern: String,
    #[serde(default = "default_step_size")]
    pub step_size: u16,
    #[serde(default = "default_max_batches")]
    pub max_batches: u16,
    #[serde(default = "default_iterations_per_batch")]
    pub iterations_per_batch: u16,
    #[serde(default = "default_max_emitted_results")]
    pub max_emitted_results: u16,
    #[serde(default = "default_min_result_distance")]
    pub min_result_distance: u16,
}

fn default_step_size() -> u16 { 1 }
fn default_max_batches() -> u16 { 1000 }
fn default_iterations_per_batch() -> u16 { 10000 }
fn default_max_emitted_results() -> u16 { 1 }
fn default_min_result_distance() -> u16 { 0 }
