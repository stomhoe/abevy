#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::IngameSystems;
use crate::game::being::modifier::{
    modifier_systems::*,
//    modifier_components::*,
//    modifier_resources::*,
//    modifier_constants::*,
//    modifier_events::*,
//    modifier_layout::*,
};
mod modifier_systems;
pub mod modifier_components;
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
            .add_systems(Update, (update_current_potency, ).in_set(ModifierSystems).in_set(IngameSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            //.init_resource::<RESOURCE_NAME>()
            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
        ;
    }
}