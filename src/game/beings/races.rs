use bevy::prelude::*;
use strum_macros::EnumCount;

use crate::game::beings::races::races_resources::*;


// Module race
pub mod races_components;
pub mod races_resources;
//mod race_systems;
//mod race_events;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,ACÁ)) del módulo padre !!
pub struct RacesPlugin;
#[allow(unused_parens)]
impl Plugin for RacesPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
            .init_resource::<RaceDatabase>()
        ;
    }
}

#[derive(EnumCount)]
pub enum BaseRacesNids {
    Human = 0,
    Dwarf,
    Elf,
}
