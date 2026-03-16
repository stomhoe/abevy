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
    #[serde(default)]
    pub spawn_pack_size_normal_dist: Option<NormalDistSeri>,
    #[serde(default)]
    pub belongs_to_packs: Vec<String>,
    #[serde(default)]
    pub biome_affinity: HashMap<String, f32>,

    #[serde(default)]
    //if empty, can spawn on any tile. if nonempty, can only spawn on tiles with at least one of these tags
    pub whitelisted_spawn_tile_tags: HashSet<String>,

    #[serde(default)]
    //if empty, can spawn on any tile. if nonempty, cannot spawn on tiles with any of these tags, except if they are also in the whitelist, in which case the whitelist takes priority and the tags in this blacklist are ignored.
    pub blacklisted_spawn_tile_tags: HashSet<String>,
}

fn default_multiplier() -> f32 { 1.0 }
fn default_predator_hunt_threshold() -> f32 { ::being_shared::PredatorHuntThreshold::SERI_SENTINEL }
fn default_melee_interaction_zone() -> InteractionZoneSeri {
    InteractionZoneSeri {
        offset_positions: vec![(0, 1)],
        radius_offset: Vec::new(),
    }
}
