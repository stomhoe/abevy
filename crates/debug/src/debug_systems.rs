use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::{ClientState, FromClient, SendMode, ToClients};
use common::HashId;
use common::log_targets::DEBUG;
use game_common::game_common_components::Templ;
use movement::movement_components::InputMoveDir;
use modifier_shared::{
    modifier_components::*,
    modifier_move_bundles::SpeedModifier,
    modifier_types::WalkSpeed,
};
use ac_input::ac_input_actions::{DebugDecreaseSpeedAction, DebugIncreaseSpeedAction};
use ::being_shared::*;

use crate::debug_messages::{BeingDebugSpeedApplied, UpdateBeingSpeed};
use crate::debug_resources::PendingSpeedDebugUpdates;

#[allow(unused_parens)]
pub fn debug_increase_speed(
    mut cmd: Commands,
    speed_up: Single<&Action<DebugIncreaseSpeedAction>>,
    speed_down: Single<&Action<DebugDecreaseSpeedAction>>,
    my_being_query: Query<(Entity, &AppliedModifiers), (LocalHumanControlled)>,
    debug_modi_query: Query<(&HashId, Option<&BaseValue>), (With<WalkSpeed>, Without<Templ>)>,
    client_state: Res<State<ClientState>>,
    mut pending: ResMut<PendingSpeedDebugUpdates>,
    mut writer: MessageWriter<UpdateBeingSpeed>,
    mut msgs: Local<Vec<UpdateBeingSpeed>>,
) {
    let is_client = *client_state.get() == ClientState::Connected;
    let debug_hash = HashId::hash("debug");
    let factor = if ***speed_up {
        1.05
    } else if ***speed_down {
        0.95
    } else {
        return;
    };

    let Ok((being_ent, applied_modifiers)) = my_being_query.single() else {
        return;
    };

    if is_client {
        pending.by_being.insert(being_ent);
        debug!(target: DEBUG, "Queued debug speed update for {:?} factor {}", being_ent, factor);
        msgs.push(UpdateBeingSpeed {
            being_ent,
            factor,
        });
    } else {
        let mut debug_modifier_ent = None;
        let mut current_value = 1.0;
        for modifier_ent in applied_modifiers.iter() {
            let Ok((hash_id, base_value)) = debug_modi_query.get(modifier_ent) else {
                continue;
            };
            if *hash_id == debug_hash {
                debug_modifier_ent = Some(modifier_ent);
                current_value = base_value.map(|value| value.0).unwrap_or(1.0);
                break;
            }
        }

        let new_value = (current_value * factor).max(0.1);
        if let Some(modifier_ent) = debug_modifier_ent {
            cmd.entity(modifier_ent).insert(BaseValue(new_value));
        } else {
            cmd.spawn((
                SpeedModifier::new(being_ent, being_ent, new_value, ApplyMode::Mul),
                debug_hash,
            ));
        }
    }

    writer.write_batch(msgs.drain(..));
}

pub fn disable_movement_while_speed_debug_update_pending(
    pending: ResMut<PendingSpeedDebugUpdates>,
    walk_speeds: Query<(&ModifierTarget, &HashId), (With<WalkSpeed>, Without<Templ>)>,
    mut move_dirs: Query<&mut InputMoveDir>,
) {
    for (target, hash_id) in walk_speeds.iter() {
        if *hash_id != HashId::hash("debug") {
            continue;
        }
        if !pending.by_being.contains(&target.0) {
            continue;
        }
        let Ok(mut input_move_dir) = move_dirs.get_mut(target.0) else {
            continue;
        };
        if input_move_dir.0 != Vec2::ZERO {
            debug!(target: DEBUG, "Holding movement for {:?} until speed update is applied", target.0);
            input_move_dir.0 = Vec2::ZERO;
        }
    }
}

#[allow(unused_parens, )]
pub fn receive_speed_update_applied_from_server(
    mut events: MessageReader<BeingDebugSpeedApplied>,
    mut pending: ResMut<PendingSpeedDebugUpdates>,
) {
    for event in events.read() {
        if pending.by_being.remove(&event.being_ent) {
            debug!(
                target: DEBUG,
                "Applied debug speed update ack for {:?} applied={}",
                event.being_ent,
                event.applied
            );
        }
    }
}

#[allow(unused_parens, )]
pub fn receive_increase_speed_from_client(
    mut cmd: Commands,
    mut events: MessageReader<FromClient<UpdateBeingSpeed>>,
    controlled_beings_query: Query<(&AppliedModifiers, &ComputedBy, ), ()>,
    debug_modi_query: Query<(Entity, &HashId, Option<&BaseValue>), (With<WalkSpeed>, Without<Templ>)>,
    mut writer: MessageWriter<ToClients<BeingDebugSpeedApplied>>,
    mut msgs: Local<Vec<ToClients<BeingDebugSpeedApplied>>>,
) {
    let debug_hash = HashId::hash("debug");

    for from_client in events.read() {
        let UpdateBeingSpeed { factor, being_ent } = from_client.message.clone();
        debug!(target: DEBUG, "Received speed update for {:?} factor {}", being_ent, factor);

        let mut applied = false;
        let Ok((applied_modifiers, controlled_by, )) = controlled_beings_query.get(being_ent) else {
            warn!(target: DEBUG, "Client tried to control a being that does not exist in server or is not controllable {}", being_ent);
            msgs.push(ToClients {
                mode: SendMode::Direct(from_client.client_id),
                message: BeingDebugSpeedApplied {
                    being_ent,
                    applied,
                },
            });
            continue;
        };
        let Some(client_entity) = from_client.client_id.entity() else {
            msgs.push(ToClients {
                mode: SendMode::Direct(from_client.client_id),
                message: BeingDebugSpeedApplied {
                    being_ent,
                    applied,
                },
            });
            continue;
        };
        if controlled_by.client_ent != client_entity {
            warn!(
                target: DEBUG,
                "Client tried to control a being not controlled by them: {} (controlled_by.client: {:?}, from_client.client_entity: {:?})",
                being_ent, controlled_by.client_ent, client_entity
            );
            msgs.push(ToClients {
                mode: SendMode::Direct(from_client.client_id),
                message: BeingDebugSpeedApplied {
                    being_ent,
                    applied,
                },
            });
            continue;
        }

        let mut debug_modifier_ent = None;
        let mut current_value = 1.0;
        for modifier_ent in applied_modifiers.iter() {
            let Ok((entity, hash_id, base_value)) = debug_modi_query.get(modifier_ent) else {
                continue;
            };
            if *hash_id == debug_hash {
                debug_modifier_ent = Some(entity);
                current_value = base_value.map(|value| value.0).unwrap_or(1.0);
                break;
            }
        }

        let new_value = (current_value * factor).max(0.1);
        if let Some(modifier_ent) = debug_modifier_ent {
            cmd.entity(modifier_ent).insert(BaseValue(new_value));
        } else {
            cmd.spawn((
                SpeedModifier::new(being_ent, being_ent, new_value, ApplyMode::Mul),
                debug_hash,
            ));
        }
        applied = true;
        msgs.push(ToClients {
            mode: SendMode::Direct(from_client.client_id),
            message: BeingDebugSpeedApplied {
                being_ent,
                applied,
            },
        });
    }
    writer.write_batch(msgs.drain(..));
}
