use bevy::{platform::collections::HashMap, prelude::*};
use game_common::game_common_seris::NormalDistSeri;

pub type SpawnWeight = f32;

pub type RankDist = NormalDistSeri;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct PackSeri {
    pub id: String,
    #[serde(default = "NormalDistSeri::sentinel")]
    pub spawn_being_count_normal_dist: NormalDistSeri,
    #[serde(default)]
    pub race_ids: HashMap<String, (SpawnWeight, RankDist)>,
    #[serde(default)]
    pub bit_ids: HashMap<String, (SpawnWeight, RankDist)>,
    #[serde(default)]
    pub biome_affinity: HashMap<String, f32>,
    #[serde(default)]//placeholder, dont use this
    pub behavior_on_member_attack: String,

    #[serde(default)]//if empty, default to 1 inbetween chunk for all other packs
    pub chunk_separation_to_others: HashMap<String, u8>,//u8: inbetween chunks of separation (should work radially)
}
