use ::being_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use ac_input::ac_input_actions::*;

use crate::{
    movement_components::*, movement_helpers::normalize_to_axis_dir,
    movement_messages::SendMoveInput, movement_secondary_systems::INPUT_DEADZONE,
};

pub fn send_move_input_to_server(
    mut commands: Commands,
    mut writer: MessageWriter<SendMoveInput>,
    move_actions: Query<(&Action<BeingMoveAction>, &ActionOf<BeingInputContext>)>,
    beings: Query<(
        &ControlledBy,
        Has<ComputedLocally>,
        Option<&PendingMoveIntents>,
    )>,
    mut next_seq_by_being: Local<EntityHashMap<u32>>,
    mut messages: Local<Vec<SendMoveInput>>,
) {
    for (move_action, action_of) in move_actions.iter() {
        let being_ent = **action_of;
        let Ok((controlled_by, controlled_locally, pending)) = beings.get(being_ent) else {
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
        let next_seq = next_seq_by_being
            .get(&being_ent)
            .copied()
            .unwrap_or_default()
            .wrapping_add(1);
        next_seq_by_being.insert(being_ent, next_seq);
        let mut pending_intents = pending.cloned().unwrap_or_default();
        pending_intents.0.push(PendingMoveIntent {
            input_seq: next_seq,
            dir,
        });
        commands.entity(being_ent).insert(pending_intents);
        messages.push(SendMoveInput {
            being_ent,
            dir,
            input_seq: next_seq,
        });
    }
    writer.write_batch(messages.drain(..));
}
