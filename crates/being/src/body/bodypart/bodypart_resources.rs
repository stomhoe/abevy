use bevy::prelude::*;
use game_common::game_common_components::TemplEnti;
pub use crate::body::bodypart::bodypart_seris::*;
use ::being_shared::*;

common::define_entity_map_systems!(
    Bodypart,
    (With<TemplEnti>, Without<BodypartChildOfBodypart>, Without<BodypartChildrenBodyparts>),
    (Bodypart, TemplEnti),
    BodypartSeri, "seri.being.body.part", "bodypart.ron",
);
