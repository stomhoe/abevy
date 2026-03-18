use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::{ClientState, FromClient};
use common::log_targets::DEBUG;
use movement::movement_components::InputMoveDir;
use modifier_shared::{modifier_components::*, modifier_types::WalkSpeed, };
use ac_input::ac_input_actions::{DebugDecreaseSpeedAction, DebugIncreaseSpeedAction};
use ::being::being_components::*;
use ::being_shared::*;

use crate::debug_messages::UpdateBeingSpeed;
use crate::debug_resources::PendingSpeedDebugUpdates;

#[allow(unused_parens)]
pub fn debug_increase_speed(
    speed_up: Single<&Action<DebugIncreaseSpeedAction>>,
    speed_down: Single<&Action<DebugDecreaseSpeedAction>>,
    my_being_query: Query<(&Being), (LocalHumanControlled)>,
    mut query: Query<(&ModifierTarget, &mut BaseValue),(With<WalkSpeed>, )>,
    client_state: Res<State<ClientState>>,
    mut pending: ResMut<PendingSpeedDebugUpdates>,
    mut writer: MessageWriter<UpdateBeingSpeed>,
    mut msgs: Local<Vec<UpdateBeingSpeed>>,
) {
    let is_client = *client_state.get() == ClientState::Connected;

    if ***speed_up {
        query.iter_mut().for_each(|(target, mut val)| {
            if my_being_query.get(target.0).is_ok() {
                let mut new_val = BaseValue(pending.by_being.get(&target.0).copied().unwrap_or(val.0));
                new_val.0 *= 1.05;
                if is_client {
                    pending.by_being.insert(target.0, new_val.0);
                    debug!(target: DEBUG, "Queued debug speed increase for {:?} -> {}", target.0, new_val.0);
                    msgs.push( UpdateBeingSpeed { being_ent: target.0, value: new_val, } );
                } else {
                    val.0 = new_val.0;
                }
            }
        });
    } else if ***speed_down {
        query.iter_mut().for_each(|(target, mut val)| {
            if my_being_query.get(target.0).is_ok() {
                let mut new_val = BaseValue(pending.by_being.get(&target.0).copied().unwrap_or(val.0));
                new_val.0 *= 0.95;
                new_val.0 = new_val.0.max(1.);
                if is_client {
                    pending.by_being.insert(target.0, new_val.0);
                    debug!(target: DEBUG, "Queued debug speed decrease for {:?} -> {}", target.0, new_val.0);
                    msgs.push( UpdateBeingSpeed { being_ent: target.0, value: new_val, } );
                } else {
                    val.0 = new_val.0;
                }
            }
        });
    }
    writer.write_batch(msgs.drain(..));
}

pub fn disable_movement_while_speed_debug_update_pending(
    mut pending: ResMut<PendingSpeedDebugUpdates>,
    local_beings: Query<(), LocalHumanControlled>,
    walk_speeds: Query<(&ModifierTarget, &BaseValue), With<WalkSpeed>>,
    mut move_dirs: Query<&mut InputMoveDir>,
) {
    let mut completed = Vec::new();

    for (target, base_value) in walk_speeds.iter() {
        let Some(&pending_value) = pending.by_being.get(&target.0) else { continue; };
        if local_beings.get(target.0).is_err() {
            completed.push(target.0);
            continue;
        }
        if (base_value.0 - pending_value).abs() <= f32::EPSILON {
            debug!(target: DEBUG, "Applied debug speed update for {:?} at {}", target.0, pending_value);
            completed.push(target.0);
            continue;
        }
        let Ok(mut input_move_dir) = move_dirs.get_mut(target.0) else { continue; };
        if input_move_dir.0 != Vec2::ZERO {
            debug!(target: DEBUG, "Holding movement for {:?} until speed update reaches {}", target.0, pending_value);
            input_move_dir.0 = Vec2::ZERO;
        }
    }

    for being_ent in completed {
        pending.by_being.remove(&being_ent);
    }
}

#[allow(unused_parens, )]
pub fn receive_increase_speed_from_client(
    mut events: MessageReader<FromClient<UpdateBeingSpeed>>,
    controlled_beings_query: Query<(&AppliedModifiers, &ComputedBy, ), ()>,
    mut modifiers_query: Query<(&mut BaseValue),(With<WalkSpeed>, )>,
) {
    for from_client in events.read() {
        let UpdateBeingSpeed { value: new_value, being_ent } = from_client.message.clone();
        debug!(target: DEBUG, "Received speed update for {:?} -> {:?}", being_ent, new_value);

        if let Ok((applied_modifiers, controlled_by, )) = controlled_beings_query.get(being_ent) {

            let Some(client_entity) = from_client.client_id.entity() else { continue; };

            if controlled_by.client_ent == client_entity {
                for modifier_ent in applied_modifiers.iter() {
                    if let Ok(mut effective_value) = modifiers_query.get_mut(modifier_ent) {
                        effective_value.0 = new_value.0;
                    }
                }
            } else {

                warn!(
                    target: DEBUG,
                    "Client tried to control a being not controlled by them: {} (controlled_by.client: {:?}, from_client.client_entity: {:?})",
                    being_ent, controlled_by.client_ent, client_entity
                );
            }
        } else {
            warn!(target: DEBUG, "Client tried to control a being that does not exist in server or is not controllable {}", being_ent);
        }
    }
}
