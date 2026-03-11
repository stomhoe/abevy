use core::f32;

use being_shared::{BodyTreeWeightSum, ComputedLocally, ControlledBy};
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::*;
use common::log_targets::MOVEMENT_SYSTEM;
use game_common::game_common_components::EntityZeroRef;

use ac_input::ac_input_actions::*;
use modifier_shared::modifier_components::*;
use modifier_shared::modifier_types::InvertMovement;
use modifier_shared::modifier_types::WalkSpeed;
use param_sets::BlockingTileParamSet;
use sprite_animation_shared::{BeingChangedMoveState, MoveAnimActive};
use tilemap_shared::*;

use crate::{
    movement_components::*, movement_drift_log::drift_log, movement_log::movement_log,
    movement_messages::*,
};

fn log_role(client_state: &State<ClientState>) -> &'static str {
    if client_state.get() == &ClientState::Connected {
        "client"
    } else {
        "server"
    }
}

fn normalize_to_axis_dir(input: Vec2) -> IVec2 {
    if input == Vec2::ZERO {
        IVec2::ZERO
    } else if input.x.abs() >= input.y.abs() {
        IVec2::new(input.x.signum() as i32, 0)
    } else {
        IVec2::new(0, input.y.signum() as i32)
    }
}

fn ticks_per_tile(speed: f32, delta: f32, dir: IVec2) -> u16 {
    if speed <= 0.0 || delta <= 0.0 || dir == IVec2::ZERO {
        return 0;
    }
    let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
    let distance = if dir.x != 0 { tile_size.x } else { tile_size.y }.max(1.0);
    ((distance / (speed * delta)).ceil() as u16).max(1)
}

fn grid_translation(tile_pos: GlobalTilePos, glm: &GridLockedMovement, z: f32) -> Vec3 {
    let origin = GlobalTilePos(glm.visual_origin_tile).to_translation(z);
    if !glm.is_stepping() || glm.step_ticks_total == 0 {
        return tile_pos.to_translation(z);
    }
    let t = (glm.progress_ticks as f32 / glm.step_ticks_total as f32).clamp(0.0, 1.0);
    let target = tile_pos.to_translation(z);
    origin.lerp(target, t)
}

fn move_anim_changed(
    being_ent: Entity,
    move_anim: &mut MoveAnimActive,
    active: bool,
    messages: &mut HashSet<BeingChangedMoveState>,
) {
    move_anim.set(active, being_ent, messages);
}

pub fn do_free_movement(
    mut query: Query<
        (Entity, &mut Transform, &mut MoveAnimActive, &MoveVecMag),
        Without<GridLockedMovement>,
    >,
    time: Res<Time>,
    mut writer: MessageWriter<BeingChangedMoveState>,
) {
    let mut move_anim_msgs = HashSet::new();
    for (being_ent, mut transform, mut move_anim, move_state) in query.iter_mut() {
        let velocity = move_state.norm_move_dir * move_state.speed_magnitude;
        if velocity != Vec2::ZERO {
            move_anim_changed(being_ent, &mut move_anim, true, &mut move_anim_msgs);
            transform.translation += (velocity * time.delta_secs()).extend(0.0);
        } else {
            move_anim_changed(being_ent, &mut move_anim, false, &mut move_anim_msgs);
        }
    }
    writer.write_batch(move_anim_msgs);
}

