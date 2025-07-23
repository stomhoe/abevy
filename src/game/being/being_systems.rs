use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::game::{being::being_components::*, game_components::FacingDirection, multiplayer::{multiplayer_components::MpAuthority, multiplayer_events::TransformFromClient}};



pub fn handle_movement(
    mut cmd: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &InputMoveDirection, &mut Transform), >,
) {
    for (ent, InputMoveDirection(dir), mut transform) in query.iter_mut() {
        let speed = 1000.0;
        let delta = time.delta_secs();
        let movement = dir * speed * delta;

        let prev_translation = transform.translation;

        transform.translation += movement;

        if *dir == Vec3::ZERO {//PROVISORIO
            cmd.entity(ent).remove::<Moving>();
        } else if prev_translation != transform.translation {
            cmd.entity(ent).insert(Moving);
        }


        // commands.client_trigger(
        //     TransformFromClient {
        //         entity: ent,
        //         transf: transform.clone(),
        //         time: std::time::SystemTime::now(),
        //     }
        // );ESTO NO LO DEBE HACER EL SERVER O CRASHEA
    }
}//With<MpAuthority> Option<&MpAuthority>

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn update_direction(
    mut query: Query<(&InputMoveDirection, &mut FacingDirection), >
) {
    for (InputMoveDirection(inp_vec), mut facing_dir) in query.iter_mut() {

        if inp_vec.xy() == Vec2::ZERO {
            continue;
        }
        
        let new_facing_dir = if inp_vec.x.abs() > inp_vec.y.abs() {
            if inp_vec.x < 0.0 {
                FacingDirection::Left
            } else {
                FacingDirection::Right
            }
        } else {
            if inp_vec.y <= 0.0 {
                FacingDirection::Down
            } else {
                FacingDirection::Up
            }
        };
        *facing_dir = new_facing_dir;
    }
}
