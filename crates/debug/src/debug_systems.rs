use bevy::prelude::*;
use bevy_fps_counter::FpsCounter;
use bevy_replicon::prelude::{ClientState, FromClient};
use modifier::{modifier_components::*, modifier_move_components::Speed};
use being::being_components::{Being, ControlledBy, ControlledLocally};

use crate::debug_messages::UpdateBeingSpeed;

#[allow(unused_parens)]
pub fn debug_increase_speed(
    keys: Res<ButtonInput<KeyCode>>,
    my_being_query: Query<(&Being), (With<ControlledLocally>)>,
    mut query: Query<(&ModifierTarget, &mut CurrFinalValue),(With<Speed>, )>,
    client_state: Res<State<ClientState>>,
    mut writer: MessageWriter<UpdateBeingSpeed>,
) {

    let is_client = *client_state.get() == ClientState::Connected;
    let mut msgs = Vec::new();

    if keys.pressed(KeyCode::NumpadAdd) {
        query.iter_mut().for_each(|(target, mut val)| {
            if my_being_query.get(target.0).is_ok() {
                val.0 *= 1.05;
                if is_client {
                    msgs.push( UpdateBeingSpeed { being_ent: target.0, value: val.clone(), } );
                }
            }
        });
    } else if keys.pressed(KeyCode::NumpadSubtract) {
        query.iter_mut().for_each(|(target, mut val)| {
            if my_being_query.get(target.0).is_ok() {
                val.0 *= 0.95;
                val.0 = val.0.max(1.);
                if is_client {
                    msgs.push( UpdateBeingSpeed { being_ent: target.0, value: val.clone(), } );
                }
            }
        });
    }

    if is_client {
        writer.write_batch(msgs);
    }
}

#[allow(unused_parens, )]
pub fn receive_increase_speed_from_client(
    mut events: MessageReader<FromClient<UpdateBeingSpeed>>,
    controlled_beings_query: Query<(&AppliedModifiers, &ControlledBy, ), ()>,
    mut modifiers_query: Query<(&mut CurrFinalValue),(With<Speed>, )>,
) {
    for from_client in events.read() {
        let UpdateBeingSpeed { value: new_value, being_ent } = from_client.message.clone();
        info!(target:"debug", "Received speed update for being {:?} with value {:?}", being_ent, new_value);

        if let Ok((applied_modifiers, controlled_by, )) = controlled_beings_query.get(being_ent) {

            let Some(client_entity) = from_client.client_id.entity() else { continue; };
            
            if controlled_by.client == client_entity {
                for modifier_ent in applied_modifiers.entities() {
                    if let Ok(mut effective_value) = modifiers_query.get_mut(*modifier_ent) {
                        effective_value.0 = new_value.0;
                    }
                }
            } else {

                warn!(
                    "Client tried to control a being not controlled by them: {} (controlled_by.client: {:?}, from_client.client_entity: {:?})",
                    being_ent, controlled_by.client, client_entity
                );
            }
        } else {
            warn!("Client tried to control a being that does not exist in server or is not controllable {}", being_ent);
        }
    }
}


fn mouse_handler(
    mouse_button_input: Res<ButtonInput<KeyCode>>,
    mut diags_state: ResMut<FpsCounter>,
) {
    if mouse_button_input.pressed(KeyCode::F11) {
        if diags_state.is_enabled() {
            diags_state.disable();
        } else {
            diags_state.enable();
        }
    }
}