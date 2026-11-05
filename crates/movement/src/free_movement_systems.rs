use bevy::prelude::*;
use game_common::Dead;
use sprite_animation_shared::MoveAnimActive;

use being_shared::movement_shared_components::{GridLockedMovement, FinalNormMoveDir, SpeedMagnitude};

pub fn do_free_movement(
    mut query: Query<
        (Entity, &mut Transform, &mut MoveAnimActive, &FinalNormMoveDir, &SpeedMagnitude, Has<Dead>),
        Without<GridLockedMovement>,
    >,
    time: Res<Time>,
) {
    for (being_ent, mut transform, mut move_anim, norm_move_dir, speed_magnitude, dead) in query.iter_mut() {
        if dead { move_anim.set(false); continue; }
        let velocity = norm_move_dir.0 * speed_magnitude.0;
        if velocity != Vec2::ZERO {
            let _ = being_ent;
            move_anim.set(true);
            transform.translation += (velocity * time.delta_secs()).extend(0.0);
        } else {
            let _ = being_ent;
            move_anim.set(false);
        }
    }
}
