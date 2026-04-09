use bevy::prelude::*;
use bevy::platform::collections::HashSet;

#[derive(Asset, TypePath, Clone, Debug)]
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

impl Default for TerrainProbeSeri {
    fn default() -> Self {
        Self {
            id: String::new(),
            opfilter_id: String::new(),
            opfilter_tags: HashSet::default(),
            opfilter_var_name: String::new(),
            opfilter_min_val: f32::NEG_INFINITY,
            opfilter_max_val: f32::INFINITY,
            structuregen_whitelist: HashSet::default(),
            structuregen_blacklist: HashSet::default(),
            required_tile_tags: HashSet::default(),
            probe_pattern: String::new(),
            concentric_sample_spacing: 0.0,
            step_size: 0,
            region_multiplier: 0.0,
            max_batches: 0,
            iterations_per_batch: 0,
            max_emitted_results: 0,
            min_result_distance: 0,
            collect: false,
        }
    }
}
