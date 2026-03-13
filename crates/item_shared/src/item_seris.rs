use bevy::platform::collections::{HashMap, HashSet};
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

#[derive(Deserialize, Default, Debug, Clone)]
pub struct SlottedItemHolderSeri {
    #[serde(default)]
    pub slots: HashMap<String, u32>,
}

#[derive(Deserialize, Asset, TypePath, Default, Debug, Clone)]
pub struct ItemsGeneratedOnDeathSeri {
    pub id: String,
    /// Weighted drop entries where each entry is an item-id -> count map
    #[serde(default)]
    pub weights: Vec<(HashMap<String, u32>, f32)>,
    /// Weighted references to other ItemsDroppedOnDeathSeri ids.
    #[serde(default)]
    pub refs: Vec<(String, f32)>,

    #[serde(default = "default_drop_count_multiplier")]
    pub count_multiplier: f32,
}

fn default_drop_count_multiplier() -> f32 { 1.0 }

impl ItemsGeneratedOnDeathSeri {
    pub fn is_sentinel(&self) -> bool {
        self.id.trim().is_empty()
            && self.weights.is_empty()
            && self.refs.is_empty()
            && self.count_multiplier == default_drop_count_multiplier()
    }
}
