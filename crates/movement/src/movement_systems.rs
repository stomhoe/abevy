use core::f32;

use being_shared::{BodyTreeWeightSum, ComputedLocally, ControlledBy};
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::game_common_components::EntityZeroRef;

use modifier_shared::modifier_types::WalkSpeed;
use modifier_shared::{modifier_components::*, modifier_move_components::*};
use param_sets::BlockingTileParamSet;
use sprite_animation_shared::{MoveAnimActive, BeingChangedMoveState};
use ::tilemap_shared::*;

use crate::{
    movement_components::*,
    movement_drift_log::drift_log,
    movement_messages::*,
};

fn log_role(client_state: &State<ClientState>) -> &'static str {
    if client_state.get() == &ClientState::Connected {
        "client"
    } else {
        "server"
    }
}

fn get_step_origin_tile_and_progress(transform: &Transform, moving_dir: Vec2) -> (GlobalTilePos, f32) {
    let current_tile = GlobalTilePos::from(transform.translation.truncate());
    if moving_dir == Vec2::ZERO {
        return (current_tile, 0.0);
    }

    let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
    let pos = transform.translation.truncate();
    const EPS: f32 = 0.0001;

    if moving_dir.x != 0.0 {
        let tile_coord = pos.x / tile_size.x.max(1.0);
        let floored = tile_coord.floor();
        let frac = tile_coord - floored;
        if moving_dir.x > 0.0 {
            (GlobalTilePos(IVec2::new(floored as i32, current_tile.0.y)), frac.clamp(0.0, 1.0))
        } else if frac <= EPS {
            (GlobalTilePos(IVec2::new(floored as i32, current_tile.0.y)), 0.0)
        } else {
            (
                GlobalTilePos(IVec2::new((floored as i32) + 1, current_tile.0.y)),
                (1.0 - frac).clamp(0.0, 1.0),
            )
        }
    } else {
        let tile_coord = pos.y / tile_size.y.max(1.0);
        let floored = tile_coord.floor();
        let frac = tile_coord - floored;
        if moving_dir.y > 0.0 {
            (GlobalTilePos(IVec2::new(current_tile.0.x, floored as i32)), frac.clamp(0.0, 1.0))
        } else if frac <= EPS {
            (GlobalTilePos(IVec2::new(current_tile.0.x, floored as i32)), 0.0)
        } else {
            (
                GlobalTilePos(IVec2::new(current_tile.0.x, (floored as i32) + 1)),
                (1.0 - frac).clamp(0.0, 1.0),
            )
        }
    }
}

fn normalize_to_axis_dir(input: Vec2) -> Vec2 {
    if input == Vec2::ZERO {
        Vec2::ZERO
    } else if input.x.abs() >= input.y.abs() {
        Vec2::new(input.x.signum(), 0.0)
    } else {
        Vec2::new(0.0, input.y.signum())
    }
}

pub fn do_free_movement(
    mut query: Query<
        (Entity, &mut Transform, &mut MoveAnimActive, &MoveVecMag),
        (Without<GridLockedMovement>,),
    >,
    time: Res<Time>,
    mut writer: MessageWriter<BeingChangedMoveState>,
) {
    let mut move_anim_msgs = HashSet::new();
    for (being_ent, mut transform, mut move_anim, move_state) in query.iter_mut() {
        let velocity = move_state.norm_move_dir * move_state.speed_magnitude;
        if velocity != Vec2::ZERO {
            move_anim.set(true, being_ent, &mut move_anim_msgs);
            transform.translation += (velocity * time.delta_secs()).extend(0.0);
        } else {
            move_anim.set(false, being_ent, &mut move_anim_msgs);
        }
    }
    writer.write_batch(move_anim_msgs);
}

