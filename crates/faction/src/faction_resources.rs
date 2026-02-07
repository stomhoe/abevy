use bevy::prelude::*;
use common::{common_components::{DisplayName, Prefix, StrId}, common_types::HashIdToEntityMap};
use player::player_components::Mine;

use crate::{faction_components::Faction, };

common::define_entity_map_systems!(
    Faction
);
