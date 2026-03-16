use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::game_common::SimRunningSystems;
use tilemap_shared::CardinalDirection;

use crate::{
    free_movement_systems::*, movement_components::*, movement_modifier_systems::*,
    movement_secondary_systems::*, grid_movement_systems::*, movement_host_systems::*,
    movement_messages::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            add_movement_components_to_beings,
            copy_client_move_input_to_controlled_beings,
            receive_gpos_from_server
                .run_if(in_state(ClientState::Connected))
                .run_if(on_message::<SyncGpos>),
        )
            .in_set(MovementSystems),
    )
        .add_systems(
            MOVEMENT_SCHEDULE,
            (
                (
                    receive_step_request_from_client
                        .run_if(in_state(ServerState::Running))
                        .run_if(on_message::<FromClient<SendStepRequest>>),
                ),
                apply_pending_tile_corrections
                    .run_if(in_state(ClientState::Connected)),
                process_input_direction_modifiers,
                process_speed_modifiers,
                emit_move_state_on_movevecmag_speed_mag_change,
                start_grid_locked_steps,
                progress_tile_transition_transform,
                update_facing_dir,
                do_free_movement,
            )
                .in_set(MovementSystems),
        )
        .configure_sets(FixedUpdate, MovementSystems.in_set(SimRunningSystems))
        .configure_sets(Update, MovementSystems.in_set(SimRunningSystems))
        .add_mapped_client_message::<SendStepRequest>(Channel::Ordered)
        .add_mapped_server_message::<SyncGpos>(Channel::Ordered)
        .replicate_once::<GridLockedMovement>()
        .replicate::<SpeedMagnitude>()
        .replicate_filtered::<CardinalDirection, (Without<FinalNormMoveDir>,)>();
}
