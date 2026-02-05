use core::f32;

use being_shared::{ControlledBy, ControlledLocally, HumanControlled};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity::EntityHashSet;

use game_common::game_common_components::Direction;
use modifier::{modifier_components::*, modifier_move_components::*};
use player::{player_components::*, player_resources::KeyboardInputMappings};
use sprite_animation_shared::MoveAnimActive;
use dimension_shared::DimensionRef;
use tilemap::tile::tile_components::{ WalkSpeed};
use tilemap::tilemap_resources::TilesAtGpos;
use tilemap_shared::GlobalTilePos;

use crate::{movement_components::*, movement_messages::{SendMoveInput, TransformFromServer}};

#[allow(unused_parens, )]///DON'T DELETE THIS
pub fn do_free_movement(
    client_state : Res<State<ClientState>>,
    
    mut query: Query<(&mut FinalMoveVector, &ProcessedInputMoveVector, &OutputSpeedMagnitude, Has<ControlledLocally>), (Without<GridLockedMovement>, )>,
) {
    for (mut final_move_vector, &ProcessedInputMoveVector(input), &OutputSpeedMagnitude(speed_mag), controlled_locally) in query.iter_mut() {

        if *client_state.get() == ClientState::Connected && !controlled_locally { continue; }


        if final_move_vector.0 != (input * speed_mag) {
            final_move_vector.0 = (input * speed_mag);
        }
    }
}


#[allow(unused_parens)]
pub fn modify_transform(
    mut query: Query<(&mut Transform, &mut MoveAnimActive, Entity, &FinalMoveVector, ), ()>,
    time: Res<Time>,
    server_state: Res<State<ServerState>>,
    mut ewriter: MessageWriter<ToClients<TransformFromServer>>,
) {
    let mut to_write = Vec::new();

    for (mut transform, mut move_anim, being_ent, move_vec ) in query.iter_mut() {
        
        if move_vec.0 != Vec2::ZERO {
            if !move_anim.0 { move_anim.0 = true; }
        
            transform.translation += (move_vec.0 * time.delta_secs()).extend(0.0);
            if server_state.get() == &ServerState::Running {
                let to_clients = ToClients { 
                    mode: SendMode::BroadcastExcept(ClientId::Server), 
                    message: TransformFromServer::new(being_ent, transform.clone(), false),
                };
                to_write.push(to_clients);
            }
        }
        else{
            if move_anim.0 { move_anim.0 = false; }
        }
    }
    ewriter.write_batch(to_write);   
}


#[allow(unused_parens)]
pub fn prepare_grid_locked_movement(
    mut cmd: Commands,
    tiles_at_gpos: Res<TilesAtGpos>,
    blocking_tiles: Query<&WalkSpeed, ()>,
    mut query: Query<(
        &mut FinalMoveVector,
        &mut Transform,
        &ProcessedInputMoveVector,
        &OutputSpeedMagnitude,
        &DimensionRef,
        Has<ControlledLocally>,
        Has<WallPhaser>,
    ), (With<GridLockedMovement>)>,
) {
    debug!(target: "grid_movement", "prepare_grid_locked_movement system running");
    for (mut final_move_vector, mut transform, input, output_speed_mag, &dim_ref, controlled_locally, can_phase) in query.iter_mut() {
        
        if !controlled_locally { continue; }


        
        let snapped_translation: Vec3 = GlobalTilePos::from(transform.translation.truncate()).to_translation(transform.translation.z);

        let input = {
            let mut dir_vec = input.0;
            if dir_vec == Vec2::ZERO {
                transform.translation = snapped_translation;
            final_move_vector.0 = Vec2::ZERO;
                
                continue;
            } else {
                if dir_vec.x.abs() >= dir_vec.y.abs() {
                    dir_vec.y = 0.0;
                    if (transform.translation.y - snapped_translation.y).abs() > 0.01 {
                        transform.translation.y = snapped_translation.y;
                    }
                    dir_vec.x = dir_vec.x.signum();
                } else {
                    dir_vec.x = 0.0;
                    if (transform.translation.x - snapped_translation.x).abs() > 0.01 {
                        transform.translation.x = snapped_translation.x;
                    }
                    dir_vec.y = dir_vec.y.signum();
                }
                dir_vec
            }
        };

        let target_gpos = GlobalTilePos::from((transform.translation.xy() + (input * GlobalTilePos::TILE_SIZE_PXS.as_vec2())));

        let mut blocked = false;
        for &tile_entity in tiles_at_gpos.tiles_at_pos(dim_ref, target_gpos) {
            if let Ok(walk_speed) = blocking_tiles.get(tile_entity) {
                if !can_phase && walk_speed.0 == 0.0 {
                    blocked = true;
                    break;
                }
            } else{
                if !can_phase {
                    blocked = true;
                    break;
                }
            }
        }
        if blocked {
            transform.translation = snapped_translation;
            final_move_vector.0 = Vec2::ZERO;
            continue;
        } 

        final_move_vector.0 = input * output_speed_mag.0;

    }
}