pub fn send_transforms_to_clients(
    query: Query<
        (Entity, &Transform, &MoveVecMag, Option<&LastProcessedMoveInputSeq>, Option<&ControlledBy>),
        (Without<GridLockedMovement>, Changed<Transform>),
    >,
    server_state: Res<State<ServerState>>,
    mut ewriter: MessageWriter<ToClients<UnreliableTransform>>,
) {
    if server_state.get() != &ServerState::Running {
        return;
    }

    for (being_ent, transform, _, last_processed_seq, controlled_by) in query.iter() {
        let to_clients = ToClients {
            mode: controlled_by.map_or(
                SendMode::BroadcastExcept(ClientId::Server),
                |controller| SendMode::BroadcastExcept(ClientId::Client(controller.client_ent)),
            ),
            message: UnreliableTransform::new(
                being_ent,
                transform.clone(),
                last_processed_seq.map_or(0, |seq| seq.0),
            ),
        };
        ewriter.write(to_clients);
    }
}

pub fn send_grid_move_state_acks(
    query: Query<
        (
            Entity,
            &Transform,
            &GridLockedMovement,
            Option<&LastProcessedMoveInputSeq>,
            Option<&ControlledBy>,
        ),
        Changed<Transform>,
    >,
    server_state: Res<State<ServerState>>,
    mut ewriter: MessageWriter<ToClients<GridMoveStateAck>>,
    mut messages: Local<Vec<ToClients<GridMoveStateAck>>>,
) {
    if server_state.get() != &ServerState::Running {
        return;
    }

    for (being_ent, transform, glm, last_processed_seq, controlled_by) in query.iter() {
        let (origin_tile, step_progress) =
            get_step_origin_tile_and_progress(transform, glm.active_move_dir);
        let mode = controlled_by.map_or(
            SendMode::BroadcastExcept(ClientId::Server),
            |controller| SendMode::Direct(ClientId::Client(controller.client_ent)),
        );
        messages.push(ToClients {
            mode,
            message: GridMoveStateAck {
                being_ent,
                tile_pos: origin_tile.0.as_vec2(),
                moving_dir: glm.active_move_dir,
                step_progress,
                last_processed_input_seq: last_processed_seq.map_or(0, |seq| seq.0),
            },
        });
    }

    ewriter.write_batch(messages.drain(..));
}

