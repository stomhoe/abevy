use bevy::prelude::*;
use strum_macros::EnumCount;

use crate::{game::{being::{sprite::SpriteSystems, race::{race_components::RaceSeri, race_resources::*, race_systems::*}}, GameDataInitSystems}, AppState};
use bevy_common_assets::ron::RonAssetPlugin;


// Module race
pub mod race_components;
pub mod race_resources;
mod race_systems;
pub mod race_constants;
pub mod race_utils;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RaceSystemsSet;

pub struct RacePlugin;
#[allow(unused_parens)]
impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((RonAssetPlugin::<RaceSeri>::new(&["race.ron"])))
            .add_systems(OnEnter(AppState::StatefulGameSession), (init_races).in_set(GameDataInitSystems).in_set(RaceSystemsSet))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
            .init_resource::<IdRaceEntityMap>()
            .configure_sets(OnEnter(AppState::StatefulGameSession), RaceSystemsSet.after(SpriteSystems))
        ;
    }
}
