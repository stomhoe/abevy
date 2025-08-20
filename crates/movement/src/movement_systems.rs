use core::f32;

use being_shared::{ControlledBy, ControlledLocally, HumanControlled};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon_renet::renet::{RenetClient, RenetServer};
use common::common_states::GameSetupType;
use game_common::game_common_components::FacingDirection;
use modifier::{modifier_components::*, modifier_move_components::*};
use player::{player_components::*, player_resources::KeyboardInputMappings};
use sprite_animation_shared::MoveAnimActive;

use crate::{movement_components::*, movement_events::{SendMoveInput, TransformFromServer}};

#[allow(unused_parens, )]
pub fn update_human_move_input(
    keys: Res<ButtonInput<KeyCode>>,
    input_mappings: Res<KeyboardInputMappings>,
    mut move_input: Query<(&mut InputMoveVector, &HumanControlled), (With<ControlledLocally>, )>,
) {
    let mut input_dir = Vec2::ZERO;
    if keys.pressed(input_mappings.move_up) {input_dir.y += 1.0;}
    if keys.pressed(input_mappings.move_down) {input_dir.y -= 1.0;}
    if keys.pressed(input_mappings.move_left) {input_dir.x -= 1.0;}
    if keys.pressed(input_mappings.move_right) {input_dir.x += 1.0;}
    
    if input_dir != Vec2::ZERO {input_dir = input_dir.normalize();}
    
    for (mut move_input_dir, human_controlled) in move_input.iter_mut() {
        if human_controlled.0 && move_input_dir.0 != input_dir { 
            trace!(target: "movement", "Updating human move input");
            move_input_dir.0 = input_dir; 
        }
    }
}

#[allow(unused_parens)]
// pub fn update_jump_duck_inputs(
//     mut cmd: Commands, 
//     query: Query<(Entity, &HumanControlled),(With<ControlledLocally>, )>,
//     keys: Res<ButtonInput<KeyCode>>,
//     input_mappings: Res<KeyboardInputMappings>,) 


// {
//     if keys.pressed(input_mappings.jump_or_fly) {//TODO USAR BEVY ENHANCED INPUT
//         for ent in query.iter() {cmd.entity(ent).insert(InputJump);}
//     } else {
//         for ent in query.iter() {cmd.entity(ent).remove::<InputJump>();}
//     }
//     if keys.pressed(input_mappings.duck) {
//         for ent in query.iter() {cmd.entity(ent).insert(InputDuck);}//MEJOR HACER Q HOLDEE UN BOOLEANO?
//     } else {
//         for ent in query.iter() {cmd.entity(ent).remove::<InputDuck>();}
//     }
// }


#[allow(unused_parens, )]
pub fn send_move_input_to_server(
    mut cmd: Commands,  move_input: Query<(Entity, &InputMoveVector), (Changed<InputMoveVector>, With<ControlledLocally>)>,
) {
    for (being_ent, move_vec) in move_input.iter() {
        trace!(target: "movement", "Sending move input for entity {:?} with vector {:?}", being_ent, move_vec);
        cmd.client_trigger( SendMoveInput { being_ent, vec: move_vec.clone(), } );
    }

}

#[allow(unused_parens, )]
pub fn receive_move_input_from_client(
    trigger: Trigger<FromClient<SendMoveInput>>,
    mut controlled_beings_query: Query<(&mut InputMoveVector, &ControlledBy, ), ()>,

) -> Result {
    let SendMoveInput { vec: new_vec, being_ent } = trigger.event.clone();
    
    if let Ok((mut input_vec, controlled_by, )) = controlled_beings_query.get_mut(being_ent) {
        if controlled_by.client == trigger.client_entity {
            if input_vec.0 != new_vec.0 { input_vec.0 = new_vec.0; }
            //debug!(target: "movement", "Received move input for entity {:?} with vector {:?}", being_ent, new_vec);
        } else {
            warn!("Client tried to control a being not controlled by them: {}", being_ent);
        }
    } else {
        warn!("Client tried to control a being that does not exist in server or is not controllable {}", being_ent);
    }
    
    Ok(())
}

#[allow(unused_parens, )]
pub fn apply_movement(
    mut cmd: Commands, time: Res<Time>, 
    state : Res<State<GameSetupType>>,
    server: Option<Res<RenetServer>>,
    
    mut query: Query<(Entity, &InputSpeedVector, &mut Transform, &mut MoveAnimActive, Has<ControlledLocally>), >,
) {
    for (being_ent, InputSpeedVector(speed_vec), mut transform, mut move_anim, controlled_locally) in query.iter_mut() {

        if *state.get() == GameSetupType::AsJoiner && !controlled_locally { continue;}

        let delta = time.delta_secs();
        let movement = speed_vec  * delta;

        if movement != Vec2::ZERO {
            transform.translation += movement.extend(0.0);
            if server.is_some() {
                //info!("Sending transform for being: {:?}", being_ent);
                let to_clients = ToClients { 
                    mode: SendMode::Broadcast, 
                    event: TransformFromServer::new(being_ent, transform.clone(), true),
                };
                cmd.server_trigger(to_clients);
            }
            
            if !move_anim.0 { move_anim.0 = true; }
        } 
        else if move_anim.0 { move_anim.0 = false; }
    }
}



