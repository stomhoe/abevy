#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::factions::factions_systems::*;
use crate::game::factions::factions_components::*;
//use crate::game::factions::factions_resources::*;
//use crate::game::factions::factions_layout::*;
//use crate::game::factions::factions_events::*;
mod factions_systems;
pub mod factions_components;
pub mod factions_resources;
//mod factions_layout;
//mod factions_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct FactionsSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Factions)) del módulo parent_module_name !!
pub struct FactionsPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for FactionsPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem, ).in_set(FactionsSystems).in_set(IngameSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.init_resource::<RESOURCE_NAME>()
            .add_plugins((
                superstate_plugin::<NonNeutralState, (AtWar, Truce, Ally)>,
            ),)

        ;
    }
}