pub fn server_receive_move_inputs(
    mut reader: MessageReader<FromClient<SendMoveInput>>,
    client_state: Res<State<ClientState>>,
    time: Res<Time>,
    mut param_set: ParamSet<(
        BlockingTileParamSet,
        Query<(
            Entity,
            &DimensionRef,
            &mut GlobalTilePos,
            &mut GridLockedMovement,
            &MoveVecMag,
            &mut CardinalDirection,
            Option<&mut LastProcessedMoveInputSeq>,
            &ControlledBy,
        )>,
    )>,
    mut commands: Commands,
    mut writer: MessageWriter<ToClients<GridMoveStateAck>>,
    mut to_drain: Local<Vec<Entity>>,
    mut messages: Local<Vec<ToClients<GridMoveStateAck>>>,
) {
    let role = log_role(&client_state);
    for from_client in reader.read() {
        let Some(client_entity) = from_client.client_id.entity() else {
            continue;
        };
        drift_log(
            role,
            &format!(
                "t={:.3} recv_step client={:?} ent={:?} seq={} dir=({}, {})",
                time.elapsed_secs(),
                client_entity,
                from_client.message.being_ent,
                from_client.message.input_seq,
                from_client.message.dir.x,
                from_client.message.dir.y
            ),
        );
        movement_log(
            role,
            &format!(
                "t={:.3} recv_step client={:?} ent={:?} seq={} dir=({}, {})",
                time.elapsed_secs(),
                client_entity,
                from_client.message.being_ent,
                from_client.message.input_seq,
                from_client.message.dir.x,
                from_client.message.dir.y
            ),
        );
        let (
            being_ent,
            dim_ref,
            prev_seq,
            move_speed,
            controlled_by_client_ent,
            mut tile_pos_snapshot,
            mut glm_snapshot,
            mut facing_snapshot,
            had_last_processed_seq,
        ) = {
            let mut beings = param_set.p1();
            let Ok((
                being_ent,
                &dim_ref,
                tile_pos,
                mut glm,
                move_state,
                facing_dir,
                last_processed_seq,
                controlled_by,
            )) = beings.get_mut(from_client.message.being_ent)
            else {
                continue;
            };
            glm.ensure_grid_anchor(*tile_pos);
            (
                being_ent,
                dim_ref,
                last_processed_seq.as_ref().map_or(0, |seq| seq.0),
                move_state.speed_magnitude,
                controlled_by.client_ent,
                *tile_pos,
                *glm,
                *facing_dir,
                last_processed_seq.is_some(),
            )
        };
        if controlled_by_client_ent != client_entity || from_client.message.input_seq <= prev_seq {
            drift_log(
                role,
                &format!(
                    "t={:.3} drop_step ent={:?} client={:?} seq={} prev_seq={} owner_match={}",
                    time.elapsed_secs(),
                    being_ent,
                    client_entity,
                    from_client.message.input_seq,
                    prev_seq,
                    controlled_by_client_ent == client_entity
                ),
            );
            movement_log(
                role,
                &format!(
                    "t={:.3} drop_step ent={:?} client={:?} seq={} prev_seq={} owner_match={}",
                    time.elapsed_secs(),
                    being_ent,
                    client_entity,
                    from_client.message.input_seq,
                    prev_seq,
                    controlled_by_client_ent == client_entity
                ),
            );
            continue;
        }
        let dir = from_client.message.dir;
        let mut accepted = false;
        if dir != IVec2::ZERO {
            accepted = glm_snapshot.try_start_step(
                &param_set.p0(),
                &mut to_drain,
                dim_ref,
                being_ent,
                &mut tile_pos_snapshot,
                dir,
                ticks_per_tile(move_speed, time.delta_secs(), dir),
            );
            if accepted {
                facing_snapshot = CardinalDirection::from_dir_vec(dir);
            }
        }
        drift_log(
            role,
            &format!(
                "t={:.3} server_step_result ent={:?} seq={} accepted={} tile=({}, {}) origin=({}, {}) dir=({}, {}) ticks={}/{}",
                time.elapsed_secs(),
                being_ent,
                from_client.message.input_seq,
                accepted,
                tile_pos_snapshot.0.x,
                tile_pos_snapshot.0.y,
                glm_snapshot.visual_origin_tile.x,
                glm_snapshot.visual_origin_tile.y,
                glm_snapshot.step_dir.x,
                glm_snapshot.step_dir.y,
                glm_snapshot.progress_ticks,
                glm_snapshot.step_ticks_total
            ),
        );
        movement_log(
            role,
            &format!(
                "t={:.3} server_step_result ent={:?} seq={} accepted={} tile=({}, {}) dir=({}, {}) ticks={}/{}",
                time.elapsed_secs(),
                being_ent,
                from_client.message.input_seq,
                accepted,
                tile_pos_snapshot.0.x,
                tile_pos_snapshot.0.y,
                glm_snapshot.step_dir.x,
                glm_snapshot.step_dir.y,
                glm_snapshot.progress_ticks,
                glm_snapshot.step_ticks_total
            ),
        );
        {
            let mut beings = param_set.p1();
            let Ok((_, _, mut tile_pos, mut glm, _, mut facing_dir, last_processed_seq, _)) =
                beings.get_mut(from_client.message.being_ent)
            else {
                continue;
            };
            *tile_pos = tile_pos_snapshot;
            *glm = glm_snapshot;
            *facing_dir = facing_snapshot;
            if let Some(mut last_processed_seq) = last_processed_seq {
                last_processed_seq.0 = from_client.message.input_seq;
            } else if !had_last_processed_seq {
                commands
                    .entity(being_ent)
                    .insert(LastProcessedMoveInputSeq(from_client.message.input_seq));
            }
        }
        let owner_message = GridMoveStateAck {
            being_ent,
            tile_pos: tile_pos_snapshot.0,
            visual_origin_tile: glm_snapshot.visual_origin_tile,
            step_dir: if accepted {
                glm_snapshot.step_dir
            } else {
                IVec2::ZERO
            },
            progress_ticks: if accepted {
                glm_snapshot.progress_ticks
            } else {
                0
            },
            step_ticks_total: if accepted {
                glm_snapshot.step_ticks_total
            } else {
                0
            },
            facing_dir: facing_snapshot,
            last_processed_input_seq: from_client.message.input_seq,
        };
        messages.push(ToClients {
            mode: SendMode::Direct(ClientId::Client(client_entity)),
            message: owner_message.clone(),
        });
        if accepted {
            messages.push(ToClients {
                mode: SendMode::BroadcastExcept(ClientId::Client(client_entity)),
                message: GridMoveStateAck {
                    last_processed_input_seq: 0,
                    ..owner_message
                },
            });
        }
    }
    writer.write_batch(messages.drain(..));
}

