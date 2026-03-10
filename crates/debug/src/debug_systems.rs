use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::{ClientState, FromClient};
use modifier_shared::{modifier_components::*, modifier_types::WalkSpeed, };
use ac_input::ac_input_actions::{DebugDecreaseSpeedAction, DebugIncreaseSpeedAction};
use ::being::being_components::*;
use ::being_shared::*;

use crate::debug_messages::UpdateBeingSpeed;

#[allow(unused_parens)]
pub fn debug_increase_speed(
    speed_up: Single<&Action<DebugIncreaseSpeedAction>>,
    speed_down: Single<&Action<DebugDecreaseSpeedAction>>,
    my_being_query: Query<(&Being), (LocalPlayerControlled)>,
    mut query: Query<(&ModifierTarget, &mut BaseValue),(With<WalkSpeed>, )>,
    client_state: Res<State<ClientState>>,
    mut writer: MessageWriter<UpdateBeingSpeed>,
    mut msgs: Local<Vec<UpdateBeingSpeed>>,
) {
    let is_client = *client_state.get() == ClientState::Connected;

    if ***speed_up {
        query.iter_mut().for_each(|(target, mut val)| {
            if my_being_query.get(target.0).is_ok() {
                let mut new_val = val.clone();
                new_val.0 *= 1.05;
                if is_client {
                    val.0 = new_val.0;
                    msgs.push( UpdateBeingSpeed { being_ent: target.0, value: new_val, } );
                } else {
                    val.0 = new_val.0;
                }
            }
        });
    } else if ***speed_down {
        query.iter_mut().for_each(|(target, mut val)| {
            if my_being_query.get(target.0).is_ok() {
                let mut new_val = val.clone();
                new_val.0 *= 0.95;
                new_val.0 = new_val.0.max(1.);
                if is_client {
                    val.0 = new_val.0;
                    msgs.push( UpdateBeingSpeed { being_ent: target.0, value: new_val, } );
                } else {
                    val.0 = new_val.0;
                }
            }
        });
    }
    writer.write_batch(msgs.drain(..));
}

#[allow(unused_parens, )]
pub fn receive_increase_speed_from_client(
    mut events: MessageReader<FromClient<UpdateBeingSpeed>>,
    controlled_beings_query: Query<(&AppliedModifiers, &ControlledBy, ), ()>,
    mut modifiers_query: Query<(&mut BaseValue),(With<WalkSpeed>, )>,
) {
    for from_client in events.read() {
        let UpdateBeingSpeed { value: new_value, being_ent } = from_client.message.clone();
        trace!(target: "debug", "Received speed update for being {:?} with value {:?}", being_ent, new_value);

        if let Ok((applied_modifiers, controlled_by, )) = controlled_beings_query.get(being_ent) {

            let Some(client_entity) = from_client.client_id.entity() else { continue; };

            if controlled_by.client_ent == client_entity {
                for modifier_ent in applied_modifiers.entities() {
                    if let Ok(mut effective_value) = modifiers_query.get_mut(*modifier_ent) {
                        effective_value.0 = new_value.0;
                    }
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
