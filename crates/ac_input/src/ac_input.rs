use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{ac_input_actions::*, ac_input_systems::*};

pub fn plugin(app: &mut App) {
    app
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<PlayerInputContext>()
        .add_input_context::<BeingInputContext>()
        .add_systems(Startup, spawn_player_input_context);
}
