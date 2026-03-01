use bevy::platform::collections::HashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(Replicated, Prefix::trunc("Item"), AssetScoped, SparedFromHotReloading)]
pub struct Item {
    pub equip_sprite_cfg_ids: Vec<StrId>,
    pub dropped_sprite_cfg_id: StrId,
    pub icon_sprite_cfg_id: StrId,
    pub icon_img_path: String,
}
impl Item {
    pub const MIN_ID_LENGTH: u8 = 2;
}

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

fn default_stack_limit() -> u16 {
    1
}

common::define_entity_map_systems!(
    Item,
    (With<game_common::game_common_components::EntityZero>, common::AnyDisabling),
    (Item, game_common::game_common_components::EntityZero),
    ItemSeri,
    "seri.item",
    "item.ron",
);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct ItemTags(pub TagSet);
