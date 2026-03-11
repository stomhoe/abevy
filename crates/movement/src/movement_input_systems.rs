use ::being_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use common::log_targets::MOVEMENT_SYSTEM;

use crate::{movement_components::*, movement_messages::SendMoveInput};

pub fn send_move_input_to_server(
    mut writer: MessageWriter<SendMoveInput>,
    beings: Query<(Entity, &InputMoveDir), (With<ComputedLocally>, Changed<InputMoveDir>)>,
    mut last_tick_by_being: Local<EntityHashMap<u32>>,
    mut current_tick: Local<u32>,
    mut messages: Local<Vec<SendMoveInput>>,
) {
    *current_tick = current_tick.wrapping_add(1);
    for (being_ent, input_move_dir) in beings.iter() {

        let dir = input_move_dir.0.as_ivec2();
        let prev_tick = last_tick_by_being
            .get(&being_ent)
            .copied()
            .unwrap_or_default();

        let intent = PendingMoveIntent::new(dir, *current_tick, prev_tick);


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
    mut cmd: Commands,
    controlled_beings: Query<&ComputedBy>,
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
            pending.0.push_back(intent);
        } else {
            cmd.entity(being_ent).insert(PendingMoveIntents::new(intent));
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
    mut query: Query<(
        Entity,
        &mut PendingMoveIntents,
        Has<ComputedLocally>,
        &mut InputMoveDir,
    )>,
    mut current_tick: Local<u32>,
    mut last_tick_by_being: Local<EntityHashMap<u32>>,
) {
    *current_tick = current_tick.wrapping_add(1);
    for (being_ent, mut pending, controlled_locally, mut input_move_dir) in query.iter_mut() {
        if controlled_locally {
            continue;
        }
        let Some(intent) = pending.front() else {
            continue;
        };
        let last_tick = last_tick_by_being
            .get(&being_ent)
            .copied()
            .unwrap_or(*current_tick);
        let elapsed_ticks = current_tick.wrapping_sub(last_tick);
        if elapsed_ticks < intent.ticks_since_prev_intent.max(1) {
            continue;
        }
        let Some(intent) = pending.pop_front() else {
            continue;
        };
        last_tick_by_being.insert(being_ent, *current_tick);
        input_move_dir.0 = intent.dir.as_vec2();
        debug!(
            target: MOVEMENT_SYSTEM,
            "Replayed server move input for {:?}: dir={:?} dticks={}",
            being_ent,
            intent.dir,
            intent.ticks_since_prev_intent
        );
    }
}
