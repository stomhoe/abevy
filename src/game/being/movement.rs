#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::ActiveGameSystems;
use crate::game::being::movement::{
   movement_systems::*,
   movement_components::*,
   //movement_resources::*,
};
mod movement_systems;
pub mod movement_components;
//mod movement_resources;
//mod movement_constants;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Movement)) del módulo being !!
pub struct MovementPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (process_movement_modifiers, update_facing_dir, apply_movement, ).in_set(MovementSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            //.init_resource::<RESOURCE_NAME>()
            .configure_sets(Update, 
                (MovementSystems).in_set(ActiveGameSystems)
            )

            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
        ;
    }
}