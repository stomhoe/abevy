use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::game_common::SimRunningSystems;
use tilemap_shared::CardinalDirection;

use crate::{
    free_movement_systems::*, movement_components::*, movement_modifier_systems::*,
    movement_secondary_systems::*, grid_movement_systems::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, add_being_input_context)
        .add_systems(
            MOVEMENT_SCHEDULE,
            (
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
        .replicate_once::<GridLockedMovement>()
        .replicate_filtered::<CardinalDirection, (Without<MoveVecMag>,)>();
}
