use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game::movement_components::*;
use game_common::game_common::{GameplaySystems, SimRunningSystems};
use multiplayer_shared::multiplayer_events::{SendMoveInput, TransformFromServer};

use crate::movement_systems::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;


#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(MOVEMENT_SCHEDULE, (
            (process_movement_modifiers, update_facing_dir, apply_movement, update_human_move_input
            ).in_set(MovementSystems),
        ))
        .configure_sets(MOVEMENT_SCHEDULE, 
            (MovementSystems).in_set(SimRunningSystems)
        )

        .add_mapped_client_trigger::<SendMoveInput>(Channel::Unreliable)
        .add_mapped_server_trigger::<TransformFromServer>(Channel::Unreliable)
        .add_observer(receive_move_input_from_client)

        
        .register_type::<InputMoveVector>()
        .register_type::<InputSpeedVector>()
    ;
}