use bevy::prelude::*;
use game_common::HostSystems;

use crate::terrgen_natural_spawning::*;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(Update, spawn_natural_wildlife_for_chunk.in_set(HostSystems))
    ;
}
