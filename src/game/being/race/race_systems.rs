use bevy::log::tracing::span::Id;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::being::{race::{
   race_components::*, race_constants::*, race_resources::*
   //race_events::*,
}, sprite::animation_resources::IdSpriteDataEntityMap};

pub fn init_races(
    mut cmd: Commands,
    aserver: Res<AssetServer>,
    mut race_seris: ResMut<Assets<RaceSeri>>,

    mut race_map: ResMut<RaceIdEntityMap>, 
    sprite_map: Res<IdSpriteDataEntityMap>, 
) {

    let human_handle: Handle<RaceSeri> = load_race(&aserver, "human");

    race_map.new_race_from_seri(&mut cmd, human_handle, &mut race_seris, &sprite_map);
}



pub fn load_race(
    aserver: &AssetServer,
    path: impl AsRef<str>,
) -> Handle<RaceSeri> {
    aserver.load(format!("{}{}{}", RACES_DIR, path.as_ref(), RACES_EXTENSION))
}

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
