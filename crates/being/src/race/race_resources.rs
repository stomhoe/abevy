use bevy::{prelude::*};
use crate::race::Race;
pub use crate::race::race_seris::*;

common::define_entity_map_systems!(
    Race,
    RaceSeri, "seri.being.race", "race.ron",
);
