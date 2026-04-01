#[allow(unused_imports)]
use bevy::prelude::*;

use ::sprite_shared::*;


common::define_entity_map_systems!(
    SpriteConfig,
    (With<game_common::game_common_components::Templ>, ),
    Sc,
    "sprite_config",
    "",
    SpriteConfig,
    common::common_components::StrId,
    SpriteConfigSeri,
    "seri.sprite.config",
    "sprite.ron",
);