pub fn prepare_grid_locked_movement(
    blocking_tiles: BlockingTileParamSet,
    time: Res<Time>,
    client_state: Res<State<ClientState>>,
    mut tsf_from_server_writer: MessageWriter<ToClients<UnreliableTransform>>,
    mut beings_changed_anim_move_state_writer: MessageWriter<BeingChangedMoveState>,
    mut being_changed_state_set: Local<HashSet<BeingChangedMoveState>>,

    mut query: Query<
        (
            &mut Transform,
            &mut MoveAnimActive,
            Entity,
            &MoveVecMag,
            &DimensionRef,
            &mut GridLockedMovement,
            Option<&LastProcessedMoveInputSeq>,
            Option<&ControlledBy>,
            Has<ComputedLocally>,
        ),
    >,
    mut to_drain: Local<Vec<Entity>>,
    mut transforms_to_send: Local<Vec<ToClients<UnreliableTransform>>>,
) {
    being_changed_state_set.reserve(query.iter().size_hint().0);

    let is_client = client_state.get() == &ClientState::Connected;

    for (
        mut transform,
        mut move_anim,
        being_ent,
        move_state,
        &dim_ref,
        mut glm,
        last_processed_seq,
        controlled_by,
        controlled_locally,
    ) in query.iter_mut()
    {
        if !controlled_locally && is_client {
            continue;
        }
        let snapped_translation: Vec3 = GlobalTilePos::from(transform.translation.truncate())
            .to_translation(transform.translation.z);
        let offset = transform.translation - snapped_translation;
        let is_mid_tile = offset.x.abs() > 0.01 || offset.y.abs() > 0.01;
        let raw_input = move_state.norm_move_dir;


        let axis_input_dir = normalize_to_axis_dir(raw_input);

        let (mut dir_vec, finish_current_tile_only) = if is_mid_tile {
            let mut active_dir = glm.active_move_dir;
            if active_dir == Vec2::ZERO {
                active_dir = axis_input_dir;
            }
            if active_dir == Vec2::ZERO {
                if offset.x.abs() >= offset.y.abs() {
                    active_dir = Vec2::new(offset.x.signum(), 0.0);
                } else {
                    active_dir = Vec2::new(0.0, offset.y.signum());
                }
            }

            if axis_input_dir == Vec2::ZERO {
                if active_dir != Vec2::ZERO {
                    glm.active_move_dir = active_dir;
                }
                (active_dir, true)
            } else if active_dir == Vec2::ZERO {
                glm.active_move_dir = axis_input_dir;
                (axis_input_dir, false)
            } else if axis_input_dir == active_dir {
                glm.active_move_dir = active_dir;
                (active_dir, false)
            } else if axis_input_dir == -active_dir {
                glm.queued_move_dir = Vec2::ZERO;
                glm.active_move_dir = axis_input_dir;
                (axis_input_dir, false)
            } else {
                glm.queued_move_dir = axis_input_dir;
                glm.active_move_dir = active_dir;
                (active_dir, true)
            }
        } else if glm.queued_move_dir != Vec2::ZERO {
            let queued = glm.queued_move_dir;
            glm.queued_move_dir = Vec2::ZERO;
            glm.active_move_dir = queued;
            (queued, false)
        } else {
            if axis_input_dir != Vec2::ZERO {
                glm.active_move_dir = axis_input_dir;
            } else {
                glm.active_move_dir = Vec2::ZERO;
            }
            (axis_input_dir, false)
        };
        if dir_vec == Vec2::ZERO {
            transform.translation = snapped_translation;
            move_anim.set(false, being_ent, &mut being_changed_state_set);
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
            if blocking_tiles.is_blocked_at(&mut to_drain, dim_ref, GlobalTilePos::from(next_target), being_ent) {
                glm.queued_move_dir = Vec2::ZERO;
                glm.active_move_dir = Vec2::ZERO;
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
                let target = current_snapped.xy() + (current_dir * step_distance);
                if blocking_tiles.is_blocked_at(&mut to_drain, dim_ref, GlobalTilePos::from(target), being_ent) {
                    glm.queued_move_dir = Vec2::ZERO;
                    glm.active_move_dir = Vec2::ZERO;
                    break;
                }
                remaining_to_boundary = step_distance;
            }

            let step = remaining_distance.min(remaining_to_boundary);
            let will_reach_boundary = step >= remaining_to_boundary - 0.0001;

            if will_reach_boundary {
                let target = current_snapped.xy() + (current_dir * step_distance);
                if blocking_tiles.is_blocked_at(&mut to_drain, dim_ref, GlobalTilePos::from(target), being_ent) {
                    glm.queued_move_dir = Vec2::ZERO;
                    glm.active_move_dir = Vec2::ZERO;
                    if !moved {
                        current_translation = current_snapped;
                    }
                    break;
                }

                current_translation += (current_dir * remaining_to_boundary).extend(0.0);
                remaining_distance -= remaining_to_boundary;
                moved |= remaining_to_boundary > 0.0;
                //client never reaches this point

                if finish_current_tile_only && glm.queued_move_dir == Vec2::ZERO {
                    glm.active_move_dir = Vec2::ZERO;
                    break;
                }

                if glm.queued_move_dir != Vec2::ZERO {
                    current_dir = glm.queued_move_dir;
                    glm.queued_move_dir = Vec2::ZERO;
                    glm.active_move_dir = current_dir;
                }
            } else {
                current_translation += (current_dir * step).extend(0.0);
                moved |= step > 0.0;
                break;
            }
        }

        transform.translation = current_translation;
        if moved {
            move_anim.set(true, being_ent, &mut being_changed_state_set);
            if !is_client {
                let to_clients = ToClients {
                    mode: controlled_by.map_or(
                        SendMode::BroadcastExcept(ClientId::Server),
                        |controller| SendMode::BroadcastExcept(ClientId::Client(controller.client_ent)),
                    ),
                    message: UnreliableTransform::new(
                        being_ent,
                        transform.clone(),
                        last_processed_seq.map_or(0, |seq| seq.0),
                    ),
                };
                transforms_to_send.push(to_clients);
            }
        } else {
            move_anim.set(false, being_ent, &mut being_changed_state_set);
        }
    }
    beings_changed_anim_move_state_writer.write_batch(being_changed_state_set.drain());
    tsf_from_server_writer.write_batch(transforms_to_send.drain(..));
}

