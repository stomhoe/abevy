
#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::time::time_resources::*;
use crate::game::{ActiveGameSystems, SimRunningSystems};

use crate::game::time::time_systems::*;
use crate::game::time::time_components::*;
//use crate::game::time::time_resources::*;
//use crate::game::time::time_layout::*;
//use crate::game::time::time_events::*;
mod time_systems;
mod time_resources;
mod time_types;
mod time_components;
//mod time_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClockSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Clock)) del módulo parent_module_name !!
pub struct ClockPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (pass_time, reduce_remaining_days, ).in_set(ClockSystems).in_set(SimRunningSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            .init_resource::<CurrYear>()
            .init_resource::<CurrSeason>()
            .init_resource::<CurrDay>()
            .init_resource::<CurrHour>()
            .init_resource::<CurrMin>()
            .init_resource::<SimTimeScale>()
            .init_resource::<InGameTiming>()
        ;
    }
}