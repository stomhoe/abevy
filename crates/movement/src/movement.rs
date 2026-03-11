use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::game_common::SimRunningSystems;
use tilemap_shared::CardinalDirection;

use crate::{
    free_movement_systems::*, movement_components::*, movement_modifier_systems::*,
    movement_secondary_systems::*, grid_movement_systems::*, movement_input_systems::*,
    movement_messages::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;

pub fn plugin(app: &mut App) {
    app.init_resource::<MovementSimTick>()
        .init_resource::<LastKnownServerMovementTick>()
        .add_systems(
        Update,
        (
            add_movement_components_to_beings,
            add_being_input_context,
        )
            .in_set(MovementSystems),
    )
        .add_systems(
            FixedUpdate,
            (
                receive_gpos_from_server
                    .run_if(in_state(ClientState::Connected))
                    .run_if(on_message::<SyncGpos>),
                receive_transform_from_server
                    .run_if(in_state(ClientState::Connected))
                    .run_if(on_message::<SyncTransform>),
                receive_movement_tick_from_server
                    .run_if(in_state(ClientState::Connected))
                    .run_if(on_message::<SyncMovementTick>),
            )
                .in_set(MovementSystems),
        )

        .add_systems(
            MOVEMENT_SCHEDULE,
            (
                tick_movement_sim,
                send_movement_tick_to_clients
                    .run_if(in_state(ServerState::Running)),
                (
                    copy_player_move_input_to_beings,
                    send_move_input_to_server
                        .run_if(in_state(ClientState::Connected)),
                    receive_move_input_from_client
                        .run_if(in_state(ServerState::Running))
                        .run_if(on_message::<FromClient<SendMoveInput>>),
                    apply_buffered_move_input
                        .run_if(in_state(ServerState::Running)),
                ).chain(),
                process_input_direction_modifiers,
                process_speed_modifiers,
                emit_move_state_on_movevecmag_speed_mag_change,
                start_grid_locked_steps,
                progress_tile_transition_transform,
                do_free_movement,
                update_facing_dir,
            )
                .chain()
                .in_set(MovementSystems),
        )
        .configure_sets(FixedUpdate, MovementSystems.in_set(SimRunningSystems))
        .configure_sets(Update, MovementSystems.in_set(SimRunningSystems))
        .add_mapped_client_message::<SendMoveInput>(Channel::Ordered)
        .add_mapped_server_message::<SyncGpos>(Channel::Ordered)
        .add_mapped_server_message::<SyncMovementTick>(Channel::Ordered)
        .add_mapped_server_message::<SyncTransform>(Channel::Unreliable)
        .replicate_once::<GridLockedMovement>()
        .replicate_filtered::<CardinalDirection, (Without<MoveVecMag>,)>();
}
