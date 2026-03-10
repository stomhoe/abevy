use core::f32;

use being_shared::{BodyTreeWeightSum, ComputedLocally, ControlledBy};
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::log_targets::MOVEMENT_SYSTEM;
use game_common::game_common_components::EntityZeroRef;

use modifier_shared::modifier_types::WalkSpeed;
use modifier_shared::modifier_components::*;
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

fn normalize_to_axis_dir(input: Vec2) -> Vec2 {
    if input == Vec2::ZERO {
        Vec2::ZERO
    } else if input.x.abs() >= input.y.abs() {
        Vec2::new(input.x.signum(), 0.0)
    } else {
        Vec2::new(0.0, input.y.signum())
    }
}

fn translation_differs(a: Vec3, b: Vec3) -> bool {
    (a.x - b.x).abs() > 0.01 || (a.y - b.y).abs() > 0.01 || (a.z - b.z).abs() > 0.01
}

fn grid_progress_ratio(glm: &GridLockedMovement) -> f32 {
    if glm.progress_ticks == 0 || glm.step_ticks_total == 0 {
        0.0
    } else {
        (glm.progress_ticks as f32 / glm.step_ticks_total as f32).clamp(0.0, 1.0)
    }
}

fn ticks_per_tile(speed: f32, delta: f32, dir: Vec2) -> u16 {
    if speed <= 0.0 || delta <= 0.0 || dir == Vec2::ZERO {
        return 0;
    }
    let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
    let distance = if dir.x != 0.0 { tile_size.x } else { tile_size.y }.max(1.0);
    ((distance / (speed * delta)).ceil() as u16).max(1)
}

fn next_step_dir(glm: &mut GridLockedMovement, axis_input_dir: Vec2, finish_current_tile_only: bool) -> Vec2 {
    if glm.queued_move_dir != Vec2::ZERO {
        let queued = glm.queued_move_dir;
        glm.queued_move_dir = Vec2::ZERO;
        queued
    } else if !finish_current_tile_only && axis_input_dir != Vec2::ZERO {
        axis_input_dir
    } else {
        Vec2::ZERO
    }
}

fn step_grid_motor(
    blocking_tiles: &BlockingTileParamSet,
    to_drain: &mut Vec<Entity>,
    dim_ref: DimensionRef,
    being_ent: Entity,
    glm: &mut GridLockedMovement,
    axis_input_dir: Vec2,
    speed: f32,
    delta: f32,
) -> bool {
    let mut finish_current_tile_only = false;
    if glm.progress_ticks > 0 && glm.active_move_dir != Vec2::ZERO {
        if axis_input_dir == Vec2::ZERO {
            glm.queued_move_dir = Vec2::ZERO;
            finish_current_tile_only = true;
        } else if axis_input_dir == glm.active_move_dir {
            glm.queued_move_dir = Vec2::ZERO;
        } else {
            glm.queued_move_dir = axis_input_dir;
            finish_current_tile_only = true;
        }
    } else {
        glm.progress_ticks = 0;
        glm.step_ticks_total = 0;
        glm.active_move_dir = if axis_input_dir != Vec2::ZERO {
            axis_input_dir
        } else if glm.queued_move_dir != Vec2::ZERO {
            let queued = glm.queued_move_dir;
            glm.queued_move_dir = Vec2::ZERO;
            queued
        } else {
            Vec2::ZERO
        };
        if axis_input_dir != Vec2::ZERO {
            glm.queued_move_dir = Vec2::ZERO;
        }
    }
    if glm.active_move_dir == Vec2::ZERO || speed <= 0.0 || delta <= 0.0 {
        glm.progress_ticks = 0;
        glm.step_ticks_total = 0;
        return false;
    }
    if glm.progress_ticks == 0 {
        let next_tile = GlobalTilePos(glm.origin_tile + glm.active_move_dir.as_ivec2());
        if blocking_tiles.is_blocked_at(to_drain, dim_ref, next_tile, being_ent) {
            glm.queued_move_dir = Vec2::ZERO;
            glm.active_move_dir = Vec2::ZERO;
            glm.progress_ticks = 0;
            glm.step_ticks_total = 0;
            return false;
        }
        glm.step_ticks_total = ticks_per_tile(speed, delta, glm.active_move_dir);
        if glm.step_ticks_total == 0 {
            glm.queued_move_dir = Vec2::ZERO;
            glm.active_move_dir = Vec2::ZERO;
            return false;
        }
    }
    glm.progress_ticks = glm.progress_ticks.saturating_add(1).min(glm.step_ticks_total.max(1));
    if glm.progress_ticks < glm.step_ticks_total {
        return true;
    }
    glm.origin_tile += glm.active_move_dir.as_ivec2();
    glm.progress_ticks = 0;
    glm.step_ticks_total = 0;
    glm.active_move_dir = next_step_dir(glm, axis_input_dir, finish_current_tile_only);
    if glm.active_move_dir == Vec2::ZERO {
        return true;
    }
    let next_tile = GlobalTilePos(glm.origin_tile + glm.active_move_dir.as_ivec2());
    if blocking_tiles.is_blocked_at(to_drain, dim_ref, next_tile, being_ent) {
        glm.queued_move_dir = Vec2::ZERO;
        glm.active_move_dir = Vec2::ZERO;
    }
    true
}

