use core::f32;

use being_shared::ControlledLocally;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use dimension_shared::DimensionRef;
use game_common::game_common_components::CardinalDirection;
use modifier::modifier_types::WalkSpeed;
use modifier::{modifier_components::*, modifier_move_components::*};
use sprite_animation_shared::MoveAnimActive;
use tilemap::tile::tile_components::WalkSpeedMultIfOnTop;
use tilemap::tilemap_resources::TilesAtGpos;
use tilemap_shared::GlobalTilePos;

use crate::{
    movement_components::*,
    movement_messages::TransformFromServer,
};

fn normalize_to_axis_dir(input: Vec2) -> Vec2 {
    if input == Vec2::ZERO {
        Vec2::ZERO
    } else if input.x.abs() >= input.y.abs() {
        Vec2::new(input.x.signum(), 0.0)
    } else {
        Vec2::new(0.0, input.y.signum())
    }
}

fn set_animation_active(anim: &mut MoveAnimActive, is_active: bool) {
    if anim.0 != is_active { anim.0 = is_active; }
}

pub fn do_free_movement(
    mut query: Query<
        (&mut Transform, &mut MoveAnimActive, &MoveState),
        (Without<GridLockedMovement>,),
    >,
    time: Res<Time>,
) {
    for (mut transform, mut move_anim, move_state) in query.iter_mut() {
        let velocity = move_state.norm_move_dir * move_state.speed_magnitude;
        if velocity != Vec2::ZERO {
            set_animation_active(&mut move_anim, true);
            transform.translation += (velocity * time.delta_secs()).extend(0.0);
        } else {
            set_animation_active(&mut move_anim, false);
        }
    }
}

pub fn sync_movement_to_server(
    query: Query<
        (Entity, &Transform, &MoveState),
        (Without<GridLockedMovement>, Changed<Transform>),
    >,
    server_state: Res<State<ServerState>>,
    mut ewriter: MessageWriter<ToClients<TransformFromServer>>,
) {
    if server_state.get() != &ServerState::Running {
        return;
    }

    for (being_ent, transform, _) in query.iter() {
        let to_clients = ToClients {
            mode: SendMode::BroadcastExcept(ClientId::Server),
            message: TransformFromServer::new(being_ent, transform.clone(), false),
        };
        ewriter.write(to_clients);
    }
}

pub fn prepare_grid_locked_movement(
    tiles_at_gpos: Res<TilesAtGpos>,
    blocking_tiles: Query<&WalkSpeedMultIfOnTop, ()>,
    time: Res<Time>,
    server_state: Res<State<ServerState>>,
    mut ewriter: MessageWriter<ToClients<TransformFromServer>>,
    mut query: Query<
        (
            &mut Transform,
            &mut MoveAnimActive,
            Entity,
            &MoveState,
            &DimensionRef,
            &mut QueuedGridMoveDir,
            Has<ControlledLocally>,
            Has<WallPhaser>,
        ),
        (With<GridLockedMovement>,),
    >,
) {
    for (
        mut transform,
        mut move_anim,
        being_ent,
        move_state,
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
        let raw_input = move_state.norm_move_dir;

        let axis_input_dir = normalize_to_axis_dir(raw_input);

        let (mut dir_vec, finish_current_tile_only) = if is_mid_tile {
            let last_move = move_state.norm_move_dir * move_state.speed_magnitude;
            let active_dir = normalize_to_axis_dir(last_move);

            if active_dir == Vec2::ZERO && offset.x.abs() >= offset.y.abs() {
                Vec2::new(1.0, 0.0)
            } else if active_dir == Vec2::ZERO {
                Vec2::new(0.0, 1.0)
            } else {
                active_dir
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
            set_animation_active(&mut move_anim, false);
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
            continue;
        }

        let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
        let distance = move_state.speed_magnitude * delta;
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
                break;
            }
        }

        transform.translation = current_translation;
        if moved {
            set_animation_active(&mut move_anim, true);

            if server_state.get() == &ServerState::Running {
                let to_clients = ToClients {
                    mode: SendMode::BroadcastExcept(ClientId::Server),
                    message: TransformFromServer::new(being_ent, transform.clone(), false),
                };
                ewriter.write(to_clients);
            }
        } else {
            set_animation_active(&mut move_anim, false);
        }
    }
}

pub fn process_speed_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        Entity,
        &AppliedModifiers,
        &mut MoveState,
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
    for (being_ent, applied, mut move_state, controlled_locally) in being_query.iter_mut() {
        let is_client = state.get() != &ClientState::Disconnected;
        if is_client && !controlled_locally {
            continue;
        }

        let mut speed_max: f32 = f32::INFINITY;
        let mut speed_min: f32 = 0.0;
        let mut speed_scale: f32 = 1.0;
        let mut speed_neg_sum: f32 = 0.0;
        let mut slowdown_mitigators_sum: f32 = 0.0;
        let mut speed_sum: f32 = 0.0;

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
        move_state.speed_magnitude = final_speed;
    }
}

pub fn update_facing_dir(
    mut query: Query<(
        &InputDirection,
        &MoveState,
        Option<&QueuedGridMoveDir>,
        &mut CardinalDirection,
    )>,
) {
    for (input_dir, move_state, queued_dir, mut facing_dir) in query.iter_mut() {
        let move_vec = move_state.norm_move_dir * move_state.speed_magnitude;
        let dir_vec = if move_vec != Vec2::ZERO {
            move_vec
        } else if let Some(q) = queued_dir {
            if q.0 != Vec2::ZERO {
                q.0
            } else {
                input_dir.0
            }
        } else {
            input_dir.0
        };

        if dir_vec == Vec2::ZERO {
            continue;
        }

        *facing_dir = if dir_vec.x.abs() > dir_vec.y.abs() || (dir_vec.x.abs() == dir_vec.y.abs()) {
            if dir_vec.x < 0.0 {
                CardinalDirection::West
            } else {
                CardinalDirection::East
            }
        } else {
            if dir_vec.y <= 0.0 {
                CardinalDirection::South
            } else {
                CardinalDirection::North
            }
        };
    }
}
