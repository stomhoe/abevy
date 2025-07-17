use bevy::prelude::*;
use strum_macros::EnumCount;

use crate::{game::{beings::races::{races_resources::*, races_systems::*}, GameDataInitSystems}, AppState};


// Module race
pub mod races_components;
pub mod races_resources;
mod races_systems;
//mod race_events;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,ACÁ)) del módulo padre !!
pub struct RacesPlugin;
#[allow(unused_parens)]
impl Plugin for RacesPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem))
            .add_systems(OnEnter(AppState::StatefulGameSession), (init_races).in_set(GameDataInitSystems))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
            .init_resource::<RaceEntityMap>()
        ;
    }
}

#[derive(EnumCount)]
pub enum BaseRacesNids {
    Human = 0,
    Dwarf,
    Elf,
}
