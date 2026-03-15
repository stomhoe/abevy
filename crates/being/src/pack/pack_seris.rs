use bevy::{platform::collections::HashMap, prelude::*};
use game_common::game_common_seris::NormalDistSeri;

pub type SpawnWeight = f32;
pub type LeaderPriority = f32;


#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct PackSeri {
    pub id: String,
    #[serde(default)]
    pub initial_spawn_normal_dist: NormalDistSeri,
    #[serde(default)]
    pub race_ids: HashMap<String, (SpawnWeight, LeaderPriority)>,
    #[serde(default)]
    pub bit_ids: HashMap<String, (SpawnWeight, LeaderPriority)>,
    #[serde(default)]
    pub biome_affinity: HashMap<String, f32>,
    #[serde(default)]//placeholder, dont use this
    pub behavior: String,
}
