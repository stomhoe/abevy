use crate::body::BodyPart;
use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)]
use bevy::prelude::*;
use game_common::game_common_components::EntityZero;
pub use crate::body::body_part::body_part_seris::*;

common::define_entity_map_systems!(
    BodyPart,
    With<EntityZero>,
    BodyPartSeri, "seri.being.body.part", "bodypart.ron",
);
