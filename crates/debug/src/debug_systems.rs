use bevy::prelude::*;
use bevy_fps_counter::FpsCounter;
use modifier::{modifier_components::*, modifier_move_components::Speed};
use being::being_components::{Being, ControlledLocally};

#[allow(unused_parens)]
pub fn debug_increase_speed(
    keys: Res<ButtonInput<KeyCode>>,
    my_being_query: Query<(&Being), (With<ControlledLocally>)>,
    mut query: Query<(&ModifierTarget, &mut EffectiveValue),(With<Speed>, )>,
) {
    if keys.pressed(KeyCode::NumpadAdd) {
        query.iter_mut().for_each(|(target, mut val)| {
            if my_being_query.get(target.0).is_ok() {
                val.0 *= 1.1;
            }
        });
    } else if keys.pressed(KeyCode::NumpadSubtract) {
        query.iter_mut().for_each(|(target, mut val)| {
            if my_being_query.get(target.0).is_ok() {
                val.0 *= 0.9;
            }
        });
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