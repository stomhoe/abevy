#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::being::body::{
    body_systems::*,
//    body_components::*,
//    body_resources::*,
//    body_constants::*,
//    body_events::*,
//    body_layout::*,
};
pub mod body_components;
mod body_systems;
//mod body_resources;
//mod body_constants;
//mod body_events;
//mod body_layout;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodySystems;
//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Body)) del módulo being !!
pub struct BodyPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem, ).in_set(BodySystems).in_set(IngameSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            //.init_resource::<RESOURCE_NAME>()
            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
        ;
    }
}