pub fn start_local_predicted_steps(
    client_state: Res<State<ClientState>>,
    time: Res<Time>,
    move_actions: Query<(&Action<BeingMoveAction>, &ActionOf<BeingInputContext>)>,
    mut param_set: ParamSet<(
        BlockingTileParamSet,
        Query<(
            Entity,
            &ControlledBy,
            Has<ComputedLocally>,
            &DimensionRef,
            &MoveVecMag,
            &mut GlobalTilePos,
            &mut GridLockedMovement,
            &mut CardinalDirection,
            Option<&PendingMoveIntents>,
        )>,
    )>,
    mut commands: Commands,
    mut writer: MessageWriter<SendMoveInput>,
    mut next_seq_by_being: Local<EntityHashMap<u32>>,
    mut to_drain: Local<Vec<Entity>>,
    mut messages: Local<Vec<SendMoveInput>>,
) {
    let role = log_role(&client_state);
    for (move_action, action_of) in move_actions.iter() {
        let being_ent = **action_of;
        let (
            entity,
            controlled_locally,
            human_input,
            dim_ref,
            move_speed,
            mut tile_pos_snapshot,
            mut glm_snapshot,
            pending_snapshot,
        ) = {
            let mut beings = param_set.p1();
            let Ok((
                entity,
                controlled_by,
                controlled_locally,
                &dim_ref,
                move_state,
                tile_pos,
                mut glm,
                _facing_dir,
                pending,
            )) = beings.get_mut(being_ent)
            else {
                continue;
            };
            glm.ensure_grid_anchor(*tile_pos);
            (
                entity,
                controlled_locally,
                controlled_by.human_input,
                dim_ref,
                move_state.speed_magnitude,
                *tile_pos,
                *glm,
                pending.cloned().unwrap_or_default(),
            )
        };
        if !controlled_locally || !human_input {
            continue;
        }
        let dir = normalize_to_axis_dir(**move_action);
        if dir == IVec2::ZERO {
            continue;
        }
        drift_log(
            role,
            &format!(
                "t={:.3} local_step_attempt ent={:?} dir=({}, {})",
                time.elapsed_secs(),
                entity,
                dir.x,
                dir.y
            ),
        );
        movement_log(
            role,
            &format!(
                "t={:.3} local_step_attempt ent={:?} dir=({}, {})",
                time.elapsed_secs(),
                entity,
                dir.x,
                dir.y
            ),
        );
        if !glm_snapshot.try_start_step(
            &param_set.p0(),
            &mut to_drain,
            dim_ref,
            entity,
            &mut tile_pos_snapshot,
            dir,
            ticks_per_tile(move_speed, time.delta_secs(), dir),
        ) {
            drift_log(
                role,
                &format!(
                    "t={:.3} local_step_blocked ent={:?} tile=({}, {}) dir=({}, {})",
                    time.elapsed_secs(),
                    entity,
                    tile_pos_snapshot.0.x,
                    tile_pos_snapshot.0.y,
                    dir.x,
                    dir.y
                ),
            );
            movement_log(
                role,
                &format!(
                    "t={:.3} local_step_blocked ent={:?} tile=({}, {}) dir=({}, {})",
                    time.elapsed_secs(),
                    entity,
                    tile_pos_snapshot.0.x,
                    tile_pos_snapshot.0.y,
                    dir.x,
                    dir.y
                ),
            );
            continue;
        }
        {
            let mut beings = param_set.p1();
            let Ok((_, _, _, _, _, mut tile_pos, mut glm, mut facing_dir, _)) =
                beings.get_mut(entity)
            else {
                continue;
            };
            *tile_pos = tile_pos_snapshot;
            *glm = glm_snapshot;
            *facing_dir = CardinalDirection::from_dir_vec(dir);
        }
        let input_seq = next_seq_by_being
            .get(&entity)
            .copied()
            .unwrap_or_default()
            .wrapping_add(1);
        next_seq_by_being.insert(entity, input_seq);
        let mut pending = pending_snapshot;
        pending.0.push(PendingMoveIntent { input_seq, dir });
        commands.entity(entity).insert(pending);
        drift_log(
            role,
            &format!(
                "t={:.3} local_step_started ent={:?} seq={} tile=({}, {}) origin=({}, {}) dir=({}, {}) ticks={}/{}",
                time.elapsed_secs(),
                entity,
                input_seq,
                tile_pos_snapshot.0.x,
                tile_pos_snapshot.0.y,
                glm_snapshot.visual_origin_tile.x,
                glm_snapshot.visual_origin_tile.y,
                glm_snapshot.step_dir.x,
                glm_snapshot.step_dir.y,
                glm_snapshot.progress_ticks,
                glm_snapshot.step_ticks_total
            ),
        );
        movement_log(
            role,
            &format!(
                "t={:.3} local_step_started ent={:?} seq={} tile=({}, {}) dir=({}, {}) ticks={}/{}",
                time.elapsed_secs(),
                entity,
                input_seq,
                tile_pos_snapshot.0.x,
                tile_pos_snapshot.0.y,
                glm_snapshot.step_dir.x,
                glm_snapshot.step_dir.y,
                glm_snapshot.progress_ticks,
                glm_snapshot.step_ticks_total
            ),
        );
        messages.push(SendMoveInput {
            being_ent: entity,
            dir,
            input_seq,
        });
    }
    writer.write_batch(messages.drain(..));
}

