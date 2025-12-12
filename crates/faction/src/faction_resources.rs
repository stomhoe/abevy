use bevy::prelude::*;
use common::{common_components::{DisplayName, EntityPrefix, StrId}, common_types::HashIdToEntityMap};
use player::player_components::OfSelf;

use crate::{faction_components::Faction, };

#[derive(Resource, Reflect, Default)]
pub struct FactionEntityMap (pub HashIdToEntityMap);