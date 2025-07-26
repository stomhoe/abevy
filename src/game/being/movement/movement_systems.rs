#[allow(unused_imports)] use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::Target;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::{being::{being_components::{Being, }, modifier::modifier_components::*, movement::movement_components::*}, game_components::FacingDirection};

pub fn apply_movement(
    mut cmd: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &FinalMoveVector, &mut Transform), /*(With<VoluntarilyMoving>)*/>,
) {
    for (ent, FinalMoveVector(move_dir), mut transform) in query.iter_mut() {
        let speed = 1000.0;
        let delta = time.delta_secs();
        let movement = move_dir * speed * delta;

        let prev_translation = transform.translation;

        transform.translation += movement;

        if *move_dir == Vec3::ZERO {//PROVISORIO
            cmd.entity(ent).remove::<VoluntarilyMoving>();
        } else if prev_translation != transform.translation {
            cmd.entity(ent).insert(VoluntarilyMoving);
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

#[allow(unused_parens)]
pub fn update_facing_dir(mut query: Query<(&FinalMoveVector, &mut FacingDirection), >) {
    for (FinalMoveVector(dir_vec), mut facing_dir) in query.iter_mut() {
        if dir_vec.xy() == Vec2::ZERO {continue;}
        
        *facing_dir = if dir_vec.x.abs() > dir_vec.y.abs() {
            if dir_vec.x < 0.0 {FacingDirection::Left} else {FacingDirection::Right}
        } else {
            if dir_vec.y <= 0.0 {FacingDirection::Down} else {FacingDirection::Up}
        };
    }
}

//PARA HACER ANTÍDOTOS Q ATACAN SUSTANCIAS ESPECÍFICAS, HACER OTRO SISTEMA Q AFECTE EL POWER DE OTROS EFECTOS

#[allow(unused_parens)]
pub fn process_movement_modifiers(mut cmd: Commands, 
    mut being_query: Query<(Entity, &AppliedModifiers, &InputMoveVector, &mut FinalMoveVector), (With<Being>)>,
    mut effects_query: Query<(
        &Modifier,
        &CurrentPotency,
        Option<&MitigatingOnly>,
        Option<&MultiplyingModifier>,
        Option<&Speed>, 
        Option<&InvertMovement>, 
    )>, 

){

    for (ent, modifiers, InputMoveVector(inp_vec), mut final_vec) in being_query.iter_mut() {

        final_vec.0 = *inp_vec;

        let mut changes_position = false;
        let mut invert_movement: f32 = 0.0;

        for effect in modifiers.entities().iter() {
            if let Ok((_modifier, _potency, mitigating_only, multiplying_modifier, speed, invert_move)) = effects_query.get(*effect) {

                match (invert_move, mitigating_only) {
                    (Some(_), None) => {
                        invert_movement += 1.0;
                    },
                    (None, Some(_)) => {
                        invert_movement -= 1.0;
                    },
                    (Some(_), Some(_)) => {},
                    (None, None) => {}
                }
            }
        }

        if changes_position {
            cmd.entity(ent).insert(VoluntarilyMoving);
        }

        if invert_movement > 0.0 {
            final_vec.0 = -final_vec.0;
        }
        
    }
}


