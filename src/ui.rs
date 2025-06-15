use bevy::{input_focus::InputFocus, prelude::*};
use bevy_ui_text_input::TextSubmitEvent;

use crate::ui::{ui_systems::*};

// Module ui
pub mod ui_components;
mod ui_systems;
//pub mod ui_events;
pub mod ui_styles;
pub mod ui_utils;
pub mod ui_resources;
pub struct MyUiPlugin;
#[allow(unused_parens)]
impl Plugin for MyUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputFocus>()
            .add_event::<TextSubmitEvent>()
            .add_plugins(bevy_ui_text_input::TextInputPlugin)
            .add_systems(Update, (update_line_edits_text, button_change_color_on_mouse_action))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup))
            //.add_systems(Update, somesystem.runif(in_state(SomeStateType::Literal)))
        ;
    }
}