use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::{ClientMessageAppExt, ServerMessageAppExt, ClientState, ServerState};

use crate::{ac_input_actions::*, ac_input_egui_holders::*, ac_input_systems::*, player_action_requests};

pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            EnhancedInputPlugin,
            player_action_requests::melee_attack_plugin,
            player_action_requests::item_pickup_plugin,
            player_action_requests::debug_increase_speed_plugin,
            player_action_requests::debug_decrease_speed_plugin,
        ))
        .add_input_context::<DebugInputContext>()
        .add_input_context::<BeingDirectControlInputContext>()
        .add_systems(Startup, (spawn_egui_holders, spawn_input_contexts).chain())
        .add_systems(
            Update,
            (
                toggle_simulation,
                add_being_input_context,
                sync_egui_input_holders,
                make_observers_be_children_of_egui_holder,
                receive_toggle_simulation_request
                    .run_if(on_message::<bevy_replicon::prelude::FromClient<ClientToggleSimulationRequest>>)
                    .run_if(in_state(ServerState::Running)),
                receive_simulation_state_from_server
                    .run_if(on_message::<SyncSimulationState>)
                    .run_if(in_state(ClientState::Connected)),
            ),
        )
        .add_mapped_client_message::<ClientToggleSimulationRequest>(bevy_replicon::prelude::Channel::Unordered)
        .add_mapped_server_message::<SyncSimulationState>(bevy_replicon::prelude::Channel::Unordered);
}
