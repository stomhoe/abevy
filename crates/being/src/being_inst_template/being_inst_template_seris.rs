use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use game_common::game_common_seris::NormalDistSeri;
use tilemap_shared::tilemap_seris::InteractionZoneSeri;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct BitSeri {
    pub id: String,
    pub points: u32,
    #[serde(default)]
    pub fallback_faction: String,
    #[serde(default)]
    pub consecutive_name_weighted_distributions: Vec<Vec<(String, f32)>>,
    pub race: String,
    #[serde(default)]
    pub scs_samplers: Vec<String>,
    #[serde(default)]
    pub sprites_scale_ranges: HashMap<String, (f32, f32)>,
    pub size_variation: Option<NormalDistSeri>,
    pub hori_variation: Option<NormalDistSeri>,
    pub vert_variation: Option<NormalDistSeri>,
    #[serde(default = "default_multiplier")]
    pub health_multiplier: f32,
    #[serde(default)]
    pub body_tree: String,
    pub recruitment_difficulty: Option<i32>,
    #[serde(default)]
    pub whitelisted_tiles_for_spawning: HashSet<String>,
    #[serde(default)]
    pub blacklisted_tiles_for_spawning: HashSet<String>,
    #[serde(default = "default_predator_hunt_threshold")]
    pub predator_hunt_threshold: f32,
    #[serde(default = "default_melee_interaction_zone")]
    pub melee_interaction_zone: InteractionZoneSeri,
    #[serde(default)]
    pub hitbox_hashid: String,
}

fn default_multiplier() -> f32 { 1.0 }
fn default_predator_hunt_threshold() -> f32 { ::being_shared::PredatorHuntThreshold::SERI_SENTINEL }
fn default_melee_interaction_zone() -> InteractionZoneSeri {
    InteractionZoneSeri {
        offset_positions: vec![(0, 1)],
        radius_offset: Vec::new(),
    }
}
