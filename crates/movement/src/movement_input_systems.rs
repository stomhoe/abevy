use ::being_shared::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use common::log_targets::MOVEMENT_SYSTEM;

use crate::{movement_components::*, movement_messages::SendMoveInput};

pub fn send_move_input_to_server(
    mut writer: MessageWriter<SendMoveInput>,
    beings: Query<(Entity, &InputMoveDir), (With<ComputedLocally>, Changed<InputMoveDir>)>,
    mut messages: Local<Vec<SendMoveInput>>,
) {
    for (being_ent, input_move_dir) in beings.iter() {
        let dir = input_move_dir.0.as_ivec2();
        messages.push(SendMoveInput {
            being_ent,
            intent: PendingMoveIntent { dir },
        });
        debug!(
            target: MOVEMENT_SYSTEM,
            "Queued client move input for {:?}: dir={:?}",
            being_ent,
            dir,
        );
    }
    writer.write_batch(messages.drain(..));
}

pub fn receive_move_input_from_client(
    mut events: MessageReader<FromClient<SendMoveInput>>,
    controlled_beings: Query<&ComputedBy>,
    mut input_move_dirs: Query<&mut InputMoveDir, Without<ComputedLocally>>,
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
        let Ok(mut input_move_dir) = input_move_dirs.get_mut(being_ent) else {
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped move input for {:?}: missing InputMoveDir or unexpectedly ComputedLocally",
                being_ent
            );
            continue;
        };
        error!(
            target: MOVEMENT_SYSTEM,
            "InputMoveDir set from client message for {:?}: {:?} -> {:?}",
            being_ent,
            input_move_dir.0,
            intent.dir.as_vec2()
        );
        input_move_dir.0 = intent.dir.as_vec2();
        debug!(
            target: MOVEMENT_SYSTEM,
            "Accepted server move input for {:?}: dir={:?}",
            being_ent,
            intent.dir,
        );
    }
}
