use bevy::{platform::collections::HashSet, prelude::*};

use serde::Deserialize;

use crate::terrain::terrprobe::opfilter::opfilter_components::OpFilter;

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
    #[serde(default = "default_op_i")]
    pub op_i: u16,
    #[serde(default = "default_min_val")]
    pub min_val: f32,
    #[serde(default = "default_max_val")]
    pub max_val: f32,
}

fn default_op_i() -> u16 { u16::MAX }
fn default_min_val() -> f32 { f32::NEG_INFINITY }
fn default_max_val() -> f32 { f32::INFINITY }
