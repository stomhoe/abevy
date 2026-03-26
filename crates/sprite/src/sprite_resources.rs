#[allow(unused_imports)]
use bevy::prelude::*;

pub use sprite_shared::sprite_resources::*;

use crate::sprite_components::SpriteConfig;

common::define_entity_map_systems!(
    SpriteConfig,
    (With<game_common::game_common_components::TemplEnti>, ),
    Sc,
    "sprite_config",
    "",
    SpriteConfig,
    common::common_components::StrId,
    SpriteConfigSeri,
    "seri.sprite.config",
    "sprite.ron",
);
