use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use common::common_tag_components::TagSet;
use tilemap_shared::tilemap_shared_samplers::NormalDistSeri;

use being_shared::WanderSeri;

pub type SpawnWeight = f32;

pub type RankDist = NormalDistSeri;

#[derive(serde::Deserialize, Asset, TypePath, Debug)]
#[serde(default)]
pub struct PackSeri {
    pub id: String,
    pub tags: HashSet<String>,
    pub spawn_pack_entity: bool,
    pub spawn_being_count_normal_dist: NormalDistSeri,
    pub pack_spawn_radius: u8,
    pub race_ids: HashMap<String, (SpawnWeight, RankDist)>,
    pub bit_ids: HashMap<String, (SpawnWeight, RankDist)>,
    pub center_rank_weight_multiplier: f32,
    pub center_rank_weight_multipliers: HashMap<String, f32>,
    pub biome_affinity: HashMap<String, f32>,
    pub behavior_on_member_attack: String,
    pub attack_alert_effectiveness_falloff: f32,
    pub counter_regroup_tightness: f32,
    pub wander: WanderSeri,
    pub chunk_separation_to_others: HashMap<String, u8>,//u8: inbetween chunks of separation (should work radially)
}

impl Default for PackSeri {
    fn default() -> Self {
        Self {
            id: String::default(),
            tags: HashSet::default(),
            spawn_pack_entity: true,
            spawn_being_count_normal_dist: NormalDistSeri::default(),
            pack_spawn_radius: being_shared::PackSpawnRadius::default().0,
            race_ids: HashMap::default(),
            bit_ids: HashMap::default(),
            center_rank_weight_multiplier: 1.0,
            center_rank_weight_multipliers: HashMap::default(),
            biome_affinity: HashMap::default(),
            behavior_on_member_attack: String::default(),
            attack_alert_effectiveness_falloff: 0.05,
            counter_regroup_tightness: 1.5,
            wander: WanderSeri::default(),
            chunk_separation_to_others: HashMap::default(),
        }
    }
}

impl PackSeri {
    pub fn tags_with_id(&self) -> TagSet {
        TagSet::new(self.tags.iter().chain(std::iter::once(&self.id)))
    }
}
