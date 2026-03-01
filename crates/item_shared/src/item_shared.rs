use bevy::platform::collections::HashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};
pub use crate::item_seris::*;

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