pub fn progress_grid_locked_movement(
    client_state: Res<State<ClientState>>,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &GlobalTilePos,
        &mut Transform,
        &mut MoveAnimActive,
        &mut GridLockedMovement,
        Has<ComputedLocally>,
    )>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut messages: Local<HashSet<BeingChangedMoveState>>,
) {
    let is_client = client_state.get() == &ClientState::Connected;
    let role = log_role(&client_state);
    for (being_ent, tile_pos, mut transform, mut move_anim, mut glm, controlled_locally) in
        query.iter_mut()
    {
        if is_client && !controlled_locally {
            // Remote entities animate from authoritative step messages only on clients.
        }
        glm.ensure_grid_anchor(*tile_pos);
        let was_stepping = glm.is_stepping();
        let prev_progress = glm.progress_ticks;
        glm.progress_grid_step(*tile_pos);
        transform.translation = grid_translation(*tile_pos, &glm, transform.translation.z);
        if was_stepping && !glm.is_stepping() {
            drift_log(
                role,
                &format!(
                    "t={:.3} step_complete ent={:?} tile=({}, {}) prev_progress={} final_translation=({:.2}, {:.2}, {:.2})",
                    time.elapsed_secs(),
                    being_ent,
                    tile_pos.0.x,
                    tile_pos.0.y,
                    prev_progress,
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z
                ),
            );
            movement_log(
                role,
                &format!(
                    "t={:.3} step_complete ent={:?} tile=({}, {}) final_translation=({:.2}, {:.2}, {:.2})",
                    time.elapsed_secs(),
                    being_ent,
                    tile_pos.0.x,
                    tile_pos.0.y,
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z
                ),
            );
        }
        move_anim_changed(being_ent, &mut move_anim, glm.is_stepping(), &mut messages);
    }
    writer.write_batch(messages.drain());
}

