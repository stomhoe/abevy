use bevy::{
    prelude::*,
};
use faction_shared::Culture;
use crate::culture::culture_seris::*;

common::define_entity_map_systems!(
    Culture,
    CultureSeri, "seri.faction.culture", "culture.ron",
);