fn grid_translation(glm: &GridLockedMovement, z: f32) -> Vec3 {
    let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
    GlobalTilePos(glm.origin_tile).to_translation(z)
        + (glm.active_move_dir * grid_progress_ratio(glm) * tile_size).extend(0.0)
}

fn sync_grid_origin_from_transform(transform: &Transform, glm: &mut GridLockedMovement) {
    if glm.progress_ticks > 0 || glm.active_move_dir != Vec2::ZERO {
        return;
    }
    let tile = GlobalTilePos::from(transform.translation.truncate());
    if glm.origin_tile != tile.0 {
        glm.origin_tile = tile.0;
        glm.queued_move_dir = Vec2::ZERO;
    }
}

fn replay_predicted_grid_movement(
    blocking_tiles: &BlockingTileParamSet,
    to_drain: &mut Vec<Entity>,
    dim_ref: DimensionRef,
    being_ent: Entity,
    glm: &mut GridLockedMovement,
    axis_input_dir: Vec2,
    speed: f32,
    delta: f32,
) -> bool {
    step_grid_motor(
        blocking_tiles,
        to_drain,
        dim_ref,
        being_ent,
        glm,
        axis_input_dir,
        speed,
        delta,
    )
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
        (Entity, &Transform, &MoveVecMag, Option<&LastAppliedMoveInputSeq>, Option<&ControlledBy>),
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
    mut query: Query<
        (
            Entity,
            &Transform,
            &mut GridLockedMovement,
            &CardinalDirection,
            Option<&LastAppliedMoveInputSeq>,
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

    for (being_ent, transform, mut glm, facing_dir, last_processed_seq, controlled_by) in query.iter_mut() {
        sync_grid_origin_from_transform(transform, &mut glm);
        let mode = controlled_by.map_or(
            SendMode::BroadcastExcept(ClientId::Server),
            |controller| SendMode::Direct(ClientId::Client(controller.client_ent)),
        );
        messages.push(ToClients {
            mode,
            message: GridMoveStateAck {
                being_ent,
                tile_pos: glm.origin_tile,
                moving_dir: glm.active_move_dir,
                facing_dir: *facing_dir,
                progress_ticks: glm.progress_ticks,
                step_ticks_total: glm.step_ticks_total,
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
            &CardinalDirection,
            Option<&LastAppliedMoveInputSeq>,
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
        facing_dir,
        last_processed_seq,
        controlled_by,
        controlled_locally,
    ) in query.iter_mut()
    {
        if !controlled_locally && is_client {
            continue;
        }
        sync_grid_origin_from_transform(&transform, &mut glm);
        let raw_input = move_state.norm_move_dir;
        let axis_input_dir = normalize_to_axis_dir(raw_input);
        if (glm.active_move_dir == Vec2::ZERO && glm.progress_ticks == 0 && axis_input_dir == Vec2::ZERO)
            || move_state.speed_magnitude <= 0.0
        {
            glm.progress_ticks = 0;
            glm.step_ticks_total = 0;
            if axis_input_dir == Vec2::ZERO {
                glm.active_move_dir = Vec2::ZERO;
                glm.queued_move_dir = Vec2::ZERO;
            }
            let snapped_translation = GlobalTilePos(glm.origin_tile).to_translation(transform.translation.z);
            if translation_differs(transform.translation, snapped_translation) {
                transform.translation = snapped_translation;
            }
            move_anim.set(false, being_ent, &mut being_changed_state_set);
            continue;
        }
        let delta = time.delta_secs();
        if delta <= 0.0 {
            continue;
        }
        let blocked_check_tile = if glm.progress_ticks == 0 && glm.active_move_dir != Vec2::ZERO {
            Some(GlobalTilePos(glm.origin_tile + glm.active_move_dir.as_ivec2()))
        } else {
            None
        };
        let moved = step_grid_motor(
            &blocking_tiles,
            &mut to_drain,
            dim_ref,
            being_ent,
            &mut glm,
            axis_input_dir,
            move_state.speed_magnitude,
            delta,
        );
        if !moved && blocked_check_tile.is_some() {
            let snapped_translation = GlobalTilePos(glm.origin_tile).to_translation(transform.translation.z);
            let blocked_tile = blocked_check_tile.unwrap();
            drift_log(
                log_role(&client_state),
                &format!(
                    "t={:.3} blocked_idle ent={:?} pos=({:.2},{:.2}) snap=({:.2},{:.2}) face={:?} input=({:.0},{:.0}) active=({:.0},{:.0}) queued=({:.0},{:.0}) blocked_tile={:?}",
                    time.elapsed_secs(),
                    being_ent,
                    transform.translation.x,
                    transform.translation.y,
                    snapped_translation.x,
                    snapped_translation.y,
                    facing_dir,
                    axis_input_dir.x,
                    axis_input_dir.y,
                    glm.active_move_dir.x,
                    glm.active_move_dir.y,
                    glm.queued_move_dir.x,
                    glm.queued_move_dir.y,
                    blocked_tile,
                ),
            );
        }
        let current_translation = grid_translation(&glm, transform.translation.z);
        if translation_differs(transform.translation, current_translation) {
            transform.translation = current_translation;
        }
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
    mut param_set: ParamSet<(
        BlockingTileParamSet,
        Query<(
            &mut Transform,
            &mut GridLockedMovement,
            &mut CardinalDirection,
            &MoveVecMag,
            &DimensionRef,
            Option<&mut PendingMoveIntents>,
            Has<ComputedLocally>,
        )>,
    )>,
    mut last_ack_by_being: Local<EntityHashMap<u32>>,
    mut last_sample_log_by_being: Local<EntityHashMap<f32>>,
    mut to_drain: Local<Vec<Entity>>,
) {
    let role = log_role(&client_state);
    const DRIFT_SAMPLE_SECS: f32 = 0.35;
    let now = time.elapsed_secs();
    let mut latest_by_being = EntityHashMap::default();
    for msg in reader.read() {
        if last_ack_by_being
            .get(&msg.being_ent)
            .is_some_and(|ack| msg.last_processed_input_seq < *ack)
        {
            continue;
        }
        latest_by_being.insert(msg.being_ent, msg.clone());
    }
    for (_, msg) in latest_by_being.drain() {
        let (
            mut authoritative_glm,
            local_translation,
            local_facing_dir,
            move_speed,
            move_input_dir,
            dim_ref,
            pending_count,
            pending_snapshot,
        ) = {
            let mut beings = param_set.p1();
            let Ok((transform, glm, facing_dir, move_state, &dim_ref, pending_intents, controlled_locally)) =
                beings.get_mut(msg.being_ent)
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
            let mut authoritative_glm = *glm;
            authoritative_glm.origin_tile = msg.tile_pos;
            authoritative_glm.active_move_dir = msg.moving_dir;
            authoritative_glm.progress_ticks = msg.progress_ticks.min(msg.step_ticks_total);
            authoritative_glm.step_ticks_total = msg.step_ticks_total;
            let mut pending_snapshot = Vec::new();
            let pending_count;
            if let Some(mut pending_intents) = pending_intents {
                pending_intents
                    .0
                    .retain(|intent| intent.input_seq > msg.last_processed_input_seq);
                pending_count = pending_intents.0.len();
                pending_snapshot.extend(pending_intents.0.iter().copied());
                if let Some(last_intent) = pending_intents.0.last().copied() {
                    authoritative_glm.queued_move_dir = last_intent.dir;
                } else {
                    authoritative_glm.queued_move_dir = Vec2::ZERO;
                }
            } else {
                pending_count = 0;
                authoritative_glm.queued_move_dir = Vec2::ZERO;
            }
            (
                authoritative_glm,
                transform.translation,
                *facing_dir,
                move_state.speed_magnitude,
                move_state.norm_move_dir,
                dim_ref,
                pending_count,
                pending_snapshot,
            )
        };
        let mut replayed_ticks = 0usize;
        if pending_count == 0 && authoritative_glm.active_move_dir == Vec2::ZERO {
            authoritative_glm.queued_move_dir = Vec2::ZERO;
        }
        for intent in pending_snapshot {
            replay_predicted_grid_movement(
                &param_set.p0(),
                &mut to_drain,
                dim_ref,
                msg.being_ent,
                &mut authoritative_glm,
                normalize_to_axis_dir(intent.dir),
                move_speed,
                time.delta_secs(),
            );
            replayed_ticks += 1;
        }
        let authoritative_translation = grid_translation(&authoritative_glm, local_translation.z);
        let authoritative_current_tile = GlobalTilePos::from(authoritative_translation.truncate());
        let local_tile = GlobalTilePos::from(local_translation.truncate());
        let drift_dist = local_translation.distance(authoritative_translation);
        if last_sample_log_by_being
            .get(&msg.being_ent)
            .is_none_or(|last| now - *last >= DRIFT_SAMPLE_SECS)
            && drift_dist > 0.25
        {
            drift_log(
                role,
                &format!(
                    "t={:.3} reconcile ent={:?} ack_seq={} drift={:.3} local_tile={:?} server_tile={:?} from=({:.2},{:.2}) to=({:.2},{:.2}) pending={} replayed={} input=({:.0},{:.0}) local_face={:?} auth_face={:?} active=({:.0},{:.0}) queued=({:.0},{:.0}) step={:.3}",
                    now,
                    msg.being_ent,
                    msg.last_processed_input_seq,
                    drift_dist,
                    local_tile,
                    authoritative_current_tile,
                    local_translation.x,
                    local_translation.y,
                    authoritative_translation.x,
                    authoritative_translation.y,
                    pending_count,
                    replayed_ticks,
                    move_input_dir.x,
                    move_input_dir.y,
                    local_facing_dir,
                    msg.facing_dir,
                    authoritative_glm.active_move_dir.x,
                    authoritative_glm.active_move_dir.y,
                    authoritative_glm.queued_move_dir.x,
                    authoritative_glm.queued_move_dir.y,
                    grid_progress_ratio(&authoritative_glm)
                ),
            );
            last_sample_log_by_being.insert(msg.being_ent, now);
        }
        if drift_dist > 8.0 {
            debug!(
                target: MOVEMENT_SYSTEM,
                "Resetting predicted grid state for {:?}: drift {:.2}, ack_seq {}, pending {}, replayed {}",
                msg.being_ent,
                drift_dist,
                msg.last_processed_input_seq,
                pending_count,
                replayed_ticks
            );
        }
        let mut beings = param_set.p1();
        let Ok((mut transform, mut glm, mut facing_dir, _, _, _, _)) = beings.get_mut(msg.being_ent)
        else {
            continue;
        };
        if *facing_dir != msg.facing_dir {
            *facing_dir = msg.facing_dir;
        }
        *glm = authoritative_glm;
        transform.translation = authoritative_translation;
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