pub fn apply_grid_move_state_acks(
    client_state: Res<State<ClientState>>,
    time: Res<Time>,
    mut reader: MessageReader<GridMoveStateAck>,
    mut beings: Query<(
        &mut GlobalTilePos,
        &mut Transform,
        &mut GridLockedMovement,
        &mut CardinalDirection,
        Has<ComputedLocally>,
        Option<&mut PendingMoveIntents>,
    )>,
) {
    let role = log_role(&client_state);
    for msg in reader.read() {
        let Ok((mut tile_pos, mut transform, mut glm, mut facing_dir, controlled_locally, pending)) =
            beings.get_mut(msg.being_ent)
        else {
            continue;
        };
        if controlled_locally {
            if let Some(mut pending) = pending {
                pending
                    .0
                    .retain(|intent| intent.input_seq > msg.last_processed_input_seq);
            }
        }
        tile_pos.0 = msg.tile_pos;
        glm.visual_origin_tile = msg.visual_origin_tile;
        glm.step_dir = msg.step_dir;
        glm.progress_ticks = msg.progress_ticks.min(msg.step_ticks_total);
        glm.step_ticks_total = msg.step_ticks_total;
        *facing_dir = msg.facing_dir;
        transform.translation = grid_translation(*tile_pos, &glm, transform.translation.z);
        drift_log(
            role,
            &format!(
                "t={:.3} apply_ack ent={:?} owner_local={} ack_seq={} tile=({}, {}) origin=({}, {}) dir=({}, {}) ticks={}/{} facing={:?}",
                time.elapsed_secs(),
                msg.being_ent,
                controlled_locally,
                msg.last_processed_input_seq,
                tile_pos.0.x,
                tile_pos.0.y,
                glm.visual_origin_tile.x,
                glm.visual_origin_tile.y,
                glm.step_dir.x,
                glm.step_dir.y,
                glm.progress_ticks,
                glm.step_ticks_total,
                *facing_dir
            ),
        );
        movement_log(
            role,
            &format!(
                "t={:.3} apply_ack ent={:?} owner_local={} ack_seq={} tile=({}, {}) dir=({}, {}) ticks={}/{} facing={:?}",
                time.elapsed_secs(),
                msg.being_ent,
                controlled_locally,
                msg.last_processed_input_seq,
                tile_pos.0.x,
                tile_pos.0.y,
                glm.step_dir.x,
                glm.step_dir.y,
                glm.progress_ticks,
                glm.step_ticks_total,
                *facing_dir
            ),
        );
    }
}

pub fn process_speed_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<(
        Entity,
        &DimensionRef,
        &GlobalTilePos,
        &AppliedModifiers,
        &mut MoveVecMag,
        Option<&BodyTreeWeightSum>,
        Has<ComputedLocally>,
    )>,
    modifiers_query: Query<
        (
            Entity,
            &ModifierTarget,
            &CurrEffectiveValue,
            &ApplyMode,
            Has<MitigatingOnly>,
        ),
        With<WalkSpeed>,
    >,
    tile_entity_zero_refs: Query<&EntityZeroRef>,
    tile_walk_speed_mults: Query<&WalkSpeedMultIfOnTop>,
    tile_gathering: TileGatheringParamSet,
    mut entity_vec: Local<Vec<Entity>>,
) {
    for (
        being_ent,
        &dim_ref,
        tile_pos,
        applied,
        mut move_state,
        body_weight_sum,
        controlled_locally,
    ) in being_query.iter_mut()
    {
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
            let Ok((_, _, &CurrEffectiveValue(val), optype, mitigating)) =
                modifiers_query.get(*effect)
            else {
                continue;
            };
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
                ApplyMode::Mul => speed_scale *= val.max(0.0),
                ApplyMode::Min => speed_min = speed_min.max(val),
                ApplyMode::Max => speed_max = speed_max.min(val).max(0.0),
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
        tile_gathering.gather_tiles_at(&mut *entity_vec, dim_ref, *tile_pos);
        for tile_ent in entity_vec.iter() {
            let Ok(tile_cfg_ref) = tile_entity_zero_refs.get(*tile_ent) else {
                continue;
            };
            let Ok(tile_walk_mult_cfg) = tile_walk_speed_mults.get(tile_cfg_ref.0) else {
                continue;
            };
            tile_walk_mult = tile_walk_mult.min(tile_walk_mult_cfg.0);
        }
        let final_speed = final_speed * tile_walk_mult.max(0.0);
        if (move_state.speed_magnitude - final_speed).abs() > f32::EPSILON {
            move_state.speed_magnitude = final_speed * 5000.;
        }
    }
}

