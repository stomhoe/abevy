use bevy::prelude::*;
use bevy::platform::collections::HashSet;

#[derive(Asset, TypePath, Clone, Debug, Default)]
pub struct TerrainProbeSeri {
    pub id: String,
    pub opfilter_id: String,
    pub opfilter_tags: HashSet<String>,
    pub opfilter_var_name: String,
    pub opfilter_min_val: f32,
    pub opfilter_max_val: f32,
    pub structuregen_whitelist: HashSet<String>,
    pub structuregen_blacklist: HashSet<String>,
    pub required_tile_tags: HashSet<String>,
    pub probe_pattern: String,
    pub concentric_sample_spacing: f32,
    pub step_size: u16,
    pub region_multiplier: f32,
    pub max_batches: u16,
    pub iterations_per_batch: u16,
    pub max_emitted_results: u32,
    pub min_result_distance: u16,
    pub collect: bool,
}
