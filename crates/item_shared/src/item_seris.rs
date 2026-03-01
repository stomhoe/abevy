use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Default)]
pub struct ItemSeri {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: HashSet<String>,
    #[serde(default)]
    pub mass: f32,
    #[serde(default)]
    pub encumberance: f32,
    #[serde(default)]
    pub bulk: f32,
    #[serde(default)]
    pub durability: f32,
    #[serde(default)]
    pub max_durability: f32,
    #[serde(default)]
    pub market_value: f32,
    #[serde(default)]
    pub warmth: f32,
    #[serde(default)]
    pub armor_blunt: f32,
    #[serde(default)]
    pub armor_sharp: f32,
    #[serde(default)]
    pub armor_fire: f32,
    #[serde(default = "default_stack_limit")]
    pub stack_limit: u16,
    #[serde(default)]
    pub equip_sprite_cfg_ids: Vec<String>,
    #[serde(default)]
    pub dropped_sprite_cfg_id: String,
    #[serde(default)]
    pub icon_sprite_cfg_id: String,
    #[serde(default)]
    pub icon_img_path: String,
}

fn default_stack_limit() -> u16 { 1 }