#[allow(unused_parens)]
pub fn update_facing_dir(mut query: Query<(&InputSpeedVector, &mut FacingDirection), >) {
    for (InputSpeedVector(dir_vec), mut facing_dir) in query.iter_mut() {
        if dir_vec.xy() == Vec2::ZERO {continue;}
        
        *facing_dir = if dir_vec.x.abs() > dir_vec.y.abs() {
            if dir_vec.x < 0.0 {FacingDirection::Left} else {FacingDirection::Right}
        } else {
            if dir_vec.y <= 0.0 {FacingDirection::Down} else {FacingDirection::Up}
        };
    }
}

//PARA HACER ANTÍDOTOS Q ATACAN SUSTANCIAS ESPECÍFICAS, HACER OTRO SISTEMA Q AFECTE EL POWER DE OTROS EFECTOS

#[allow(unused_parens)]//LO HACE EL CLEINT TMB CON LOS Q CONTROLA EL PARA TENER UNA TRANSFORM Q SE ACTUALIZA PREDECIBLEMENTE SIMILAR AL SERVER
pub fn process_movement_modifiers(
    //TODO: ACELERACIÓN Y FRICCIÓN? P. EJ, PARA TENER CABALLOS CON INERCIA. USAR BEVY RAPIER DESP
    state : Res<State<GameSetupType>>,
    mut being_query: Query<(&AppliedModifiers, &InputMoveVector, &mut InputSpeedVector, Has<ControlledLocally>), >,
    speed_query: Query<(
        &EffectiveValue,
        &OperationType,
        Has<Speed>,
        Has<InvertMovement>,
        Has<MitigatingOnly>
    ), ( )>, 
) {
    for (applied, InputMoveVector(inp_vec), mut final_vec, controlled_locally) in being_query.iter_mut() {
        let is_client = state.get() == &GameSetupType::AsJoiner;
        if is_client && !controlled_locally { continue;}

        final_vec.0 = *inp_vec;

        let mut speed_max: f32 = f32::INFINITY;
        let mut speed_min: f32 = 0.0;

        let mut speed_scale: f32 = 1.0;//NO RECOMENDADO USAR MULTIPLIERS (MÁS DIFÍCIL DE BALANCEAR)
        
        let mut speed_neg_sum: f32 = 0.0;
        let mut slowdown_mitigators_sum: f32 = 0.0; 
        let mut speed_sum: f32 = 40000.0;//ESTE 400.0 ES PROVISORIO, DESPUES CAMBIAR A 0.<---------------------

        let mut invert_sum: f32 = 0.0;
        let mut invert_scale: f32 = 1.0;

        for effect in applied.entities().iter() {
            if let Ok((&EffectiveValue(val), optype, speed, invert, mitigating)) = speed_query.get(*effect) {
                match optype {
                    OperationType::Offsetting => {
                        if speed {

                            if val > 0.0 {
                                if mitigating{
                                    slowdown_mitigators_sum += val;
                                } else {
                                    speed_sum += val;
                                }
                            } else {
                                speed_neg_sum += val;
                            }
                        }
                        if invert {invert_sum += val;}
                    },
                    OperationType::Scaling => {
                        if speed { speed_scale *= val.max(0.0); }
                        if invert { invert_scale *= val.max(0.0); }
                    }
                    OperationType::Min => {
                        speed_min = speed_min.max(val)
                    },
                    OperationType::Max => {
                        speed_max = speed_max.min(val).max(0.0); 
                    },
                }
                
            }
        }
        speed_sum += (speed_neg_sum + slowdown_mitigators_sum);

        let final_speed = (speed_sum * speed_scale).max(speed_min).min(speed_max).max(0.0);
        
        
        final_vec.0 *= final_speed;

        if invert_sum * invert_scale > 1.0 { final_vec.0 = -final_vec.0; }
    }
}


//#[cfg(not(feature = "headless_server"))]
#[allow(unused_parens)]
pub fn on_receive_transf_from_server(//TODO REHACER TODO ESTO CON ALGUNA CRATE DE INTERPOLATION/PREDICTION/ROLLBACK/LOQSEA
    trigger: Trigger<TransformFromServer>, client: Option<Res<RenetClient>>,
    mut being_query: Query<(&mut Transform, &ControlledBy, &HumanControlled)>,
    selfplayer: Single<(Entity), (With<OfSelf>, With<Player>)>,
) -> Result {
    let TransformFromServer { being: entity, trans: transform, interpolate } = trigger.event().clone();

    if client.is_none() {return Ok(());}

    let Ok((mut transf, controller, human_controlled)) = being_query.get_mut(entity) else {
        let err = Err(BevyError::from(format!("Received transform for entity that does not exist: {:?}", entity)));
        return err;
    };
    
    //debug!("Applying transform to entity: {:?}", entity);
    if controller.client == selfplayer.into_inner() && interpolate && human_controlled.0 {
        transf.translation = transf.translation.lerp(transform.translation, 0.5);
    } else {
        *transf = transform;
    }
    
   Ok(())
}