pub fn process_input_direction_modifiers(
    state: Res<State<ClientState>>,
    move_actions: Query<&Action<BeingMoveAction>>,
    mut being_query: Query<(
        Entity,
        &AppliedModifiers,
        &Actions<BeingInputContext>,
        &mut MoveVecMag,
        Has<ComputedLocally>,
    )>,
    modifiers_query: Query<(
        Entity,
        &ModifierTarget,
        &CurrEffectiveValue,
        &ApplyMode,
        Has<InvertMovement>,
    )>,
) {
    let is_client = state.get() == &ClientState::Connected;
    for (being_ent, applied, actions, mut move_state, controlled_locally) in being_query.iter_mut()
    {
        if is_client && !controlled_locally {
            continue;
        }
        let Some(move_action) = move_actions.iter_many(actions).next() else {
            continue;
        };
        let input_dir = if controlled_locally {
            **move_action
        } else {
            Vec2::ZERO
        };
        let mut invert_sum: f32 = 0.0;
        let mut invert_scale: f32 = 1.0;
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
            let Ok((_, _, &CurrEffectiveValue(val), optype, invert)) = modifiers_query.get(*effect)
            else {
                continue;
            };
            match optype {
                ApplyMode::Add if invert => invert_sum += val,
                ApplyMode::Mul if invert => invert_scale *= val.max(0.0),
                _ => {}
            }
        }
        move_state.norm_move_dir = if input_dir == Vec2::ZERO {
            Vec2::ZERO
        } else if invert_sum * invert_scale > 1.0 {
            -input_dir.normalize()
        } else {
            input_dir.normalize()
        };
    }
}

pub fn emit_move_state_on_movevecmag_speed_mag_change(
    query: Query<(Entity, &MoveVecMag)>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut prev_by_ent: Local<EntityHashMap<(Vec2, f32)>>,
    mut messages: Local<Vec<BeingChangedMoveState>>,
) {
    for (ent, move_vec_mag) in query.iter() {
        let curr = (move_vec_mag.norm_move_dir, move_vec_mag.speed_magnitude);
        let Some(prev) = prev_by_ent.get(&ent) else {
            prev_by_ent.insert(ent, curr);
            continue;
        };
        if prev.1 != curr.1 {
            messages.push(BeingChangedMoveState(ent));
            prev_by_ent.insert(ent, curr);
        }
    }
    writer.write_batch(messages.drain(..));
}

pub fn update_facing_dir(
    mut query: Query<(
        Entity,
        &MoveVecMag,
        Option<&GridLockedMovement>,
        &mut CardinalDirection,
    )>,
    mut writer: MessageWriter<BeingChangedMoveState>,
    mut messages: Local<HashSet<BeingChangedMoveState>>,
) {
    for (being_ent, move_state, glm, mut facing_dir) in query.iter_mut() {
        let dir = glm
            .and_then(|glm| {
                if glm.step_dir == IVec2::ZERO {
                    None
                } else {
                    Some(glm.step_dir)
                }
            })
            .unwrap_or_else(|| normalize_to_axis_dir(move_state.norm_move_dir));
        if dir == IVec2::ZERO {
            continue;
        }
        let next = CardinalDirection::from_dir_vec(dir);
        if *facing_dir != next {
            *facing_dir = next;
            debug!(target: MOVEMENT_SYSTEM, "Facing updated for {:?} to {:?}", being_ent, next);
            messages.insert(BeingChangedMoveState(being_ent));
        }
    }
    writer.write_batch(messages.drain());
}
