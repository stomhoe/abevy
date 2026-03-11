use ::being_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::*;

use ac_input::ac_input_actions::*;
use common::log_targets::MOVEMENT_SYSTEM;

use crate::{
    movement_components::*, movement_helpers::normalize_to_axis_dir,
    movement_messages::SendMoveInput, movement_secondary_systems::INPUT_DEADZONE,
};

pub fn send_move_input_to_server(
    mut writer: MessageWriter<SendMoveInput>,
    move_actions: Query<(&Action<BeingMoveAction>, &ActionOf<BeingInputContext>)>,
    beings: Query<(&ControlledBy, Has<ComputedLocally>)>,
    mut last_tick_by_being: Local<EntityHashMap<u32>>,
    mut current_tick: Local<u32>,
    mut messages: Local<Vec<SendMoveInput>>,
) {
    *current_tick += 1;
    for (move_action, action_of) in move_actions.iter() {
        let being_ent = **action_of;
        let Ok((controlled_by, controlled_locally)) = beings.get(being_ent) else {
            continue;
        };
        if !controlled_locally || !controlled_by.human_input {
            continue;
        }
        let dir = if move_action.length() <= INPUT_DEADZONE {
            IVec2::ZERO
        } else {
            normalize_to_axis_dir(move_action.normalize())
        };
        if dir == IVec2::ZERO {
            continue;
        }
        let prev_tick = last_tick_by_being
            .get(&being_ent)
            .copied()
            .unwrap_or_default();

        let intent = PendingMoveIntent::new(dir, prev_tick, *current_tick);


        messages.push(SendMoveInput {
            being_ent,
            intent,
        });
        last_tick_by_being.insert(being_ent, *current_tick);
        debug!(
            target: MOVEMENT_SYSTEM,
            "Queued client move input for {:?}: dir={:?} dticks={}",
            being_ent,
            dir,
            intent.ticks_since_prev_intent
        );
    }
    writer.write_batch(messages.drain(..));
}

pub fn receive_move_input_from_client(
    mut events: MessageReader<FromClient<SendMoveInput>>,
    mut commands: Commands,
    controlled_beings: Query<&ControlledBy>,
    mut pending_move_intents: Query<&mut PendingMoveIntents>,
) {
    for from_client in events.read() {
        let SendMoveInput { being_ent, intent } = from_client.message.clone();
        let Some(client_ent) = from_client.client_id.entity() else {
            continue;
        };
        let Ok(controlled_by) = controlled_beings.get(being_ent) else {
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped move input for uncontrolled/missing being {:?} from {:?}",
                being_ent,
                client_ent
            );
            continue;
        };
        if controlled_by.client_ent != client_ent {
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped spoofed move input for {:?}: owner {:?}, sender {:?}",
                being_ent,
                controlled_by.client_ent,
                client_ent
            );
            continue;
        }
        if let Ok(mut pending) = pending_move_intents.get_mut(being_ent) {
            pending.0.push(intent);
        } else {
            commands.entity(being_ent).insert(PendingMoveIntents(vec![intent]));
        }
        debug!(
            target: MOVEMENT_SYSTEM,
            "Accepted server move input for {:?}: dir={:?} dticks={}",
            being_ent,
            intent.dir,
            intent.ticks_since_prev_intent
        );
    }
}

pub fn replay_move_inputs_on_server(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut PendingMoveIntents,
        Option<&mut ServerMoveReplayState>,
        Has<ComputedLocally>,
    )>,
) {
    for (being_ent, mut pending, replay_state, controlled_locally) in query.iter_mut() {
        if controlled_locally {
            continue;
        }
        let mut replay_state = replay_state.map(|state| *state).unwrap_or_default();
        if replay_state.ticks_until_next_intent > 0 {
            replay_state.ticks_until_next_intent -= 1;
        }
        while replay_state.ticks_until_next_intent == 0 && !pending.0.is_empty() {
            let next_intent = pending.0.remove(0);
            replay_state.active_dir = next_intent.dir;
            replay_state.ticks_until_next_intent =
                next_intent.ticks_since_prev_intent.saturating_sub(1);
            debug!(
                target: MOVEMENT_SYSTEM,
                "Replaying server move input for {:?}: dir={:?} next_in_ticks={}",
                being_ent,
                replay_state.active_dir,
                replay_state.ticks_until_next_intent
            );
        }
        let input = replay_state.active_dir.as_vec2();
        let state = if input == Vec2::ZERO {
            TriggerState::None
        } else {
            TriggerState::Fired
        };
        commands
            .entity(being_ent)
            .try_mock::<BeingInputContext, BeingMoveAction>(state, input, MockSpan::once());
        commands.entity(being_ent).insert(replay_state);
    }
}
