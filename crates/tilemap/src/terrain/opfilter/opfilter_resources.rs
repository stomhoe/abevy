use bevy::{platform::collections::HashSet, prelude::*};

use serde::Deserialize;

use crate::terrain::opfilter::opfilter_components::OpFilter;

common::define_entity_map_systems!(
    OpFilter,
    (),
    OpFilter,
    "opfilter",
    "opfilter",
    OpFilter,
    common::common_components::StrId,
    OpFilterSeri, "seri.tilemap.opfilter", "opfilter.ron",
);
#[derive(Deserialize, Asset, TypePath, )]
pub struct OpFilterSeri {
    pub id: String,
    pub tags: HashSet<String>,
    pub op_i: Option<u16>,
    pub min_val: Option<f32>,
    pub max_val: Option<f32>,
}
