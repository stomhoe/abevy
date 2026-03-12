use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{ac_input_actions::*, ac_input_systems::*};

pub fn plugin(app: &mut App) {
    app
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<DebugInputContext>()
        .add_input_context::<BeingDirectControlInputContext>()
        .add_systems(Startup, spawn_input_contexts)
        .add_systems(Update, add_being_input_context);
}
