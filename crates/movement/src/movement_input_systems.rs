use core::f32;

use ::being_shared::*;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::*;

use modifier_shared::{modifier_components::*, modifier_move_components::*, modifier_types::*};
use ac_input::ac_input_actions::*;

use crate::{movement_components::*, movement_drift_log::drift_log, movement_messages::SendMoveInput};

const INPUT_DEADZONE: f32 = 0.2;
const INPUT_CHANGE_EPSILON: f32 = 0.01;
const INPUT_RESEND_SECS: f32 = 0.12;

fn log_role(client_state: &State<ClientState>) -> &'static str {
    if client_state.get() == &ClientState::Connected {
        "client"
    } else {
        "server"
    }
}

fn sanitize_input_dir(input: Vec2) -> Vec2 {
    if input.length() <= INPUT_DEADZONE {
        Vec2::ZERO
    } else {
        let n = input.normalize();
        if n.x.abs() >= n.y.abs() {
            Vec2::new(n.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, n.y.signum())
        }
    }
}

pub fn add_being_input_context(
    mut commands: Commands,
    being_query: Query<
        Entity,
        (
            Or<(With<ComputedLocally>, With<ControlledBy>)>,
            Without<Actions<BeingInputContext>>,
        ),
    >,
) {
    for being_ent in being_query.iter() {
        commands.entity(being_ent).try_insert((
            BeingInputContext,
            actions!(BeingInputContext[
                (
                    Action::<BeingMoveAction>::new(),
                    Down::default(),
                    DeadZone::default(),
                    Bindings::spawn((
                        Cardinal::wasd_keys(),
                        Cardinal::arrows(),
                        Cardinal::dpad(),
                        Axial::left_stick(),
                    )),
                ),
                (
                    Action::<BeingMeleeAttackAction>::new(),
                    bindings![KeyCode::ControlLeft],
                ),
            ]),
        ));
    }
}

pub fn send_move_input_to_server(
    mut commands: Commands,
    mut event_writer: MessageWriter<SendMoveInput>,
    client_state: Res<State<ClientState>>,
    time: Res<Time>,
    move_actions: Query<(&Action<BeingMoveAction>, &ActionOf<BeingInputContext>)>,
    mut controlled_beings: Query<(
        &ControlledBy,
        Has<ComputedLocally>,
        Option<&mut PendingMoveIntents>,
    )>,
    mut last_sent_by_being: Local<EntityHashMap<Vec2>>,
    mut next_seq_by_being: Local<EntityHashMap<u32>>,
    mut client_tick: Local<u32>,
    mut last_send_time_by_being: Local<EntityHashMap<f32>>,
    mut messages: Local<Vec<SendMoveInput>>,
) {
    let role = log_role(&client_state);
    *client_tick = client_tick.wrapping_add(1);
    let mut current_beings = EntityHashSet::default();
    let now = time.elapsed_secs();
    for (move_action, action_of) in move_actions.iter() {
        let being_ent = **action_of;
        let Ok((controlled_by, controlled_locally, pending_intents)) = controlled_beings.get_mut(being_ent) else {
            continue;
        };
        if !controlled_locally || !controlled_by.human_input {
            continue;
        }
        current_beings.insert(being_ent);
        let input_dir = sanitize_input_dir(**move_action);
        let unchanged = last_sent_by_being
            .get(&being_ent)
            .is_some_and(|last_sent| last_sent.distance(input_dir) <= INPUT_CHANGE_EPSILON);
        let is_resend_due = last_send_time_by_being
            .get(&being_ent)
            .is_none_or(|last_send| now - *last_send >= INPUT_RESEND_SECS);
        if unchanged && (input_dir == Vec2::ZERO || !is_resend_due) {
            continue;
        }
        let input_seq = next_seq_by_being.get(&being_ent).copied().unwrap_or(0).wrapping_add(1);
        next_seq_by_being.insert(being_ent, input_seq);
        last_sent_by_being.insert(being_ent, input_dir);
        last_send_time_by_being.insert(being_ent, now);
        drift_log(
            role,
            &format!(
                "t={:.3} send_input ent={:?} tick={} seq={} dir=({:.0},{:.0})",
                now,
                being_ent,
                *client_tick,
                input_seq,
                input_dir.x,
                input_dir.y
            ),
        );
        if let Some(mut pending_intents) = pending_intents {
            pending_intents.0.push(PendingMoveIntent {
                input_seq,
                client_tick: *client_tick,
                dir: input_dir,
            });
        } else {
            commands.entity(being_ent).try_insert(PendingMoveIntents(vec![PendingMoveIntent {
                input_seq,
                client_tick: *client_tick,
                dir: input_dir,
            }]));
        }
        messages.push(SendMoveInput { being_ent, vec: input_dir, input_seq, client_tick: *client_tick });
    }
    last_sent_by_being.retain(|being_ent, _| current_beings.contains(being_ent));
    next_seq_by_being.retain(|being_ent, _| current_beings.contains(being_ent));
    last_send_time_by_being.retain(|being_ent, _| current_beings.contains(being_ent));
    event_writer.write_batch(messages.drain(..));
}

