use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Debug, Clone)]
pub struct TileStepSfxSeri {
    #[serde(default)]
    pub groups: Vec<(f32, Vec<String>)>,
    #[serde(default)]
    pub directory: String,
    #[serde(default = "default_weight")]
    pub directory_weight: f32,
    #[serde(default = "default_true")]
    pub prevent_repeat: bool,
}
impl Default for TileStepSfxSeri {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            directory: String::new(),
            directory_weight: default_weight(),
            prevent_repeat: default_true(),
        }
    }
}

#[derive(Deserialize, Asset, TypePath, )]
pub struct DungeonSeri {
    pub id: String,
    pub name: String,
    pub description: String,
}
fn default_weight() -> f32 { 1.0 }
fn default_true() -> bool { true }
