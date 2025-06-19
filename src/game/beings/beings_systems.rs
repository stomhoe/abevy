use bevy::prelude::*;

use crate::game::beings::beings_components::{Being, InputMoveDirection};



pub fn handle_movement(
    time: Res<Time>,
    mut move_input_dir: Single<&mut InputMoveDirection, (With<Being>)>,
) {
    let mut input_dir = Vec3::ZERO;

    // TODO extraer velocidad en el tipo de piso
    
    input_dir.xy()
}










