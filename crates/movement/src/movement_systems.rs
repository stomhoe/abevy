use core::f32;

use being_shared::{ControlledBy, ControlledLocally, HumanControlled};
use bevy::ecs::entity::EntityHashSet;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;

use dimension_shared::DimensionRef;
use game_common::game_common_components::Direction;
use modifier::modifier_types::WalkSpeed;
use modifier::{modifier_components::*, modifier_move_components::*};
use player::{player_components::*, player_resources::KeyboardInputMappings};
use sprite_animation_shared::MoveAnimActive;
use tilemap::tile::tile_components::WalkSpeedMultIfOnTop;
use tilemap::tilemap_resources::TilesAtGpos;
use tilemap_shared::GlobalTilePos;

use crate::{
    movement_components::*,
    movement_messages::{SendMoveInput, TransformFromServer},
};

#[allow(unused_parens)]
///DON'T DELETE THIS
pub fn do_free_movement(
    client_state: Res<State<ClientState>>,

    mut query: Query<
        (
            &mut FinalMoveVector,
            &ProcessedInputMoveVector,
            &OutputSpeedMagnitude,
            Has<ControlledLocally>,
        ),
        (Without<GridLockedMovement>,),
    >,
) {
    for (
        mut final_move_vector,
        &ProcessedInputMoveVector(input),
        &OutputSpeedMagnitude(speed_mag),
        controlled_locally,
    ) in query.iter_mut()
    {
        if *client_state.get() == ClientState::Connected && !controlled_locally {
            continue;
        }

        if final_move_vector.0 != (input * speed_mag) {
            final_move_vector.0 = (input * speed_mag);
        }
    }
}

#[allow(unused_parens)]
pub fn modify_transform(
    mut query: Query<
        (
            &mut Transform,
            &mut MoveAnimActive,
            Entity,
            &FinalMoveVector,
        ),
        (Without<GridLockedMovement>,),
    >,
    time: Res<Time>,
    server_state: Res<State<ServerState>>,
    mut ewriter: MessageWriter<ToClients<TransformFromServer>>,
) {
    let mut to_write = Vec::new();

    for (mut transform, mut move_anim, being_ent, move_vec) in query.iter_mut() {
        if move_vec.0 != Vec2::ZERO {
            if !move_anim.0 {
                move_anim.0 = true;
            }

            transform.translation += (move_vec.0 * time.delta_secs()).extend(0.0);
            if server_state.get() == &ServerState::Running {
                let to_clients = ToClients {
                    mode: SendMode::BroadcastExcept(ClientId::Server),
                    message: TransformFromServer::new(being_ent, transform.clone(), false),
                };
                to_write.push(to_clients);
            }
        } else {
            if move_anim.0 {
                move_anim.0 = false;
            }
        }
    }
    ewriter.write_batch(to_write);
}

