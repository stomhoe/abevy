use ::being_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use ac_input::ac_input_actions::*;

use crate::{movement_components::*, movement_messages::SendMoveInput};

const INPUT_DEADZONE: f32 = 0.2;

fn sanitize_input_dir(input: Vec2) -> IVec2 {
    if input.length() <= INPUT_DEADZONE {
        IVec2::ZERO
    } else {
        let n = input.normalize();
        if n.x.abs() >= n.y.abs() {
            IVec2::new(n.x.signum() as i32, 0)
        } else {
            IVec2::new(0, n.y.signum() as i32)
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
        let dir = sanitize_input_dir(**move_action);
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
