#[allow(unused_imports)]
use bevy::prelude::*;
use game_common::game_common_components::Templ;

use crate::body::body_components::Body;
pub use crate::body::body_seris::*;

common::define_entity_map_systems!(
    Body,
    With<Templ>,
    (Body, Templ),
    BodySeri, "seri.being.body", "body.ron",
);
