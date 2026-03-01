use bevy::prelude::*;

use crate::sex::sex_components::Sex;
pub use crate::sex::sex_seris::*;


common::define_entity_map_systems!(
    Sex,
    SexSeri, "seri.being.sex", "sex.ron",
);
