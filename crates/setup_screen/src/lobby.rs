
use bevy::prelude::*;
use bevy_replicon::prelude::server_just_started;
use common::common_states::{AppState, ConnectionAttempt, GamePhase, GameSetupType};
use multiplayer_shared::multiplayer_shared::{ClientSystems, HostSystems};

use crate::lobby::{lobby_layout::*, lobby_systems::*};




// Module lobby
pub mod lobby_components;
mod lobby_systems;
mod lobby_layout;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(
            OnEnter(ConnectionAttempt::Triggered),
            (
                (layout_for_host, host_setup).in_set(HostSystems),
            ),
        )
        .add_systems(
            OnEnter(GamePhase::ActiveGame),
            (
                remove_player_name_ui_entry
            ),
        )
        .add_systems(
            OnEnter(AppState::StatefulGameSession),
            (
                (layout_for_client, ).in_set(ClientSystems),
            ),
        )
        .add_systems(Update, (
            (lobby_button_interaction, all_on_player_added,
            ).run_if(in_state(GamePhase::Setup).and(in_state(AppState::StatefulGameSession)).and(not(in_state(GameSetupType::Singleplayer)))),
            
            (host_on_server_start_successful).run_if(server_just_started)
            
        ))


        .add_observer(on_player_disconnect)

        
    ;
}