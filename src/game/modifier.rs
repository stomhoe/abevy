#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::ActiveGameSystems;
use crate::game::modifier::{
    modi_systems::*,
//    modifier_components::*,
//    modifier_resources::*,
//    modifier_constants::*,
//    modifier_events::*,
//    modifier_layout::*,
};
mod modi_systems;
pub mod modi_components;
pub mod modi_damage_components;
pub mod modi_move_components;
//mod modifier_resources;
//mod modifier_constants;
//mod modifier_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ModifierSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Modifier)) del módulo being !!
pub struct ModifierPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for ModifierPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (apply_antidotes, ).in_set(ModifierSystems).in_set(ActiveGameSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            //.init_resource::<RESOURCE_NAME>()
            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
        ;
    }
}