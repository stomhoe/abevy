use bevy::{platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FacRelaConfigSeri {
    #[serde(default)]
    pub base_opinion: f32,
    #[serde(default)]
    pub hostility_bias: f32,
    #[serde(default = "default_trust")]
    pub trust: f32,
    #[serde(default)]
    pub aid_chance: f32,
}
impl Default for FacRelaConfigSeri {
    fn default() -> Self {
        Self {
            base_opinion: 0.0,
            hostility_bias: 0.0,
            trust: default_trust(),
            aid_chance: 0.0,
        }
    }
}
fn default_trust() -> f32 { 0.5 }

#[derive(Deserialize, Serialize, Asset, TypePath, Default, Debug)]
pub struct FactionInstTemplSeri {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub default_relationships_by_tag: HashMap<String, FacRelaConfigSeri>,
    #[serde(default)]
    pub culture_id: String,
    #[serde(default = "default_true")]
    pub player_joinable: bool,
    #[serde(default)]
    pub bit_weightmap: HashMap<String, f32>,
    #[serde(default)]
    pub starting_wealth: i64,
    #[serde(default)]
    pub lawfulness: f32,
    #[serde(default)]
    pub aggression: f32,
    #[serde(default)]
    pub isolationism: f32,
    #[serde(default)]
    pub expansionism: f32,
    #[serde(default)]
    pub max_members: Option<u32>,
}

fn default_true() -> bool { true }
