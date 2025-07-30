#[allow(unused_imports)] use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::Target;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::RenetServer;
use crate::game::{being::{being_components::{Being, ControlledBy, ControlledLocally, CpuControlled }, modifier::modifier_components::*, movement::{movement_components::*, movement_events::*}}, game_components::FacingDirection, player::{player_components::{Controls, HostPlayer}, player_resources::KeyboardInputMappings}};

#[allow(unused_parens, )]
pub fn human_move_input(
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
        move_input_dir.0 = input_dir;
    }
}

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
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
        for ent in query.iter() {cmd.entity(ent).insert(InputDuck);}
    } else {
        for ent in query.iter() {cmd.entity(ent).remove::<InputDuck>();}
    }
}

#[allow(unused_parens, )]
pub fn send_move_input_to_server(
    mut cmd: Commands, 
    human_move_input: Query<(&InputMoveVector), (Changed<InputMoveVector>, With<ControlledLocally>, Without<CpuControlled>)>,
    cpu_move_input: Query<(Entity, &InputMoveVector), (Changed<InputMoveVector>, With<ControlledLocally>, With<CpuControlled>)>,
    mut map: ResMut<ServerEntityMap>,
) {

   for (move_vec) in human_move_input.iter() {
        cmd.client_trigger(
            SendMoveInput {
                vec: move_vec.clone(),
                being_ent: None,
            }
        );
        break;
   }
    for (entity, move_vec) in cpu_move_input.iter() {
        let entity = map.client_entry(entity).get().unwrap();
        cmd.client_trigger(
            SendMoveInput {
                vec: move_vec.clone(),
                being_ent: Some(entity),
            }
        );
   }
}

#[allow(unused_parens, )]
pub fn receive_move_input_from_client(
    trigger: Trigger<FromClient<SendMoveInput>>,
    mut cmd: Commands,
    ents_controlled_by_client: Query<(&Controls, ), ( )>,
    mut controlled_beings_query: Query<(&mut InputMoveVector, &ControlledBy, Option<&CpuControlled>), ()>,

) -> Result {
    let SendMoveInput { vec: new_vec, being_ent } = trigger.event.clone();
    
    let ents_controlled_by_client = ents_controlled_by_client.get(trigger.client_entity)?;

    if let Some(being_ent) = being_ent {
        if let Ok((mut input_vec, controlled_by, _)) = controlled_beings_query.get_mut(being_ent) {
            if controlled_by.player == trigger.client_entity {
                input_vec.0 = new_vec.0;
            } else {
                warn!("Client tried to control a being not controlled by them: {}", being_ent);
            }
        } else {
            warn!("Client tried to control a being that does not exist or is not controlled by them: {}", being_ent);
        }
    } else {
        for controlled_ent in ents_controlled_by_client.0.iter() {
            if let Ok((mut inp_mov_vec, _, cpu_controlled)) = controlled_beings_query.get_mut(controlled_ent) {
                if cpu_controlled.is_none() {
                    inp_mov_vec.0 = new_vec.0;
                }
            } 
        }
   
    }

    Ok(())
}


#[allow(unused_parens, )]
pub fn apply_movement(
    mut cmd: Commands,
    time: Res<Time>,
    server: Option<Res<RenetServer>>,
    mut query: Query<(Entity, &FinalMoveVector, &mut Transform), /*(With<VoluntarilyMoving>)*/>,
) {
    for (being_ent, FinalMoveVector(move_dir), mut transform) in query.iter_mut() {
        let speed = 800.0;
        let delta = time.delta_secs();
        let movement = move_dir * speed * delta;

        if movement != Vec2::ZERO {
            transform.translation += movement.extend(0.0);
            if server.is_some() {
                info!("Sending transform for being: {:?}", being_ent);
                let to_clients = ToClients { 
                    mode: SendMode::Broadcast, 
                    event: TransformFromServer::new(being_ent, transform.clone(), true),
                };
                cmd.server_trigger(to_clients);
            }
            
            cmd.entity(being_ent).insert(VoluntarilyMoving);//PROVISORIO
        } else{

            cmd.entity(being_ent).remove::<VoluntarilyMoving>();//PROVISORIO
        }
    }
}


#[allow(unused_parens)]
pub fn on_receive_transf_from_server(//TODO REHACER TODO ESTO CON ALGUNA CRATE DE INTERPOLATION/PREDICTION/ROLLBACK/LOQSEA
    trigger: Trigger<TransformFromServer>,
    mut query: Query<&mut Transform>,
    mut map: ResMut<ServerEntityMap>,
    server: Option<Res<RenetServer>>,

) -> Result {
    let TransformFromServer { being: entity, trans: transform, interpolate } = trigger.event().clone();

    if server.is_some() {return Ok(());}

    info!("Received transform for entity: {:?}", entity);

    if let Some(entity) = map.server_entry(entity).get() {
        if let Ok(mut transf) = query.get_mut(entity) {
            info!("Applying transform to entity: {:?}", entity);
            if interpolate {
                transf.translation = transf.translation.lerp(transform.translation, 0.7);//TODO HACER Q CADA CIERTO TIEMPO SE FUERZE LA POSICIÓN REAL SIN INTERPOLACIÓN
            } else {
                *transf = transform;
            }
        } else {
            warn!("Received transform for entity that does not exist: {:?}", entity);
        }
    }
    else {warn!("Received transform for entity that is not in the server entity map: {:?}", entity);}

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

#[allow(unused_parens)]
pub fn process_movement_modifiers(mut cmd: Commands, 
    mut being_query: Query<(Entity, &AppliedModifiers, &InputMoveVector, &mut FinalMoveVector), (With<Being>)>,
    mut effects_query: Query<(
        &ModifierCategories,
        &EffectivePotency,
        Option<&MitigatingOnly>,
        Option<&MultiplyingModifier>,
        Option<&Speed>, 
        Option<&InvertMovement>, 
    )>, 

){

    for (ent, modifiers, InputMoveVector(inp_vec), mut final_vec) in being_query.iter_mut() {

        final_vec.0 = *inp_vec;

        // let mut changes_position = false;
        // let mut invert_movement: f32 = 0.0;

        // for effect in modifiers.entities().iter() {
        //     if let Ok((_modifier, _potency, mitigating_only, multiplying_modifier, speed, invert_move)) = effects_query.get(*effect) {

        //         match (invert_move, mitigating_only) {
        //             (Some(_), None) => {
        //                 invert_movement += 1.0;
        //             },
        //             (None, Some(_)) => {
        //                 invert_movement -= 1.0;
        //             },
        //             (Some(_), Some(_)) => {},
        //             (None, None) => {}
        //         }
        //     }
        // }

        // if changes_position {
        //     cmd.entity(ent).insert(VoluntarilyMoving);
        // }

        // if invert_movement > 0.0 {
        //     final_vec.0 = -final_vec.0;
        // }
        
    }
}


