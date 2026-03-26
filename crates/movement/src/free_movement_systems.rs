use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use sprite_animation_shared::{MirrorHolderStateForSprite, MoveAnimActive};

use crate::movement_components::{GridLockedMovement, FinalNormMoveDir, SpeedMagnitude};
use crate::movement_helpers::move_anim_changed;

pub fn do_free_movement(
    mut query: Query<
        (Entity, &mut Transform, &mut MoveAnimActive, &FinalNormMoveDir, &SpeedMagnitude),
        Without<GridLockedMovement>,
    >,
    time: Res<Time>,
    mut writer: MessageWriter<MirrorHolderStateForSprite>,
) {
    let mut move_anim_msgs = HashSet::new();
    for (being_ent, mut transform, mut move_anim, norm_move_dir, speed_magnitude) in query.iter_mut() {
        let velocity = norm_move_dir.0 * speed_magnitude.0;
        if velocity != Vec2::ZERO {
            move_anim_changed(being_ent, &mut move_anim, true, &mut move_anim_msgs);
            transform.translation += (velocity * time.delta_secs()).extend(0.0);
        } else {
            move_anim_changed(being_ent, &mut move_anim, false, &mut move_anim_msgs);
        }
    }
    writer.write_batch(move_anim_msgs);
}