#[allow(unused_parens)]
pub fn set_transforms_to_received(
    mut reader: MessageReader<UnreliableTransform>,
    client_state: Res<State<ClientState>>,
    mut query: Query<&mut Transform>,
    mut last_ack_by_being: Local<EntityHashMap<u32>>,
) {
    let role = log_role(&client_state);
    for msg in reader.read() {
        if last_ack_by_being
            .get(&msg.being_ent)
            .is_some_and(|ack| msg.last_processed_input_seq < *ack)
        {
            drift_log(
                role,
                &format!(
                    "drop_out_of_order_transform ent={:?} ack_seq={} last_ack_seq={}",
                    msg.being_ent,
                    msg.last_processed_input_seq,
                    last_ack_by_being
                        .get(&msg.being_ent)
                        .copied()
                        .unwrap_or_default()
                ),
            );
            continue;
        }
        let Ok(mut transf) = query.get_mut(msg.being_ent)
        else {continue;};
        *transf = msg.trans;
        last_ack_by_being.insert(msg.being_ent, msg.last_processed_input_seq);
    }
}

pub fn reconcile_grid_move_state_acks(
    mut reader: MessageReader<GridMoveStateAck>,
    client_state: Res<State<ClientState>>,
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &mut GridLockedMovement,
        Option<&mut PendingMoveIntents>,
        Has<ComputedLocally>,
    )>,
    mut last_ack_by_being: Local<EntityHashMap<u32>>,
    mut last_sample_log_by_being: Local<EntityHashMap<f32>>,
) {
    let role = log_role(&client_state);
    const DRIFT_SAMPLE_SECS: f32 = 0.35;
    const HARD_SNAP_DRIFT: f32 = 48.0;
    const SOFT_CORRECTION_START: f32 = 1.5;
    let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
    let now = time.elapsed_secs();
    for msg in reader.read() {
        let Ok((mut transform, mut glm, pending_intents, controlled_locally)) =
            query.get_mut(msg.being_ent)
        else {
            continue;
        };
        if !controlled_locally {
            continue;
        }
        if last_ack_by_being
            .get(&msg.being_ent)
            .is_some_and(|ack| msg.last_processed_input_seq < *ack)
        {
            continue;
        }

        let authoritative_origin_tile = GlobalTilePos(msg.tile_pos.as_ivec2());
        let authoritative_translation =
            authoritative_origin_tile.to_translation(transform.translation.z)
                + (msg.moving_dir * msg.step_progress * tile_size).extend(0.0);
        let authoritative_current_tile = GlobalTilePos::from(authoritative_translation.truncate());
        let local_tile = GlobalTilePos::from(transform.translation.truncate());
        let tile_mismatch = local_tile != authoritative_current_tile;

        let pending_count;
        if let Some(mut pending_intents) = pending_intents {
            pending_intents
                .0
                .retain(|intent| intent.input_seq > msg.last_processed_input_seq);
            pending_count = pending_intents.0.len();
            if let Some(last_intent) = pending_intents.0.last().copied() {
                glm.queued_move_dir = last_intent.dir;
            } else {
                glm.queued_move_dir = Vec2::ZERO;
            }
        } else {
            pending_count = 0;
            glm.queued_move_dir = Vec2::ZERO;
        }

        glm.active_move_dir = msg.moving_dir;
        let drift_dist = transform.translation.distance(authoritative_translation);
        let hard_snap = tile_mismatch || drift_dist >= HARD_SNAP_DRIFT;
        if hard_snap {
            drift_log(
                role,
                &format!(
                    "t={:.3} snap ent={:?} ack_seq={} tile_mismatch={} drift={:.3} local_tile={:?} server_tile={:?} from=({:.2},{:.2}) to=({:.2},{:.2}) dir=({:.0},{:.0}) step={:.3}",
                    now,
                    msg.being_ent,
                    msg.last_processed_input_seq,
                    tile_mismatch,
                    drift_dist,
                    local_tile,
                    authoritative_current_tile,
                    transform.translation.x,
                    transform.translation.y,
                    authoritative_translation.x,
                    authoritative_translation.y,
                    msg.moving_dir.x,
                    msg.moving_dir.y,
                    msg.step_progress
                ),
            );
            transform.translation = authoritative_translation;
        } else if drift_dist >= SOFT_CORRECTION_START {
            let alpha = (drift_dist / tile_size.max_element()).clamp(0.12, 0.40);
            let corrected = transform.translation.lerp(authoritative_translation, alpha);
            drift_log(
                role,
                &format!(
                    "t={:.3} soft_correct ent={:?} ack_seq={} drift={:.3} alpha={:.3} local_tile={:?} server_tile={:?} from=({:.2},{:.2}) to=({:.2},{:.2})",
                    now,
                    msg.being_ent,
                    msg.last_processed_input_seq,
                    drift_dist,
                    alpha,
                    local_tile,
                    authoritative_current_tile,
                    transform.translation.x,
                    transform.translation.y,
                    corrected.x,
                    corrected.y
                ),
            );
            transform.translation = corrected;
        } else if last_sample_log_by_being
            .get(&msg.being_ent)
            .is_none_or(|last| now - *last >= DRIFT_SAMPLE_SECS)
            && (msg.moving_dir != Vec2::ZERO || pending_count > 0 || drift_dist > 0.25)
        {
            drift_log(
                role,
                &format!(
                    "t={:.3} sample ent={:?} ack_seq={} drift={:.3} local_tile={:?} server_tile={:?} pos=({:.2},{:.2}) auth=({:.2},{:.2}) pending={}",
                    now,
                    msg.being_ent,
                    msg.last_processed_input_seq,
                    drift_dist,
                    local_tile,
                    authoritative_current_tile,
                    transform.translation.x,
                    transform.translation.y,
                    authoritative_translation.x,
                    authoritative_translation.y,
                    pending_count
                ),
            );
            last_sample_log_by_being.insert(msg.being_ent, now);
        }
        last_ack_by_being.insert(msg.being_ent, msg.last_processed_input_seq);
    }
}