#[allow(unused_parens)]
pub fn prepare_grid_locked_movement(
    tiles_at_gpos: Res<TilesAtGpos>,
    blocking_tiles: Query<&WalkSpeedMultIfOnTop, ()>,
    time: Res<Time>,
    server_state: Res<State<ServerState>>,
    mut ewriter: MessageWriter<ToClients<TransformFromServer>>,
    mut query: Query<
        (
            &mut FinalMoveVector,
            &mut Transform,
            &mut MoveAnimActive,
            Entity,
            &ProcessedInputMoveVector,
            &OutputSpeedMagnitude,
            &DimensionRef,
            &mut QueuedGridMoveDir,
            Has<ControlledLocally>,
            Has<WallPhaser>,
        ),
        (With<GridLockedMovement>),
    >,
) {
    for (
        mut final_move_vector,
        mut transform,
        mut move_anim,
        being_ent,
        input,
        output_speed_mag,
        &dim_ref,
        mut queued_dir,
        controlled_locally,
        can_phase,
    ) in query.iter_mut()
    {
        if !controlled_locally {
            continue;
        }

        let snapped_translation: Vec3 = GlobalTilePos::from(transform.translation.truncate())
            .to_translation(transform.translation.z);
        let offset = transform.translation - snapped_translation;
        let is_mid_tile = offset.x.abs() > 0.01 || offset.y.abs() > 0.01;
        let raw_input = input.0;

        let axis_input_dir = if raw_input == Vec2::ZERO {
            Vec2::ZERO
        } else if raw_input.x.abs() >= raw_input.y.abs() {
            Vec2::new(raw_input.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, raw_input.y.signum())
        };

        let (mut dir_vec, finish_current_tile_only) = if is_mid_tile {
            let last_move = final_move_vector.0;
            let active_dir = if last_move != Vec2::ZERO {
                if last_move.x.abs() >= last_move.y.abs() {
                    Vec2::new(last_move.x.signum(), 0.0)
                } else {
                    Vec2::new(0.0, last_move.y.signum())
                }
            } else if offset.x.abs() >= offset.y.abs() {
                Vec2::new(1.0, 0.0)
            } else {
                Vec2::new(0.0, 1.0)
            };

            let finish_current_tile_only =
                axis_input_dir == Vec2::ZERO || axis_input_dir != active_dir;

            if axis_input_dir != Vec2::ZERO && axis_input_dir != active_dir {
                queued_dir.0 = axis_input_dir;
            }

            (active_dir, finish_current_tile_only)
        } else if queued_dir.0 != Vec2::ZERO {
            let queued = queued_dir.0;
            queued_dir.0 = Vec2::ZERO;
            (queued, false)
        } else {
            (axis_input_dir, false)
        };

        if dir_vec == Vec2::ZERO {
            transform.translation = snapped_translation;
            final_move_vector.0 = Vec2::ZERO;
            if move_anim.0 {
                move_anim.0 = false;
            }
            continue;
        }

        let input = {
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
        };

        let delta = time.delta_secs();
        if delta <= 0.0 {
            final_move_vector.0 = Vec2::ZERO;
            continue;
        }

        let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
        let distance = output_speed_mag.0 * delta;
        let mut remaining_distance = distance;

        let is_blocked_at = |target_gpos: GlobalTilePos| -> bool {
            if can_phase {
                return false;
            }
            for &tile_entity in tiles_at_gpos.tiles_at_pos(dim_ref, target_gpos) {
                if let Ok(walk_speed) = blocking_tiles.get(tile_entity) {
                    if walk_speed.0 == 0.0 {
                        return true;
                    }
                } else {
                    return true;
                }
            }
            false
        };

        let mut current_translation = transform.translation;
        let mut current_dir = input;
        let mut last_move_dir = Vec2::ZERO;
        let mut moved = false;

        let mut safety = 0;
        while remaining_distance > 0.0 && safety < 64 {
            safety += 1;

            let current_snapped = GlobalTilePos::from(current_translation.truncate())
                .to_translation(current_translation.z);
            let current_offset = current_translation - current_snapped;
            let step_distance = if current_dir.x != 0.0 {
                tile_size.x
            } else {
                tile_size.y
            };

            let next_target = current_snapped.xy() + (current_dir * step_distance);
            if is_blocked_at(GlobalTilePos::from(next_target)) {
                queued_dir.0 = Vec2::ZERO;
                if !moved {
                    current_translation = current_snapped;
                }
                break;
            }

            let mut remaining_to_boundary = if current_dir.x != 0.0 {
                if current_dir.x > 0.0 {
                    (step_distance - current_offset.x).max(0.0)
                } else {
                    current_offset.x.abs().max(0.0)
                }
            } else {
                if current_dir.y > 0.0 {
                    (step_distance - current_offset.y).max(0.0)
                } else {
                    current_offset.y.abs().max(0.0)
                }
            };

            if remaining_to_boundary <= 0.0001 {
                current_translation = current_snapped;
                let target = current_snapped.xy() + (current_dir * step_distance);
                if is_blocked_at(GlobalTilePos::from(target)) {
                    queued_dir.0 = Vec2::ZERO;
                    if !moved {
                        current_translation = current_snapped;
                    }
                    break;
                }
                remaining_to_boundary = step_distance;
            }

            let step = remaining_distance.min(remaining_to_boundary);
            let will_reach_boundary = step >= remaining_to_boundary - 0.0001;

            if will_reach_boundary {
                let target = current_snapped.xy() + (current_dir * step_distance);
                if is_blocked_at(GlobalTilePos::from(target)) {
                    queued_dir.0 = Vec2::ZERO;
                    if !moved {
                        current_translation = current_snapped;
                    }
                    break;
                }

                current_translation += (current_dir * remaining_to_boundary).extend(0.0);
                remaining_distance -= remaining_to_boundary;
                moved |= remaining_to_boundary > 0.0;
                last_move_dir = current_dir;

                if finish_current_tile_only && queued_dir.0 == Vec2::ZERO {
                    break;
                }

                if queued_dir.0 != Vec2::ZERO {
                    current_dir = queued_dir.0;
                    queued_dir.0 = Vec2::ZERO;
                }
            } else {
                current_translation += (current_dir * step).extend(0.0);
                remaining_distance -= step;
                moved |= step > 0.0;
                last_move_dir = current_dir;
                break;
            }
        }

        transform.translation = current_translation;
        if moved {
            final_move_vector.0 = last_move_dir * output_speed_mag.0;
            if !move_anim.0 {
                move_anim.0 = true;
            }

            if server_state.get() == &ServerState::Running {
                let to_clients = ToClients {
                    mode: SendMode::BroadcastExcept(ClientId::Server),
                    message: TransformFromServer::new(being_ent, transform.clone(), false),
                };
                ewriter.write(to_clients);
            }
        } else {
            final_move_vector.0 = Vec2::ZERO;
            if move_anim.0 {
                move_anim.0 = false;
            }
        }
    }
}

