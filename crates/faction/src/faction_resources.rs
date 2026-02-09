use bevy::prelude::*;
use common::{common_components::*, };
use player::player_components::Mine;

use crate::{faction_components::Faction, };

common::define_entity_map_systems!(
    Faction
);
