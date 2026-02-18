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
    pub step_size: Option<u16>,
    pub max_batches: Option<u16>,
    pub iterations_per_batch: Option<u16>,
    pub max_emitted_results: Option<u16>,
    pub min_result_distance: Option<u16>,
}
