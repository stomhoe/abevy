use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::game_common::{GameplaySystems, SimRunningSystems};

use crate::{
    movement_components::*, movement_input_systems::*, movement_messages::*, movement_systems::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;

pub fn plugin(app: &mut App) {
    app
        // Input capture in Update schedule (high frequency)
        .add_systems(
            Update,
            update_human_move_input,
        )
        // Movement processing in FixedUpdate schedule
        .add_systems(
            MOVEMENT_SCHEDULE,
            (
                send_move_input_to_server.run_if(in_state(ClientState::Connected)),
                receive_move_input_from_client.run_if(in_state(ServerState::Running)),
                process_input_direction_modifiers,
                process_speed_modifiers,
                (
                    prepare_grid_locked_movement,
                    do_free_movement,
                ),
                update_facing_dir,
                sync_movement_to_server.run_if(in_state(ServerState::Running)),
            )
                .in_set(MovementSystems),
        )
        .configure_sets(FixedUpdate, MovementSystems.in_set(SimRunningSystems))
        .add_mapped_client_message::<SendMoveInput>(Channel::Unreliable)
        .add_mapped_server_message::<TransformFromServer>(Channel::Unreliable)
        .register_type::<InputDirection>()
        .register_type::<MoveState>()
        .register_type::<GridLockedMovement>()
        .replicate::<WallPhaser>()
        .replicate::<LandWalker>()
        .replicate::<Swimmer>()
        .replicate::<Flier>()
    ;
}
