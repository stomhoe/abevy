use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{ac_input_actions::*, ac_input_egui_holders::*, ac_input_systems::*, player_action_requests};

pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            EnhancedInputPlugin,
            player_action_requests::melee_attack_plugin,
            player_action_requests::item_pickup_plugin,
        ))
        .add_input_context::<DebugInputContext>()
        .add_input_context::<BeingDirectControlInputContext>()
        .add_systems(Startup, (spawn_egui_holders, spawn_input_contexts).chain())
        .add_systems(Update, (toggle_simulation, add_being_input_context, sync_egui_input_holders, make_observers_be_children_of_egui_holder));
}
