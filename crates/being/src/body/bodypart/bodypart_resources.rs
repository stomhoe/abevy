use bevy::prelude::*;
use game_common::game_common_components::EntityZero;
pub use crate::body::bodypart::bodypart_seris::*;
use ::being_shared::*;

common::define_entity_map_systems!(
    Bodypart,
    (With<EntityZero>, Without<BodypartChildOfBodypart>, Without<BodypartChildrenBodyparts>),
    (Bodypart, EntityZero),
    BodypartSeri, "seri.being.body.part", "bodypart.ron",
);
