#[allow(unused_imports)] use bevy::prelude::*;
pub use crate::item_seris::*;
pub use crate::item_components::*;


common::define_entity_map_systems!(
    Item,
    (With<game_common::game_common_components::EntityZero>, common::AnyDisabling),
    (Item, game_common::game_common_components::EntityZero),
    ItemSeri,
    "seri.item",
    "item.ron",
);