//PARA HACER ANTÍDOTOS Q ATACAN SUSTANCIAS ESPECÍFICAS, HACER OTRO SISTEMA Q AFECTE EL POWER DE OTROS EFECTOS


#[allow(unused_parens)]
pub fn process_speed_modifiers(
    state : Res<State<ClientState>>,
    mut being_query: Query<(Entity, &AppliedModifiers, &mut OutputSpeedMagnitude, Has<ControlledLocally>), >,
    modifiers_query: Query<(
        Entity,
        &ModifierTarget,
        &CurrFinalValue,
        &ApplyMode,
        Has<Speed>,
        Has<MitigatingOnly>
    ), ( )>, 
) {
    for (being_ent, applied, mut speed_output, controlled_locally) in being_query.iter_mut() {
        let is_client = state.get() != &ClientState::Disconnected;
        if is_client && !controlled_locally { continue;}

        let mut speed_max: f32 = f32::INFINITY;
        let mut speed_min: f32 = 0.0;

        let mut speed_scale: f32 = 1.0;//NO RECOMENDADO USAR MULTIPLIERS (MÁS DIFÍCIL DE BALANCEAR)
        
        let mut speed_neg_sum: f32 = 0.0;
        let mut slowdown_mitigators_sum: f32 = 0.0; 
        let mut speed_sum: f32 = 0.0;//ESTE 400.0 ES PROVISORIO, DESPUES CAMBIAR A 0.<---------------------

        let mut effects = EntityHashSet::default();
        applied.entities().iter().for_each(|&ent| { effects.insert(ent); });

        for (modifier_ent, target, ..) in modifiers_query.iter() {
            if target.0 == being_ent {
                effects.insert(modifier_ent);
            }
        }

        for effect in effects.iter() {
            if let Ok((_, _, &CurrFinalValue(val), optype, speed, mitigating)) = modifiers_query.get(*effect) {
                match optype {
                    ApplyMode::Add => {
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
                    },
                    ApplyMode::Mul => {
                        if speed { speed_scale *= val.max(0.0); }
                    }
                    ApplyMode::Min => {
                        speed_min = speed_min.max(val)
                    },
                    ApplyMode::Max => {
                        speed_max = speed_max.min(val).max(0.0); 
                    },
                }
            }
        }
        speed_sum += (speed_neg_sum + slowdown_mitigators_sum);

        let final_speed = (speed_sum * speed_scale).max(speed_min).min(speed_max).max(0.0);
        speed_output.0 = final_speed;
    }
}


#[allow(unused_parens)]
pub fn update_facing_dir(mut query: Query<(&ProcessedInputMoveVector, &mut Direction), >) {
    for (ProcessedInputMoveVector(dir_vec), mut facing_dir) in query.iter_mut() {
        if dir_vec.xy() == Vec2::ZERO {continue;}

        *facing_dir = if dir_vec.x.abs() > dir_vec.y.abs() || (dir_vec.x.abs() == dir_vec.y.abs()) {
            if dir_vec.x < 0.0 {Direction::West} else {Direction::East}
        } else {
            if dir_vec.y <= 0.0 {Direction::South} else {Direction::North}
        };
    }
}
