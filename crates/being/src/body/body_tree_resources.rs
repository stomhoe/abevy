#[allow(unused_imports)]
use bevy::prelude::*;

use crate::body::body_tree_components::BodyTree;
pub use crate::body::body_tree_seris::*;

common::define_entity_map_systems!(
    BodyTree,
    BodyTreeSeri, "seri.being.body.tree", "body.ron",
);
