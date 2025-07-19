use bevy::prelude::*;
use strum_macros::EnumCount;

use crate::{game::{beings::{animation::AnimationSystems, races::{races_components::RaceDto, races_resources::*, races_systems::*}}, GameDataInitSystems}, AppState};
use bevy_common_assets::ron::RonAssetPlugin;


// Module race
pub mod races_components;
pub mod races_resources;
mod races_systems;
pub mod race_constants;
//mod race_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RaceSystems;

pub struct RacesPlugin;
#[allow(unused_parens)]
impl Plugin for RacesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((RonAssetPlugin::<RaceDto>::new(&["race.ron"])))
            .add_systems(OnEnter(AppState::StatefulGameSession), (init_races).in_set(GameDataInitSystems).in_set(RaceSystems))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
            .init_resource::<IdRaceEntityMap>()
            .configure_sets(OnEnter(AppState::StatefulGameSession), RaceSystems.after(AnimationSystems))
        ;
    }
}

#[derive(EnumCount)]
pub enum BaseRacesNids {
    Human = 0,
    Dwarf,
    Elf,
}
