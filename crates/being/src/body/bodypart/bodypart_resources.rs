use bevy::prelude::*;
use game_common::game_common_components::Templ;
pub use crate::body::bodypart::bodypart_seris::*;
use ::being_shared::*;

common::define_entity_map_systems!(
    Bodypart,
    (With<Templ>, Without<BodypartChildOfBodypart>, Without<BodypartChildrenBodyparts>),
    (Bodypart, Templ),
    BodypartSeri, "seri.being.body.part", "bodypart.ron",
);
