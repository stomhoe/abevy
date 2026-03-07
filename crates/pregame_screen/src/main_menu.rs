use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::on_message;
use common::common_states::*;
use bevy_ui_text_input::SubmitText;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app
    .add_systems(OnEnter(AppState::NoSession), (layout).run_if(in_state(PreGameState::MainMenu)))
    .add_systems(Update, (
        menu_button_interaction,
        handle_line_edits_interaction.run_if(on_message::<SubmitText>),
    ).run_if(in_state(PreGameState::MainMenu)))
    ;
}
