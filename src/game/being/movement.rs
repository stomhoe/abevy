#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

#[allow(unused_imports)] use bevy_replicon::prelude::*;


use crate::game::being::being_components::Being;
use crate::game::player::PlayerInputSystems;
use crate::game::ActiveGameSystems;
use crate::game::being::movement::{
   movement_systems::*,
   movement_events::*,
   movement_components::*,
   //movement_resources::*,
};
mod movement_systems;
pub mod movement_components;
pub mod movement_events;
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
            .add_systems(FixedUpdate, (
                (process_movement_modifiers, update_facing_dir, apply_movement, 
                    update_jump_duck_inputs,
                    send_move_input_to_server.run_if(not(server_or_singleplayer)),
                ).in_set   (MovementSystems),
                (human_move_input, ).in_set(PlayerInputSystems)
            ))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            //.init_resource::<RESOURCE_NAME>()
            .configure_sets(Update, 
                (MovementSystems).in_set(ActiveGameSystems)
            )

            .add_client_trigger::<SendMoveInput>(Channel::Unreliable)
            .add_server_trigger::<TransformFromServer>(Channel::Unreliable)
            .add_observer(receive_move_input_from_client)
            .add_observer(on_receive_transf_from_server)

            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
            .replicate::<Altitude>()

            .replicate_with((
                RuleFns::<Being>::default(),
                (RuleFns::<Transform>::default(), SendRate::Periodic((64*3))),
            ))
        ;
    }
}