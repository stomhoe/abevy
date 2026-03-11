use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::{game_common::SimRunningSystems, AcClientSystems};
use tilemap_shared::CardinalDirection;

use crate::{
    movement_components::*, movement_input_systems::*, movement_messages::*, movement_systems::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, add_being_input_context)
        .add_systems(
            MOVEMENT_SCHEDULE,
            (
                apply_grid_move_state_acks
                    .run_if(on_message::<GridMoveStateAck>)
                    .in_set(AcClientSystems),
                server_receive_move_inputs
                    .run_if(in_state(ServerState::Running))
                    .run_if(on_message::<FromClient<SendMoveInput>>),
                process_input_direction_modifiers,
                process_speed_modifiers,
                emit_move_state_on_movevecmag_speed_mag_change,
                start_local_predicted_steps,
                progress_grid_locked_movement,
                do_free_movement,
                update_facing_dir,
            )
                .chain()
                .in_set(MovementSystems),
        )
        .configure_sets(FixedUpdate, MovementSystems.in_set(SimRunningSystems))
        .configure_sets(Update, MovementSystems.in_set(SimRunningSystems))
        .add_mapped_client_message::<SendMoveInput>(Channel::Ordered)
        .add_mapped_server_message::<GridMoveStateAck>(Channel::Ordered)
        .replicate_once::<GridLockedMovement>()
        .replicate_filtered::<CardinalDirection, (Without<MoveVecMag>,)>();
}
