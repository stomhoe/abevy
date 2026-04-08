#[allow(unused_imports)]
use bevy::prelude::*;

use ::sprite_shared::*;


common::define_entity_map_systems!(
    main_component: SpriteConfig,
    with_filters: (With<game_common::game_common_components::Templ>, ),
    abbreviation: Sc,
    target: "sprite_config",
    entity_prefix: "",
    despawn_trigger: SpriteConfig,
    id_type: common::common_components::StrId,
    assets: [(SpriteConfigSeri, "seri.sprite.config", "sprite.ron")],
);
