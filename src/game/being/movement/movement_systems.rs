#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_replicon_renet::renet::RenetServer;
use crate::game::{being::{being_components::{Being, ControlledBy, ControlledLocally, CpuControlled }, modifier::modifier_components::*, movement::{movement_components::*, movement_events::*}, sprite::sprite_components::MoveAnimActive}, game_components::FacingDirection, player::{player_components::{ OfSelf, Player}, player_resources::KeyboardInputMappings}};

#[allow(unused_parens, )]
pub fn update_human_move_input(
    keys: Res<ButtonInput<KeyCode>>,
    input_mappings: Res<KeyboardInputMappings>,
    mut move_input: Query<(&mut InputMoveVector), (With<ControlledLocally>, Without<CpuControlled>)>,
) {
    let mut input_dir = Vec2::ZERO;
    if keys.pressed(input_mappings.move_up) {input_dir.y += 1.0;}
    if keys.pressed(input_mappings.move_down) {input_dir.y -= 1.0;}
    if keys.pressed(input_mappings.move_left) {input_dir.x -= 1.0;}
    if keys.pressed(input_mappings.move_right) {input_dir.x += 1.0;}
    
    if input_dir != Vec2::ZERO {input_dir = input_dir.normalize();}

    for mut move_input_dir in move_input.iter_mut() {
        if move_input_dir.0 != input_dir { move_input_dir.0 = input_dir; }
    }
}

#[allow(unused_parens)]
pub fn update_jump_duck_inputs(
    mut cmd: Commands, 
    query: Query<(Entity),(With<ControlledLocally>, Without<CpuControlled>)>,
    keys: Res<ButtonInput<KeyCode>>,
    input_mappings: Res<KeyboardInputMappings>,) 
{
    if keys.pressed(input_mappings.jump_or_fly) {
        for ent in query.iter() {cmd.entity(ent).insert(InputJump);}
    } else {
        for ent in query.iter() {cmd.entity(ent).remove::<InputJump>();}
    }
    if keys.pressed(input_mappings.duck) {
        for ent in query.iter() {cmd.entity(ent).insert(InputDuck);}//MEJOR HACER Q HOLDEE UN BOOLEANO?
    } else {
        for ent in query.iter() {cmd.entity(ent).remove::<InputDuck>();}
    }
}

#[allow(unused_parens, )]
pub fn send_move_input_to_server(
    mut cmd: Commands,  move_input: Query<(Entity,&InputMoveVector), (Changed<InputMoveVector>, With<ControlledLocally>)>,
) {
    for (being_ent, move_vec) in move_input.iter() {
        //info!(target: "movement", "Sending move input for entity {:?} with vector {:?}", being_ent, move_vec);
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
        if controlled_by.player == trigger.client_entity {
            if input_vec.0 != new_vec.0 { input_vec.0 = new_vec.0; }
            //info!(target: "movement", "Received move input for entity {:?} with vector {:?}", being_ent, new_vec);
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
    mut cmd: Commands, time: Res<Time>, server: Option<Res<RenetServer>>,
    mut query: Query<(Entity, &FinalMoveVector, &mut Transform, &mut MoveAnimActive, Has<ControlledLocally>), >,
) {
    for (being_ent, FinalMoveVector(move_dir), mut transform, mut move_anim, controlled_locally) in query.iter_mut() {

        if server.is_none() && !controlled_locally { continue;}

        let speed = 800.0;
        let delta = time.delta_secs();
        let movement = move_dir * speed * delta;


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
            
            if !move_anim.0 {
                move_anim.0 = true;
            }
        } else if move_anim.0 {
            move_anim.0 = false;
        }
    }
}

#[allow(unused_parens)]
pub fn on_receive_transf_from_server(//TODO REHACER TODO ESTO CON ALGUNA CRATE DE INTERPOLATION/PREDICTION/ROLLBACK/LOQSEA
    trigger: Trigger<TransformFromServer>,
    mut query: Query<&mut Transform>,
    server: Option<Res<RenetServer>>,
    selfplayer: Single<(Entity), (With<OfSelf>, With<Player>)>,
    controlled_by: Query<&ControlledBy>,
) -> Result {
    let TransformFromServer { being: entity, trans: transform, interpolate } = trigger.event().clone();

    if server.is_some() {return Ok(());}

    if let Ok(mut transf) = query.get_mut(entity) {
        //info!("Applying transform to entity: {:?}", entity);
        if let Ok(controller) = controlled_by.get(entity) {
            if controller.player == selfplayer.into_inner() && interpolate {
                transf.translation = transf.translation.lerp(transform.translation, 0.5);
            } else {
                *transf = transform;
            }
        }
    } else {
        error!("Received transform for entity that does not exist: {:?}", entity);
    }

   Ok(())
}

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

#[allow(unused_parens)]//LO HACE EL CLEINT TMB CON LOS Q CONTROLA EL PARA TENER UNA TRANSFORM Q SE ACTUALIZA PREDECIBLEMENTE SIMILAR AL SERVER
pub fn process_movement_modifiers(
    server: Option<Res<RenetServer>>,
    mut being_query: Query<(&AppliedModifiers, &InputMoveVector, &mut FinalMoveVector, Has<ControlledLocally>), (With<Being>, )>,
    speed_query: Query<(
        &EffectiveValue,
        &OperationType,
        Has<Speed>,//DEJAR CON HAS POR AHORA
        Has<InvertMovement>,
        Has<MitigatingOnly>
    ), ( )>, //PRIMERO HACER UN SUMATORIO NEGATIVO DE EXCLUSIVAMENTE LOS NEGATIVOS

) {
    for (applied, InputMoveVector(inp_vec), mut final_vec, controlled_locally) in being_query.iter_mut() {
        if server.is_none() && !controlled_locally { continue;}

        final_vec.0 = *inp_vec;

        let mut speed_max: f32 = 0.0;
        let mut speed_min: f32 = 0.0;

        let mut speed_scale: f32 = 1.0;//NO RECOMENDADO USAR MULTIPLIERS (MÁS DIFÍCIL DE BALANCEAR)
        
        let mut speed_neg_sum: f32 = 0.0;
        let mut slowdown_mitigators_sum: f32 = 0.0;
        let mut speed_offset: f32 = 0.0;

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
                                    speed_offset += val;
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
                        speed_min = speed_min.min(val).min(speed_max).max(0.0);
                    },
                    OperationType::Max => {
                        speed_max = speed_max.max(val).max(0.0); 
                        speed_min = speed_min.min(val).max(0.0);
                    },
                }
                
            }
        }
        speed_offset += (speed_neg_sum + slowdown_mitigators_sum).max(0.0);

        let final_speed = speed_offset * speed_scale;
        
        final_vec.0 += final_speed;

        if invert_sum * invert_scale > 1.0 { final_vec.0 = -final_vec.0; }
    }
}


