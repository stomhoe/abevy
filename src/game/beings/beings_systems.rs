use bevy::prelude::*;

use crate::game::beings::beings_components::{Being, InputMoveDirection};



pub fn handle_movement(
    time: Res<Time>,
    mut query: Query<(&InputMoveDirection, &mut Transform), With<Being>>,
) {
    for (input_move_direction, mut transform) in query.iter_mut() {
        let speed = 1000.0;
        let delta = time.delta_secs();
        let movement = input_move_direction.0 * speed * delta;
        transform.translation += movement;
    }
}


