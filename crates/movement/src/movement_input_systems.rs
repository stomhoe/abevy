use ::being_shared::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

use common::log_targets::MOVEMENT_SYSTEM;

use crate::{movement_components::*, movement_messages::*};

const INPUT_TICK_LEAD: u32 = 2;

pub fn tick_movement_sim(mut tick: ResMut<MovementSimTick>) {
    tick.0 = tick.0.wrapping_add(1);
}

pub fn send_move_input_to_server(
    server_tick: Res<LastKnownServerMovementTick>,
    mut writer: MessageWriter<SendMoveInput>,
    beings: Query<(Entity, &InputMoveDir), (With<ComputedLocally>, Changed<InputMoveDir>)>,
    mut messages: Local<Vec<SendMoveInput>>,
) {
    for (being_ent, input_move_dir) in beings.iter() {
        let start_tick = server_tick.0.wrapping_add(INPUT_TICK_LEAD);
        let dir = input_move_dir.0.as_ivec2();
        messages.push(SendMoveInput {
            being_ent,
            intent: PendingMoveIntent {
                start_tick,
                dir,
            },
        });
        debug!(
            target: MOVEMENT_SYSTEM,
            "Queued client move input for {:?}: dir={:?} start_tick={}",
            being_ent,
            dir,
            start_tick,
        );
    }
    writer.write_batch(messages.drain(..));
}

pub fn send_movement_tick_to_clients(
    tick: Res<MovementSimTick>,
    connected: Query<&player::player_components::Player, Without<player::player_components::Mine>>,
    mut writer: MessageWriter<ToClients<SyncMovementTick>>,
    mut messages: Local<Vec<ToClients<SyncMovementTick>>>,
) {
    if connected.is_empty() {
        return;
    }
    messages.push(ToClients {
        mode: SendMode::Broadcast,
        message: SyncMovementTick { tick: tick.0 },
    });
    writer.write_batch(messages.drain(..));
}

pub fn receive_move_input_from_client(
    tick: Res<MovementSimTick>,
    mut events: MessageReader<FromClient<SendMoveInput>>,
    controlled_beings: Query<&ComputedBy>,
    mut beings: Query<(
        &mut InputMoveDir,
        Option<&mut BufferedMoveInput>,
    ), Without<ComputedLocally>>,
    mut commands: Commands,
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
        let Ok((mut input_move_dir, buffered)) = beings.get_mut(being_ent) else {
            warn!(
                target: MOVEMENT_SYSTEM,
                "Dropped move input for {:?}: missing InputMoveDir or unexpectedly ComputedLocally",
                being_ent
            );
            continue;
        };
        if intent.start_tick <= tick.0 {
            error!(
                target: MOVEMENT_SYSTEM,
                "InputMoveDir set from client message for {:?}: {:?} -> {:?}",
                being_ent,
                input_move_dir.0,
                intent.dir.as_vec2()
            );
            input_move_dir.0 = intent.dir.as_vec2();
        } else if let Some(mut buffered) = buffered {
            buffered.start_tick = intent.start_tick;
            buffered.dir = intent.dir;
        } else {
            commands.entity(being_ent).insert(BufferedMoveInput {
                start_tick: intent.start_tick,
                dir: intent.dir,
            });
        }
        debug!(
            target: MOVEMENT_SYSTEM,
            "Accepted server move input for {:?}: dir={:?} start_tick={} curr_tick={}",
            being_ent,
            intent.dir,
            intent.start_tick,
            tick.0,
        );
    }
}

pub fn apply_buffered_move_input(
    tick: Res<MovementSimTick>,
    mut beings: Query<(Entity, &mut InputMoveDir, &BufferedMoveInput), Without<ComputedLocally>>,
    mut commands: Commands,
) {
    for (being_ent, mut input_move_dir, buffered) in beings.iter_mut() {
        if buffered.start_tick > tick.0 {
            continue;
        }
        error!(
            target: MOVEMENT_SYSTEM,
            "InputMoveDir set from buffered client input for {:?}: {:?} -> {:?}",
            being_ent,
            input_move_dir.0,
            buffered.dir.as_vec2()
        );
        input_move_dir.0 = buffered.dir.as_vec2();
        commands.entity(being_ent).remove::<BufferedMoveInput>();
    }
}

pub fn receive_movement_tick_from_server(
    mut reader: MessageReader<SyncMovementTick>,
    mut server_tick: ResMut<LastKnownServerMovementTick>,
) {
    for message in reader.read() {
        server_tick.0 = message.tick;
    }
}
