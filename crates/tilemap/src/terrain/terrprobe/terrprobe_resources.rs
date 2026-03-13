use bevy::prelude::*;
pub use crate::terrain::terrprobe::terrprobe_seris::*;

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
