use bevy::{platform::collections::HashMap, prelude::*};
use game_common::game_common_seris::NormalDistSeri;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct PackSeri {
    pub id: String,
    #[serde(default)]
    pub initial_spawn_normal_dist: NormalDistSeri,
    #[serde(default)]
    pub race_ids: Vec<String>,
    #[serde(default)]
    pub bit_ids: Vec<String>,
    #[serde(default)]
    pub biome_affinity: HashMap<String, f32>,
    #[serde(default)]
    pub behavior: String,
}
