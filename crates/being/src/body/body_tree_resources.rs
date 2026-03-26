#[allow(unused_imports)]
use bevy::prelude::*;
use game_common::game_common_components::TemplEnti;

use crate::body::body_tree_components::BodyTree;
pub use crate::body::body_tree_seris::*;

common::define_entity_map_systems!(
    BodyTree,
    With<TemplEnti>,
    (BodyTree, TemplEnti),
    BodyTreeSeri, "seri.being.body.tree", "body.ron",
);
