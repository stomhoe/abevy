use bevy::{prelude::*};
use game_common::game_common_components::TemplEnti;

use crate::pack::pack_components::Pack;
pub use crate::pack::pack_seris::*;

common::define_entity_map_systems!(
    Pack,
    (With<TemplEnti>, ),
    (Pack, TemplEnti),
    PackSeri, "seri.being.pack", "pack.ron",
);
