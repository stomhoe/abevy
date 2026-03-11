use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use sprite_animation_shared::{BeingChangedMoveState, MoveAnimActive};

use crate::movement_components::{GridLockedMovement, MoveVecMag};
use crate::movement_helpers::move_anim_changed;

pub fn do_free_movement(
    mut query: Query<
        (Entity, &mut Transform, &mut MoveAnimActive, &MoveVecMag),
        Without<GridLockedMovement>,
    >,
    time: Res<Time>,
    mut writer: MessageWriter<BeingChangedMoveState>,
) {
    let mut move_anim_msgs = HashSet::new();
    for (being_ent, mut transform, mut move_anim, move_state) in query.iter_mut() {
        let velocity = move_state.norm_move_dir * move_state.speed_magnitude;
        if velocity != Vec2::ZERO {
            move_anim_changed(being_ent, &mut move_anim, true, &mut move_anim_msgs);
            transform.translation += (velocity * time.delta_secs()).extend(0.0);
        } else {
            move_anim_changed(being_ent, &mut move_anim, false, &mut move_anim_msgs);
        }
    }
    writer.write_batch(move_anim_msgs);
}
