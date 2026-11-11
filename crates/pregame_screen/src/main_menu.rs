use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::on_message;
use common::common_states::*;
use ui_shared::SubmitText;

use crate::{main_menu_layout::*, main_menu_systems::*};

pub fn plugin(app: &mut App) {
    app
    .add_observer(menu_button_interaction)
    .add_systems(OnEnter(AppState::NoSession), (layout).run_if(in_state(PreGameState::MainMenu)))
    .add_systems(Update, (
        handle_line_edits_interaction.run_if(on_message::<SubmitText>),
    ).run_if(in_state(PreGameState::MainMenu)))
    ;
}
