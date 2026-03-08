use core::f32;

use ::being_shared::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::*;
use bevy::ecs::entity::EntityHashSet;

use modifier_shared::{modifier_components::*, modifier_move_components::*, modifier_types::*};
use ac_input::ac_input_actions::*;

use crate::{movement_components::*, movement_messages::SendMoveInput};

pub fn add_being_input_context(
    mut commands: Commands,
    being_query: Query<
        Entity,
        (
            Or<(Added<ComputedLocally>, Added<ControlledBy>)>,
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
    mut event_writer: MessageWriter<SendMoveInput>,
    changed_move_actions: Query<
        (&Action<BeingMoveAction>, &ActionOf<BeingInputContext>),
        Changed<Action<BeingMoveAction>>,
    >,
    controlled_beings: Query<(&ControlledBy, Has<ComputedLocally>)>,
) {
    let mut to_write = Vec::new();
    for (move_action, action_of) in changed_move_actions.iter() {
        let being_ent = **action_of;
        let Ok((controlled_by, controlled_locally)) = controlled_beings.get(being_ent) else {
            continue;
        };
        if !controlled_locally || !controlled_by.human_input {
            continue;
        }
        let mut input_dir = **move_action;
        if input_dir != Vec2::ZERO {
            input_dir = input_dir.normalize();
        }
        trace!(target: "movement", "Sending move input for entity {:?} with vector {:?}", being_ent, input_dir);
        to_write.push(SendMoveInput { being_ent, vec: input_dir });
    }
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
) {
    for (being_ent, remote_input) in beings.iter() {
        let state = if remote_input.0 == Vec2::ZERO {
            TriggerState::None
        } else {
            TriggerState::Fired
        };
        commands
            .entity(being_ent)
            .try_mock::<BeingInputContext, BeingMoveAction>(state, remote_input.0, MockSpan::once());
    }
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
            Has<ComputedLocally>,
        ),
    >,
    modifiers_query: Query<
        (Entity, &ModifierTarget, &CurrEffectiveValue, &ApplyMode, Has<InvertMovement>),
    >,
) {
    let is_client = state.get() == &ClientState::Connected;

    for (being_ent, applied, actions, mut move_state, controlled_locally) in being_query.iter_mut() {
        if is_client && !controlled_locally { continue; }
        let Some(move_action) = move_actions.iter_many(actions).next() else {
            continue;
        };
        let mut input_dir = **move_action;
        if input_dir != Vec2::ZERO {
            input_dir = input_dir.normalize();
        }

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
    }
}
