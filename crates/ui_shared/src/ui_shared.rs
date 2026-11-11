use {crate::ui_systems::*, bevy::input_focus::InputFocus, crate::ui_components::SubmitText};
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::prelude::*;

#[allow(unused_parens)]
pub fn plugin (app: &mut App) {
    app
        .init_resource::<InputFocus>()
        .add_message::<SubmitText>()
        .add_systems(Update, apply_retro_font)
        .add_systems(Update, fit_menu_backgrounds)
        .add_observer(focus_line_edit_on_pointer_press)
        .add_systems(Update, (focus_line_edit_on_press, update_line_edits_text.run_if(resource_changed::<InputFocus>), button_change_color_on_mouse_action, submit_line_edit))
    ;
}