pub fn process_speed_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        Entity,
        &Transform,
        &DimensionRef,
        &AppliedModifiers,
        &mut MoveVecMag,
        Option<&BodyTreeWeightSum>,
        Has<ComputedLocally>,
    )>,
    modifiers_query: Query<
        (
            Entity, &ModifierTarget, &CurrEffectiveValue,
            &ApplyMode, Has<MitigatingOnly>,
        ),
        (With<WalkSpeed>,),
    >,
    tile_entity_zero_refs: Query<&EntityZeroRef>,
    tile_walk_speed_mults: Query<&WalkSpeedMultIfOnTop>,
    tile_gathering: TileGatheringParamSet,
    mut entity_vec: Local<Vec<Entity>>,
) {

    for (being_ent, transform, &dim_ref, applied, mut move_state, body_weight_sum, controlled_locally) in being_query.iter_mut() {
        let is_client = state.get() == &ClientState::Connected;
        if is_client && !controlled_locally {
            continue;
        }

        let mut speed_max: f32 = f32::INFINITY;
        let mut speed_min: f32 = 0.0;
        let mut speed_scale: f32 = 1.0;
        let mut speed_substractors_sum: f32 = 0.0;
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
                            speed_substractors_sum += val;
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
        speed_sum += speed_substractors_sum + slowdown_mitigators_sum;

        let total_weight_newtons = body_weight_sum
            .map(|sum| sum.0)
            .unwrap_or_default()
            .max(1.0);
        let mut final_speed = (speed_sum * speed_scale)
            .max(speed_min)
            .min(speed_max)
            .max(0.0);
        final_speed /= total_weight_newtons;

        let mut tile_walk_mult: f32 = 1.0;
        entity_vec.clear();
        tile_gathering.gather_tiles_at(&mut *entity_vec, dim_ref, GlobalTilePos::from(transform.translation.truncate()));
        for tile_ent in entity_vec.iter() {
            let Ok(tile_cfg_ref) = tile_entity_zero_refs.get(*tile_ent) else { continue };
            let Ok(tile_walk_mult_cfg) = tile_walk_speed_mults.get(tile_cfg_ref.0) else { continue };
            tile_walk_mult = tile_walk_mult.min(tile_walk_mult_cfg.0);
        }
        let final_speed = final_speed * tile_walk_mult.max(0.0);
        if (move_state.speed_magnitude - final_speed).abs() > f32::EPSILON {
            move_state.speed_magnitude = final_speed * 5000.;
        }
    }
}

