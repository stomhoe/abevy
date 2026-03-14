use bevy::{prelude::*};

pub use crate::terrain::terrprobe::opfilter::opfilter_seris::*;

use crate::terrain::terrprobe::opfilter::opfilter_components::OpFilter;

common::define_entity_map_systems!(
    OpFilter,
    (Without<crate::terrain::terrprobe::terrprobe_components::TerrProbeTempl>,),
    OpFilter,
    "opfilter",
    "opfilter",
    OpFilter,
    common::common_components::StrId,
    OpFilterSeri, "seri.tilemap.of", "of.ron",
);
