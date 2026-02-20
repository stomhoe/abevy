use being_shared::BeingInstTemplate;
use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use game_common::game_common_seris::NormalDistSeri;

common::define_entity_map_systems!(
    BeingInstTemplate,
    (),
    Bit,
    "bit",
    "BIT",
    BeingInstTemplate,
    common::common_components::StrId,
    BitSeri, "seri.being.inst_templ", "bit.ron",
);
#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct BitSeri {
    pub id: String,
    pub points: u32,

    #[serde(default)]//for wild animals, leave empty
    pub fallback_faction: String,
    #[serde(default)]
    pub consecutive_name_weighted_distributions: Vec<Vec<(String, f32)>>,//to be appended to each other.
    pub race: String,
    #[serde(default)]
    pub scs_samplers: Vec<String>, // sprite weighted sampler ids, or scs ids directly
    #[serde(default)]
    pub sprites_scale_ranges: HashMap<String, (f32, f32)>,//additional

    ///multiplies both hp and sprite size (both vertically and horizontally, multiplies with the other sprite normalvariation multiplier, generated multipliers should be in the range of (0.8, 1.2))
    pub size_variation: Option<NormalDistSeri>,

    // only affects sprite width (should generate multipliers close to 1 to avoid weird rsults (0.95, 1.05))
    pub hori_variation: Option<NormalDistSeri>,
    // only affects sprite height (should generate multipliers close to 1 to avoid weird rsults (0.95, 1.05))
    pub vert_variation: Option<NormalDistSeri>,

    #[serde(default = "default_multiplier")]
    pub health_multiplier: f32,

    /// overrides race's set of weighted body trees if present
    #[serde(default)]
    pub body_tree: String,

    pub recruitment_difficulty: Option<i32>,

    #[serde(default)]
    pub whitelisted_tiles_for_spawning: HashSet<String>,
    #[serde(default)]
    pub blacklisted_tiles_for_spawning: HashSet<String>,
    #[serde(default = "default_predator_hunt_threshold")]
    pub predator_hunt_threshold: f32,
}

fn default_multiplier() -> f32 { 1.0 }
fn default_predator_hunt_threshold() -> f32 { ::being_shared::PredatorHuntThreshold::SERI_SENTINEL }
