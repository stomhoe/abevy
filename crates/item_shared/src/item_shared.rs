#[allow(unused_imports)] use bevy::prelude::*;
pub use crate::item_seris::*;
pub use crate::item_components::*;


common::define_entity_map_systems!(
    main_component: Item,
    with_filters: (With<game_common::game_common_components::Templ>, common::AnyDisabling),
    abbreviation: Item,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: (Item, game_common::game_common_components::Templ),
    id_type: common::common_components::StrId,
    assets: [(ItemSeri, "seri.item", "item.ron")],
);
