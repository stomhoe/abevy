use bevy::prelude::*;
use serde::Deserialize;

use crate::terrain_gen::terrain_probe::terrain_probe_components::ProbePatternSeri;
use crate::terrain_gen::terrain_probe::terrain_probe_components::TerrainProbeTemplate;

common::define_entity_map_systems!(
    TerrainProbeTemplate,
    (),
    TerrainProbeTemplate,
    "terrain_probe",
    "terrain_probe",
    TerrainProbeTemplate,
    common::common_components::StrId,
    TerrainProbeSeri, "ron/tilemap/terrgen/probe", "probe.ron",
);

#[derive(Deserialize, Asset, TypePath)]
pub struct TerrainProbeSeri {
    pub id: String,
    pub opfilter_id: String,
    pub probe_pattern: ProbePatternSeri,
    pub step_size: Option<u16>,
    pub max_batches: Option<u16>,
    pub iterations_per_batch: Option<u16>,
    pub max_emitted_results: Option<u16>,
}
