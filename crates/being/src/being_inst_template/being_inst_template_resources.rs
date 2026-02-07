use being_shared::BeingInstTemplate;
use bevy::{asset, platform::collections::{HashMap, HashSet}, prelude::*};

#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct BitSerialization {
    pub id: String,
    pub fallback_faction: Option<String>,
    pub points: u32,

    pub consecutive_name_weighted_distributions: Option<Vec<Vec<(String, f32)>>>,//to be appended to each other.
    pub race: String,
    pub scs_samplers: Option<Vec<String>>, // sprite weighted sampler ids, or scs ids directly
    pub sprites_scale_ranges: Option<HashMap<String, (f32, f32)>>,
    pub health_multiplier: Option<f32>,

    /// overrides race's set of weighted body trees if present
    pub body_tree: Option<String>,

    pub recruitment_difficulty: Option<i32>,

    pub whitelisted_tiles_for_spawning: Option<HashSet<String>>,
    pub blacklisted_tiles_for_spawning: Option<HashSet<String>>,
}

pub type BitSeri = BitSerialization;

common::define_entity_map_systems!(
    BeingInstTemplate,
    (),
    Bit,
    "bit",
    "BIT",
    BeingInstTemplate,
    common::common_components::StrId,
    BitSeri, "ron/being/bit", "bit.ron",
);
