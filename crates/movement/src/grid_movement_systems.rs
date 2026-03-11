use being_shared::{ComputedLocally, ControlledBy};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use ac_input::ac_input_actions::*;
use param_sets::BlockingTileParamSet;
use sprite_animation_shared::{BeingChangedMoveState, MoveAnimActive};
use tilemap_shared::*;

use crate::movement_components::*;
use crate::movement_helpers::*;

pub fn start_grid_locked_steps(
    time: Res<Time>,
    move_actions: Query<(&Action<BeingMoveAction>, &ActionOf<BeingInputContext>)>,
    mut param_set: ParamSet<(
        BlockingTileParamSet,
        Query<(
            Entity,
            &ControlledBy,
            Has<ComputedLocally>,
            &DimensionRef,
            &MoveVecMag,
            &mut GlobalTilePos,
            &mut GridLockedMovement,
            &mut CardinalDirection,
        )>,
    )>,
    mut to_drain: Local<Vec<Entity>>,
) {
    for (move_action, action_of) in move_actions.iter() {
        let being_ent = **action_of;
        let (
            entity,
            controlled_locally,
            human_input,
            dim_ref,
            move_speed,
            mut tile_pos_snapshot,
            mut glm_snapshot,
        ) = {
            let mut beings = param_set.p1();
            let Ok((
                entity,
                controlled_by,
                controlled_locally,
                &dim_ref,
                move_state,
                tile_pos,
                mut glm,
                _facing_dir,
            )) = beings.get_mut(being_ent)
            else {
                continue;
            };
            glm.ensure_grid_anchor(*tile_pos);
            (
                entity,
                controlled_locally,
                controlled_by.human_input,
                dim_ref,
                move_state.speed_magnitude,
                *tile_pos,
                *glm,
            )
        };
        if !controlled_locally || !human_input {
            continue;
        }
        let dir = normalize_to_axis_dir(**move_action);
        if !glm_snapshot.try_start_step(
            &param_set.p0(),
            &mut to_drain,
            dim_ref,
            entity,
            &mut tile_pos_snapshot,
            dir,
            step_duration_secs(move_speed, dir),
            ticks_per_tile(move_speed, time.delta_secs(), dir),
        ) {
            continue;
        }
        let mut beings = param_set.p1();
        let Ok((_, _, _, _, _, mut tile_pos, mut glm, mut facing_dir)) = beings.get_mut(entity)
        else {
            continue;
        };
        *tile_pos = tile_pos_snapshot;
        *glm = glm_snapshot;
        *facing_dir = CardinalDirection::from_dir_vec(dir);
    }
}

pub fn progress_tile_transition_transform(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &GlobalTilePos,
        &mut Transform,
        &mut MoveAnimActive,
        &mut GridLockedMovement,
    )>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut messages: Local<HashSet<BeingChangedMoveState>>,
) {
    for (being_ent, tile_pos, mut transform, mut move_anim, mut glm) in query.iter_mut() {
        glm.ensure_grid_anchor(*tile_pos);
        glm.progress_grid_step(*tile_pos, time.delta_secs());
        transform.translation = glm.grid_translation(*tile_pos, transform.translation.z);
        move_anim_changed(being_ent, &mut move_anim, glm.is_stepping(), &mut messages);
    }
    writer.write_batch(messages.drain());
}
