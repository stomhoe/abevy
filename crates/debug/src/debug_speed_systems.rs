use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy_replicon::prelude::{ClientState, FromClient};
use common::common_components::SettingsEntity;
use common::HashId;
use common::log_targets::DEBUG;
use debug_shared::DebugUiConfig;
use game_common::Templ;
use modifier_shared::{
    AppliedModifiers,
    ApplyMode,
    modifier_components::BaseValue,
    modifier_move_bundles::SpeedModifier,
    modifier_types::WalkStrength,
};
use ::being_shared::{Being, LocalHumanControlled};
use crate::debug_messages::ClientDebugSetSpeedRequest;

pub const DEBUG_SPEED_MIN: f32 = 0.5;
pub const DEBUG_SPEED_MAX: f32 = 100.;

pub(crate) fn set_speed_multiplier(
    cmd: &mut Commands,
    being_ent: Entity,
    applied_modifiers: &AppliedModifiers,
    debug_modi_query: &mut Query<(Entity, &HashId, Option<&mut BaseValue>, ), (With<WalkStrength>, Without<Templ>, )>,
    multiplier: f32,
) {
    let debug_hash = HashId::hash("debug");
    for modifier_ent in applied_modifiers.iter() {
        let Ok((entity, hash_id, base_value, )) = debug_modi_query.get_mut(modifier_ent) else {
            continue;
        };
        if *hash_id != debug_hash {
            continue;
        }
        let new_value = multiplier.max(0.0);
        match base_value {
            Some(mut base_value) => base_value.0 = new_value,
            None => {
                cmd.entity(entity).insert(BaseValue(new_value));
            }
        }
        trace!(target: DEBUG, "Set debug speed multiplier {} for {:?}, new_base={}", multiplier, being_ent, new_value);
        return;
    }
    let new_value = multiplier.max(0.0);
    cmd.spawn((
        SpeedModifier::new(being_ent, being_ent, new_value, ApplyMode::Mul),
        debug_hash,
    ));
    trace!(target: DEBUG, "Set debug speed multiplier {} for {:?}, new_base={}", multiplier, being_ent, new_value);
}

fn numpad_key_to_speed(key_code: KeyCode) -> Option<f32> {
    let digit = match key_code {
        KeyCode::Numpad0 => 0,
        KeyCode::Numpad1 => 1,
        KeyCode::Numpad2 => 2,
        KeyCode::Numpad3 => 3,
        KeyCode::Numpad4 => 4,
        KeyCode::Numpad5 => 5,
        KeyCode::Numpad6 => 6,
        KeyCode::Numpad7 => 7,
        KeyCode::Numpad8 => 8,
        KeyCode::Numpad9 => 9,
        _ => return None,
    };
    match digit {
        0 => Some(DEBUG_SPEED_MIN),
        1 => Some(1.0),
        2 => Some(2.0),
        3 => Some(4.0),
        4 => Some(8.0),
        5 => Some(16.0),
        6 => Some(32.0),
        7 => Some(45.0),
        8 => Some(50.0),
        9 => Some(DEBUG_SPEED_MAX),
        _ => None,
    }
}

#[allow(unused_parens, )]
pub fn debug_numpad_speed_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    debug_ui_config: Query<&DebugUiConfig, With<SettingsEntity>>,
    client_state: Res<State<ClientState>>,
    mut client_request_writer: MessageWriter<ClientDebugSetSpeedRequest>,
    mut cmd: Commands,
    local_controlled_being_query: Query<(Entity, &AppliedModifiers), (With<Being>, LocalHumanControlled, )>,
    mut debug_modi_query: Query<(Entity, &HashId, Option<&mut BaseValue>, ), (With<WalkStrength>, Without<Templ>, )>,
) {
    let Ok(debug_ui_config) = debug_ui_config.single() else {
        return;
    };
    if !debug_ui_config.enable_debug_menus {
        return;
    }

    let mut selected_speed: Option<f32> = None;
    for key_code in [
        KeyCode::Numpad0,
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::Numpad4,
        KeyCode::Numpad5,
        KeyCode::Numpad6,
        KeyCode::Numpad7,
        KeyCode::Numpad8,
        KeyCode::Numpad9,
    ] {
        if keys.just_pressed(key_code) {
            selected_speed = numpad_key_to_speed(key_code);
            break;
        }
    }

    let Some(speed) = selected_speed else {
        return;
    };
    let Ok((being_ent, applied_modifiers)) = local_controlled_being_query.single() else {
        return;
    };

    if *client_state.get() == ClientState::Connected {
        client_request_writer.write(ClientDebugSetSpeedRequest { being_ent, speed });
    } else {
        set_speed_multiplier(&mut cmd, being_ent, applied_modifiers, &mut debug_modi_query, speed);
    }
}

#[allow(unused_parens, )]
pub fn receive_client_debug_set_speed_request(
    mut cmd: Commands,
    debug_ui_config: Query<&DebugUiConfig, With<SettingsEntity>>,
    mut requests: MessageReader<FromClient<ClientDebugSetSpeedRequest>>,
    controlled_beings_query: Query<(&AppliedModifiers, ), ()>,
    mut debug_modi_query: Query<(Entity, &HashId, Option<&mut BaseValue>, ), (With<WalkStrength>, Without<Templ>, )>,
) {
    let Ok(debug_ui_config) = debug_ui_config.single() else {
        return;
    };
    if !debug_ui_config.client_debug {
        return;
    }
    for request in requests.read() {
        let being_ent = request.message.being_ent;
        let Ok((applied_modifiers, )) = controlled_beings_query.get(being_ent) else {
            continue;
        };
        set_speed_multiplier(&mut cmd, being_ent, applied_modifiers, &mut debug_modi_query, request.message.speed);
    }
}
