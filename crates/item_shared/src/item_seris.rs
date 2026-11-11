use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Default, Clone, Debug)]
pub struct ItemSeri {
    pub id: String,
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default, rename = "abstract")]
    pub abstract_item: bool,
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

impl ItemSeri {
    pub fn resolve_inheritance(definitions: Vec<Self>) -> Vec<Self> {
        let mut by_id = bevy::platform::collections::HashMap::with_capacity(definitions.len());
        for definition in definitions {
            by_id.insert(definition.id.clone(), definition);
        }
        let ids: Vec<String> = by_id.keys().cloned().collect();
        ids.into_iter().filter_map(|id| {
            let mut visiting = bevy::platform::collections::HashSet::default();
            let resolved = resolve_item(&id, &by_id, &mut visiting)?;
            (!resolved.abstract_item).then_some(resolved)
        }).collect()
    }
}

fn resolve_item(
    id: &str,
    definitions: &bevy::platform::collections::HashMap<String, ItemSeri>,
    visiting: &mut bevy::platform::collections::HashSet<String>,
) -> Option<ItemSeri> {
    if !visiting.insert(id.to_string()) {
        error!(target: common::log_targets::ITEM_SYSTEM, "Item inheritance cycle detected at '{id}'");
        return None;
    }
    let definition = definitions.get(id)?.clone();
    let Some(parent_id) = definition.extends.as_deref().map(str::trim).filter(|id| !id.is_empty()) else {
        visiting.remove(id);
        return Some(definition);
    };
    let Some(mut parent) = resolve_item(parent_id, definitions, visiting) else {
        error!(target: common::log_targets::ITEM_SYSTEM, "Item '{}' extends missing item '{}'", id, parent_id);
        visiting.remove(id);
        return None;
    };
    merge_item(&mut parent, definition);
    visiting.remove(id);
    Some(parent)
}

fn merge_item(parent: &mut ItemSeri, child: ItemSeri) {
    parent.id = child.id;
    parent.extends = child.extends;
    parent.abstract_item = child.abstract_item;
    if !child.name.is_empty() { parent.name = child.name; }
    if !child.description.is_empty() { parent.description = child.description; }
    if !child.tags.is_empty() { parent.tags.extend(child.tags); }
    if child.mass != 0.0 { parent.mass = child.mass; }
    if child.encumberance != 0.0 { parent.encumberance = child.encumberance; }
    if child.bulk != 0.0 { parent.bulk = child.bulk; }
    if child.durability != 0.0 { parent.durability = child.durability; }
    if child.max_durability != 0.0 { parent.max_durability = child.max_durability; }
    if child.market_value != 0.0 { parent.market_value = child.market_value; }
    if child.warmth != 0.0 { parent.warmth = child.warmth; }
    if child.armor_blunt != 0.0 { parent.armor_blunt = child.armor_blunt; }
    if child.armor_sharp != 0.0 { parent.armor_sharp = child.armor_sharp; }
    if child.armor_fire != 0.0 { parent.armor_fire = child.armor_fire; }
    if child.stack_limit != 1 { parent.stack_limit = child.stack_limit; }
    if !child.equip_sprite_cfg_ids.is_empty() { parent.equip_sprite_cfg_ids = child.equip_sprite_cfg_ids; }
    if !child.dropped_sprite_cfg_id.is_empty() { parent.dropped_sprite_cfg_id = child.dropped_sprite_cfg_id; }
    if !child.icon_sprite_cfg_id.is_empty() { parent.icon_sprite_cfg_id = child.icon_sprite_cfg_id; }
    if !child.icon_img_path.is_empty() { parent.icon_img_path = child.icon_img_path; }
}

fn default_stack_limit() -> u16 { 1 }

#[derive(Deserialize, Default, Debug, Clone)]
pub struct SlottedItemHolderSeri {
    #[serde(default)]
    pub slots: HashMap<String, u32>,
}

#[derive(Deserialize, Asset, TypePath, Default, Debug, Clone)]
pub struct GeneratedItemsSeri {
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

impl GeneratedItemsSeri {
    pub fn is_sentinel(&self) -> bool {
        self.id.trim().is_empty()
            && self.weights.is_empty()
            && self.refs.is_empty()
            && self.count_multiplier == default_drop_count_multiplier()
    }
}