pub fn receive_move_input_from_client(
    mut events: MessageReader<FromClient<SendMoveInput>>,
    client_state: Res<State<ClientState>>,
    mut commands: Commands,
    mut controlled_beings_query: Query<(
        Option<&mut RemoteMoveInput>,
        Option<&mut LastProcessedMoveInputSeq>,
        Option<&mut LastProcessedMoveInputTick>,
        &ControlledBy,
    )>,
) {
    let role = log_role(&client_state);
    for from_client in events.read() {
        let SendMoveInput { vec: new_vec, being_ent, input_seq, client_tick } = from_client.message.clone();

        if let Ok((remote_input, last_processed_seq, last_processed_tick, controlled_by)) = controlled_beings_query.get_mut(being_ent) {
            let Some(client_entity) = from_client.client_id.entity() else { continue; };

            if controlled_by.client_ent == client_entity {
                let prev_seq = last_processed_seq.as_ref().map_or(0, |seq| seq.0);
                if input_seq <= prev_seq {
                    drift_log(
                        role,
                        &format!(
                            "drop_stale_input ent={:?} client={:?} seq={} last_seq={}",
                            being_ent,
                            client_entity,
                            input_seq,
                            prev_seq
                        ),
                    );
                    continue;
                }
                let mut should_insert_remote = true;
                if let Some(mut remote_input) = remote_input {
                    should_insert_remote = false;
                    if remote_input.0 != new_vec {
                        remote_input.0 = new_vec;
                    }
                }
                let mut should_insert_seq = true;
                if let Some(mut seq) = last_processed_seq {
                    should_insert_seq = false;
                    seq.0 = input_seq;
                }
                let mut should_insert_tick = true;
                if let Some(mut tick) = last_processed_tick {
                    should_insert_tick = false;
                    tick.0 = client_tick;
                }
                if should_insert_remote {
                    commands.entity(being_ent).try_insert(RemoteMoveInput(new_vec));
                }
                if should_insert_seq {
                    commands.entity(being_ent).try_insert(LastProcessedMoveInputSeq(input_seq));
                }
                if should_insert_tick {
                    commands.entity(being_ent).try_insert(LastProcessedMoveInputTick(client_tick));
                }
            } else {
                warn!(
                    "Client tried to control a being not controlled by them: {} (controlled_by.client: {:?}, from_client.client_entity: {:?})",
                    being_ent, controlled_by.client_ent, client_entity
                );
            }
        } else {
            warn!("Client tried to control a being that does not exist in server or is not controllable {}", being_ent);
        }
    }
}

