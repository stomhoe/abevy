use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath)]
pub struct SgcSeri {
    pub id: String,
    pub structure_id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub args: HashMap<String, Vec<String>>,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub priority: f32,
    #[serde(default)]
    pub pdisk_mindist_and_tag: Vec<(Option<u8>, String)>,
    #[serde(default)]
    pub min_dists_from_other_structures: HashMap<String, u8>,
    #[serde(default)]
    pub exclusive_for_dimensions: Vec<String>,
    #[serde(default)]
    pub run_before_sgcs_with_tags: HashSet<String>,
    #[serde(default)]
    pub run_after_sgcs_with_tags: HashSet<String>,
    #[serde(default)]
    pub whitelisted_tags: HashSet<String>,
    #[serde(default)]
    pub blacklisted_tags: HashSet<String>,
    #[serde(default = "default_max_per_region")]
    pub max_per_region: u32,
}
fn default_max_per_region() -> u32 { 1024 }
fn default_weight() -> f32 { f32::NEG_INFINITY }
