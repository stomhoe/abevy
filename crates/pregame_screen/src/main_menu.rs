use bevy::prelude::*;
use common::common_states::*;

use crate::{main_menu_layout::*, main_menu_systems::*};

pub fn plugin(app: &mut App) {
    app
    .add_systems(OnEnter(AppState::NoGameSession), (layout).run_if(in_state(PreGameState::MainMenu)))
    .add_systems(Update, (menu_button_interaction, handle_line_edits_interaction).run_if(in_state(PreGameState::MainMenu)))
    ;
}