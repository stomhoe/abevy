use core::f32;

use ::being_shared::*;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::*;
use common::log_targets;

use modifier_shared::{modifier_components::*, modifier_move_components::*, modifier_types::*};
use ac_input::ac_input_actions::*;

use crate::{movement_components::*, movement_messages::SendMoveInput};

const INPUT_DEADZONE: f32 = 0.2;
const INPUT_CHANGE_EPSILON: f32 = 0.01;

fn sanitize_input_dir(input: Vec2) -> Vec2 {
    if input.length() <= INPUT_DEADZONE {
        Vec2::ZERO
    } else {
        input.normalize()
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
        debug!(
            target: log_targets::MOVEMENT_SYSTEM,
            "Attaching input context being_ent={:?}",
            being_ent
        );
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
    mut event_writer: MessageWriter<SendMoveInput>,
    move_actions: Query<(&Action<BeingMoveAction>, &ActionOf<BeingInputContext>)>,
    controlled_beings: Query<(&ControlledBy, Has<ComputedLocally>)>,
    mut last_sent_by_being: Local<EntityHashMap<Vec2>>,
) {
    let mut to_write = Vec::new();
    let mut current_beings = EntityHashSet::default();
    for (move_action, action_of) in move_actions.iter() {
        let being_ent = **action_of;
        let Ok((controlled_by, controlled_locally)) = controlled_beings.get(being_ent) else {
            continue;
        };
        if !controlled_locally || !controlled_by.human_input {
            continue;
        }
        current_beings.insert(being_ent);
        let input_dir = sanitize_input_dir(**move_action);
        if last_sent_by_being.get(&being_ent).is_some_and(|last_sent| last_sent.distance(input_dir) <= INPUT_CHANGE_EPSILON) {
            continue;
        }
        last_sent_by_being.insert(being_ent, input_dir);
        debug!(
            target: log_targets::MOVEMENT_SYSTEM,
            "Sending move input to server being_ent={:?} input_dir={:?}",
            being_ent,
            input_dir
        );
        to_write.push(SendMoveInput { being_ent, vec: input_dir });
    }
    last_sent_by_being.retain(|being_ent, _| current_beings.contains(being_ent));
    event_writer.write_batch(to_write);
}

pub fn receive_move_input_from_client(
    mut events: MessageReader<FromClient<SendMoveInput>>,
    mut commands: Commands,
    mut controlled_beings_query: Query<(Option<&mut RemoteMoveInput>, &ControlledBy)>,
) {
    for from_client in events.read() {
        let SendMoveInput { vec: new_vec, being_ent } = from_client.message.clone();

        if let Ok((remote_input, controlled_by)) = controlled_beings_query.get_mut(being_ent) {
            let Some(client_entity) = from_client.client_id.entity() else { continue; };

            if controlled_by.client_ent == client_entity {
                debug!(
                    target: log_targets::MOVEMENT_SYSTEM,
                    "Server received move input being_ent={:?} client_ent={:?} input_dir={:?}",
                    being_ent,
                    client_entity,
                    new_vec
                );
                let mut should_insert = true;
                if let Some(mut remote_input) = remote_input {
                    should_insert = false;
                    if remote_input.0 != new_vec {
                        remote_input.0 = new_vec;
                    }
                }
                if should_insert {
                    commands.entity(being_ent).try_insert(RemoteMoveInput(new_vec));
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
    mut commands: Commands,
    beings: Query<(Entity, &RemoteMoveInput), (Without<ComputedLocally>, With<Actions<BeingInputContext>>)>,
    mut last_applied_by_being: Local<EntityHashMap<Vec2>>,
    mut current_beings: Local<EntityHashSet>,
) {
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
            debug!(
                target: log_targets::MOVEMENT_SYSTEM,
                "Applying remote move input to server action state being_ent={:?} input_dir={:?}",
                being_ent,
                remote_input.0
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
            debug!(
                target: log_targets::MOVEMENT_SYSTEM,
                "Resolved movement input being_ent={:?} controlled_locally={} input_source={} raw_input={:?} resolved_dir={:?}",
                being_ent,
                controlled_locally,
                if controlled_locally { "local_action" } else { "remote_input" },
                input_dir,
                move_state.norm_move_dir
            );
            prev_dir_by_being.insert(being_ent, move_state.norm_move_dir);
        }
    }
    prev_dir_by_being.retain(|being_ent, _| current_beings.contains(being_ent));
}
