use bevy::{platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Asset, TypePath, Default, Debug)]
pub struct CultureSeri {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub bit_weightmap: HashMap<String, f32>,
    #[serde(default)]
    pub races_opinion: HashMap<String, f32>,
    #[serde(default)]
    pub technology_level: f32,
    #[serde(default)]
    pub militarism: f32,
    #[serde(default)]
    pub spirituality: f32,
    #[serde(default)]
    pub trade_focus: f32,
}
