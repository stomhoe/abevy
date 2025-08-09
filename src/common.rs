use std::time::Duration;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};
use bevy::time::common_conditions::on_timer;
use bevy_replicon::prelude::*;

use crate::common::{
    common_systems::*,
    common_components::*,
//    common_resources::*,
//    common_constants::*,
//    common_events::*,
//    common_layout::*,
};
pub mod common_components;
pub mod common_utils;
mod common_systems;
//mod common_resources;
//mod common_constants;
//mod common_events;
//mod common_layout;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CommonSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Common)) del módulo parent_module_name !!
pub struct CommonPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (set_entity_name,).run_if(on_timer(Duration::from_secs(1))))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            //.init_resource::<RESOURCE_NAME>()
            .add_systems(Startup, spawn_camera)

            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
            .replicate::<DisplayName>()
            .replicate::<HashId>()
            .replicate::<StrId>()
            .replicate::<EntityPrefix>()
            .register_type::<MyZ>()
            .register_type::<DisplayName>()
            .register_type::<StrId>()
            .register_type::<HashId>()

        ;
    }
}