pub fn apply_remote_move_input_actions(
    client_state: Res<State<ClientState>>,
    mut commands: Commands,
    beings: Query<(Entity, &RemoteMoveInput), (Without<ComputedLocally>, With<Actions<BeingInputContext>>)>,
    mut last_applied_by_being: Local<EntityHashMap<Vec2>>,
    mut current_beings: Local<EntityHashSet>,
) {
    let role = log_role(&client_state);
    current_beings.clear();
    for (being_ent, remote_input) in beings.iter() {
        current_beings.insert(being_ent);
        if remote_input.0 == Vec2::ZERO
            && last_applied_by_being
                .get(&being_ent)
                .is_some_and(|prev| *prev == Vec2::ZERO)
        {
            continue;
        }
        if last_applied_by_being
            .get(&being_ent)
            .is_none_or(|prev| *prev != remote_input.0)
        {
            drift_log(
                role,
                &format!(
                    "apply_remote_input ent={:?} dir=({:.0},{:.0})",
                    being_ent,
                    remote_input.0.x,
                    remote_input.0.y
                ),
            );
        }
        let state = if remote_input.0 == Vec2::ZERO {
            TriggerState::None
        } else {
            TriggerState::Fired
        };
        commands
            .entity(being_ent)
            .try_mock::<BeingInputContext, BeingMoveAction>(state, remote_input.0, MockSpan::once());
        last_applied_by_being.insert(being_ent, remote_input.0);
    }
    last_applied_by_being.retain(|being_ent, _| current_beings.contains(being_ent));
}

pub fn process_input_direction_modifiers(
    state: Res<State<ClientState>>,
    move_actions: Query<&Action<BeingMoveAction>>,
    mut being_query: Query<
        (
            Entity,
            &AppliedModifiers,
            &Actions<BeingInputContext>,
            &mut MoveVecMag,
            Option<&RemoteMoveInput>,
            Has<ComputedLocally>,
        ),
    >,
    modifiers_query: Query<
        (Entity, &ModifierTarget, &CurrEffectiveValue, &ApplyMode, Has<InvertMovement>),
    >,
    mut prev_dir_by_being: Local<EntityHashMap<Vec2>>,
) {
    let is_client = state.get() == &ClientState::Connected;
    let mut current_beings = EntityHashSet::default();

    for (being_ent, applied, actions, mut move_state, remote_input, controlled_locally) in being_query.iter_mut() {
        if is_client && !controlled_locally { continue; }
        current_beings.insert(being_ent);
        let input_dir = if controlled_locally {
            let Some(move_action) = move_actions.iter_many(actions).next() else {
                continue;
            };
            sanitize_input_dir(**move_action)
        } else {
            remote_input.map_or(Vec2::ZERO, |remote_input| sanitize_input_dir(remote_input.0))
        };

        let mut invert_sum: f32 = 0.0;
        let mut invert_scale: f32 = 1.0;

        let mut effects = EntityHashSet::default();
        applied.entities().iter().for_each(|&ent| { effects.insert(ent); });

        for (modifier_ent, target, ..) in modifiers_query.iter() {
            if target.0 == being_ent {
                effects.insert(modifier_ent);
            }
        }

        for effect in effects.iter() {
            if let Ok((_, _, &CurrEffectiveValue(val), optype, invert)) = modifiers_query.get(*effect) {
                match optype {
                    ApplyMode::Add if invert => invert_sum += val,
                    ApplyMode::Mul if invert => invert_scale *= val.max(0.0),
                    _ => {}
                }
            }
        }

        move_state.norm_move_dir = if input_dir == Vec2::ZERO {
            Vec2::ZERO
        } else {
            let dir = input_dir;
            if invert_sum * invert_scale > 1.0 { -dir } else { dir }
        };
        if prev_dir_by_being.get(&being_ent).is_none_or(|prev| *prev != move_state.norm_move_dir) {
            prev_dir_by_being.insert(being_ent, move_state.norm_move_dir);
        }
    }
    prev_dir_by_being.retain(|being_ent, _| current_beings.contains(being_ent));
}
