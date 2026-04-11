use {crate::ui_systems::*, bevy::input_focus::InputFocus, bevy_ui_text_input::{SubmitText, TextInputPlugin}};
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::prelude::*;

#[allow(unused_parens)]
pub fn plugin (app: &mut App) {
    app
        .init_resource::<InputFocus>()
        .add_message::<SubmitText>()
        .add_plugins((TextInputPlugin, ))
        .add_systems(Update, (update_line_edits_text.run_if(resource_changed::<InputFocus>), button_change_color_on_mouse_action))
    ;
}
