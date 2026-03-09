use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::{game_common::SimRunningSystems, };
use tilemap_shared::{CardinalDirection, PreChunkDespawnSystems};

use crate::{
    movement_components::*, movement_input_systems::*, movement_messages::*, movement_systems::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;

pub fn plugin(app: &mut App) {
    app
        .add_systems(
            Update,
            add_being_input_context,
        )
        .add_systems(
            MOVEMENT_SCHEDULE,
            (
                send_move_input_to_server.run_if(in_state(ClientState::Connected)),
                receive_move_input_from_client
                    .run_if(in_state(ServerState::Running))
                    .run_if(on_message::<FromClient<SendMoveInput>>),
                apply_remote_move_input_actions.run_if(in_state(ServerState::Running)),
                process_input_direction_modifiers,
                process_speed_modifiers,
                emit_move_state_on_movevecmag_value_change,
                (
                    prepare_grid_locked_movement,
                    do_free_movement,
                ),
                update_facing_dir,
                send_transforms_to_clients.run_if(in_state(ServerState::Running)),
                set_transforms_to_received
                    .after(send_transforms_to_clients)
                    .run_if(on_message::<UnreliableTransform>),
                reconcile_controlled_transforms.after(set_transforms_to_received),
            )
            .in_set(MovementSystems),
        )
        .configure_sets(FixedUpdate, MovementSystems.in_set(SimRunningSystems))
        .configure_sets(Update, MovementSystems.in_set(SimRunningSystems))
        .add_mapped_client_message::<SendMoveInput>(Channel::Ordered)
        .add_mapped_server_message::<UnreliableTransform>(Channel::Unreliable)
        .replicate_once::<GridLockedMovement>()
        .replicate_filtered::<CardinalDirection, (Without<MoveVecMag>,)>()

    ;
}