pub fn emit_move_state_on_movevecmag_value_change(
    query: Query<(Entity, &MoveVecMag)>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut prev_by_ent: Local<EntityHashMap<(Vec2, f32)>>,
    mut messages: Local<Vec<BeingChangedMoveState>>,
) {
    let mut current_ents = EntityHashSet::with_capacity(query.iter().size_hint().0);

    for (ent, move_vec_mag) in query.iter() {
        current_ents.insert(ent);
        let curr = (move_vec_mag.norm_move_dir, move_vec_mag.speed_magnitude);
        let Some(prev) = prev_by_ent.get(&ent) else {
            prev_by_ent.insert(ent, curr);
            continue;
        };
        if (prev.1 - curr.1).abs() > f32::EPSILON {
            messages.push(BeingChangedMoveState(ent));
            prev_by_ent.insert(ent, curr);
        }
    }

    prev_by_ent.retain(|ent, _| current_ents.contains(ent));
    writer.write_batch(messages.drain(..));
}

pub fn update_facing_dir(
    mut query: Query<(
        Entity,
        &MoveVecMag,
        Option<&GridLockedMovement>,
        &mut CardinalDirection,
    )>,
    mut beings_changed_anim_move_state_writer: MessageWriter<BeingChangedMoveState>,
    mut being_changed_state_set: Local<HashSet<BeingChangedMoveState>>,
) {
    being_changed_state_set.reserve(query.iter().size_hint().0);
    for (being_ent, move_state, glm, mut facing_dir) in query.iter_mut() {
        let move_vec = move_state.norm_move_dir * move_state.speed_magnitude;
        let dir_vec = if let Some(glm) = glm {
            if glm.active_move_dir != Vec2::ZERO {
                glm.active_move_dir
            } else if move_vec != Vec2::ZERO {
                move_vec
            } else {
                Vec2::ZERO
            }
        } else {
            if move_vec != Vec2::ZERO {
                move_vec
            } else {
                Vec2::ZERO
            }
        };

        if dir_vec == Vec2::ZERO {
            continue;
        }

        let new_dir = if dir_vec.x.abs() > dir_vec.y.abs() || (dir_vec.x.abs() == dir_vec.y.abs()) {
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
        if new_dir != *facing_dir {
            *facing_dir = new_dir;
            being_changed_state_set.insert(BeingChangedMoveState(being_ent));
        }
    }
    beings_changed_anim_move_state_writer.write_batch(being_changed_state_set.drain());
}
