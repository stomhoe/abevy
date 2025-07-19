#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{common::common_components::DisplayName, game::beings::races::{race_constants::*, races_components::{Race, RaceDto}, races_resources::IdRaceEntityMap}};

pub fn init_races(
    mut cmd: Commands,
    aserver: Res<AssetServer>,
    mut race_dtos: ResMut<Assets<RaceDto>>,

    mut map: ResMut<IdRaceEntityMap>, 
) {

    let human_handle: Handle<RaceDto> = load_race(&aserver, "human");

    map.new_race_from_dto(&mut cmd, human_handle, &race_dtos);
}



pub fn load_race(
    aserver: &AssetServer,
    path: impl AsRef<str>,
) -> Handle<RaceDto> {
    aserver.load(format!("{}{}{}", RACES_DIR, path.as_ref(), RACES_EXTENSION))
}
