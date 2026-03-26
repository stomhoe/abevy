use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use common::common_tag_components::TagSet;
use tilemap_shared::tilemap_shared_samplers::NormalDistSeri;

use being_shared::WanderConfig;

pub type SpawnWeight = f32;

pub type RankDist = NormalDistSeri;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct PackSeri {
    pub id: String,
    #[serde(default)]
    pub tags: HashSet<String>,

    #[serde(default = "default_true")]
    pub spawn_pack_entity: bool,

    #[serde(default)]
    pub spawn_being_count_normal_dist: NormalDistSeri,
    #[serde(default)]
    pub race_ids: HashMap<String, (SpawnWeight, RankDist)>,
    #[serde(default)]
    pub bit_ids: HashMap<String, (SpawnWeight, RankDist)>,
    #[serde(default = "default_center_rank_weight_multiplier")]
    pub center_rank_weight_multiplier: f32,
    #[serde(default)]
    pub center_rank_weight_multipliers: HashMap<String, f32>,
    #[serde(default)]
    pub biome_affinity: HashMap<String, f32>,
    #[serde(default)]//placeholder, dont use this
    pub behavior_on_member_attack: String,
    #[serde(default = "default_alert_effectiveness_falloff")]
    pub attack_alert_effectiveness_falloff: f32,
    #[serde(default = "default_counter_regroup_tightness")]
    pub counter_regroup_tightness: f32,

    #[serde(default)]
    pub wander_config: WanderConfig,

    #[serde(default)]//if empty, default to 1 inbetween chunk for all other packs
    pub chunk_separation_to_others: HashMap<String, u8>,//u8: inbetween chunks of separation (should work radially)
}

fn default_true() -> bool {
    true
}

fn default_center_rank_weight_multiplier() -> f32 {
    1.0
}

fn default_alert_effectiveness_falloff() -> f32 {
    0.05
}

fn default_counter_regroup_tightness() -> f32 {
    1.5
}

impl PackSeri {
    pub fn tags_with_id(&self) -> TagSet {
        TagSet::new(self.tags.iter().chain(std::iter::once(&self.id)))
    }
}