//PARA HACER ANTÍDOTOS Q ATACAN SUSTANCIAS ESPECÍFICAS, HACER OTRO SISTEMA Q AFECTE EL POWER DE OTROS EFECTOS

#[allow(unused_parens)]
pub fn process_speed_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        Entity,
        &AppliedModifiers,
        &ProcessedInputMoveVector,
        &FinalMoveVector,
        Option<&QueuedGridMoveDir>,
        &mut OutputSpeedMagnitude,
        Option<&mut Direction>,
        Has<ControlledLocally>,
    )>,
    modifiers_query: Query<
        (
            Entity,
            &ModifierTarget,
            &CurrEffectiveValue,
            &ApplyMode,
            Has<MitigatingOnly>,
        ),
        (With<WalkSpeed>,),
    >,
) {
    for (
        being_ent,
        applied,
        input_vec,
        move_vec,
        queued_dir,
        mut speed_output,
        mut facing_dir,
        controlled_locally,
    ) in being_query.iter_mut()
    {
        let is_client = state.get() != &ClientState::Disconnected;
        if is_client && !controlled_locally {
            continue;
        }

        let mut speed_max: f32 = f32::INFINITY;
        let mut speed_min: f32 = 0.0;

        let mut speed_scale: f32 = 1.0; //NO RECOMENDADO USAR MULTIPLIERS (MÁS DIFÍCIL DE BALANCEAR)

        let mut speed_neg_sum: f32 = 0.0;
        let mut slowdown_mitigators_sum: f32 = 0.0;
        let mut speed_sum: f32 = 0.0; //ESTE 400.0 ES PROVISORIO, DESPUES CAMBIAR A 0.<---------------------

        let mut effects = EntityHashSet::default();
        applied.entities().iter().for_each(|&ent| {
            effects.insert(ent);
        });

        for (modifier_ent, target, ..) in modifiers_query.iter() {
            if target.0 == being_ent {
                effects.insert(modifier_ent);
            }
        }

        for effect in effects.iter() {
            if let Ok((_, _, &CurrEffectiveValue(val), optype, mitigating)) =
                modifiers_query.get(*effect)
            {
                match optype {
                    ApplyMode::Add => {
                        if val > 0.0 {
                            if mitigating {
                                slowdown_mitigators_sum += val;
                            } else {
                                speed_sum += val;
                            }
                        } else {
                            speed_neg_sum += val;
                        }
                    }
                    ApplyMode::Mul => {
                        speed_scale *= val.max(0.0);
                    }
                    ApplyMode::Min => speed_min = speed_min.max(val),
                    ApplyMode::Max => {
                        speed_max = speed_max.min(val).max(0.0);
                    }
                }
            }
        }
        speed_sum += (speed_neg_sum + slowdown_mitigators_sum);

        let final_speed = (speed_sum * speed_scale)
            .max(speed_min)
            .min(speed_max)
            .max(0.0);
        speed_output.0 = final_speed;

        if let Some(mut facing_dir) = facing_dir {
            let dir_vec = if move_vec.0.xy() != Vec2::ZERO {
                move_vec.0.xy()
            } else if queued_dir.map_or(false, |q| q.0 != Vec2::ZERO) {
                queued_dir.unwrap().0
            } else {
                input_vec.0.xy()
            };

            if dir_vec != Vec2::ZERO {
                *facing_dir =
                    if dir_vec.x.abs() > dir_vec.y.abs() || (dir_vec.x.abs() == dir_vec.y.abs()) {
                        if dir_vec.x < 0.0 {
                            Direction::West
                        } else {
                            Direction::East
                        }
                    } else {
                        if dir_vec.y <= 0.0 {
                            Direction::South
                        } else {
                            Direction::North
                        }
                    };
            }
        }
    }
}

#[allow(unused_parens)]
pub fn update_facing_dir(
    mut query: Query<
        (&ProcessedInputMoveVector, &FinalMoveVector, &mut Direction),
        (Without<GridLockedMovement>,),
    >,
) {
    for (ProcessedInputMoveVector(input_vec), FinalMoveVector(move_vec), mut facing_dir) in
        query.iter_mut()
    {
        let dir_vec = if move_vec.xy() != Vec2::ZERO {
            move_vec.xy()
        } else {
            input_vec.xy()
        };

        if dir_vec == Vec2::ZERO {
            continue;
        }

        *facing_dir = if dir_vec.x.abs() > dir_vec.y.abs() || (dir_vec.x.abs() == dir_vec.y.abs()) {
            if dir_vec.x < 0.0 {
                Direction::West
            } else {
                Direction::East
            }
        } else {
            if dir_vec.y <= 0.0 {
                Direction::South
            } else {
                Direction::North
            }
        };
    }
}
