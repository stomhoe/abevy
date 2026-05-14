use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::game_common::SimRunningSystems;
use tilemap_shared::CardinalDirection;

use crate::{
    free_movement_systems::*, movement_modifier_systems::*,
    movement_secondary_systems::*, grid_movement_systems::*, movement_host_systems::*,
    movement_messages::*,
};
use game_common::HostSystems;
use being_shared::movement_shared_components::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;

pub fn plugin(app: &mut App) {
    app.init_resource::<MpSettings>()
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), load_mp_settings)
        .add_systems(
            Update,
            (
                add_grid_locked_movement_requirements,
                add_movement_components_to_beings,
                copy_client_move_input_to_controlled_beings,
                apply_input_vec_modi_mul_to_final_norm_move_dir
                    .after(copy_client_move_input_to_controlled_beings)
                    .after(process_input_direction_modifiers),
                receive_gpos_from_server
                    .run_if(in_state(ClientState::Connected))
                    .run_if(on_message::<SyncGpos>),
            ).in_set(MovementSystems),
        )
        .add_systems(
            MOVEMENT_SCHEDULE,
            (
                receive_step_request_from_client
                    .run_if(in_state(ServerState::Running))
                    .after(progress_tile_transition_transform),
                apply_pending_tile_corrections
                    .run_if(in_state(ClientState::Connected)),
                sync_occupancy_for_beings_at_gpos_res,
            )
            .in_set(MovementSystems),
        )
        .add_systems(
            Update,
            (
                resolve_overlapping_beings,
                process_input_direction_modifiers,
                process_speed_potential_modifiers,
                process_speed_magnitude
                    .after(process_speed_potential_modifiers),
            ).run_if(on_timer(Duration::from_secs_f32(0.2)))
            .in_set(HostSystems)
            .in_set(MovementSystems),
        )
        .add_systems(
            MOVEMENT_SCHEDULE,
            (
                emit_move_state_on_movevecmag_speed_mag_change,
                start_grid_locked_steps,
                progress_tile_transition_transform,
            )
            .in_set(MovementSystems),
        )
        .add_systems(
            MOVEMENT_SCHEDULE,
            (
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
        .replicate_once::<GridLockedMovementVisual>()
        .replicate::<SpeedMagnitude>()
        .replicate::<InputInvMul>()
        .replicate_filtered::<CardinalDirection, (Without<FinalNormMoveDir>, Without<SpeedMagnitude>)>();
}
