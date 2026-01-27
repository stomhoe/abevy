
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::{AppState, GamePhase, };

use crate::lobby::{lobby_layout::*, lobby_systems::*};




// Module lobby
pub mod lobby_components;
mod lobby_systems;
mod lobby_layout;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_observer(host_setup)
        .add_systems(
            OnEnter(ServerState::Running),
            (
                layout_for_host.run_if(in_state(GamePhase::Setup)),
            ),
        )
        .add_systems(
            OnEnter(GamePhase::ActiveGame),
            (
                remove_player_name_ui_entry
            ),
        )
        .add_systems(
            OnEnter(ClientState::Connected),
            (
                (layout_for_client, ).run_if(in_state(GamePhase::Setup)),
            ),
        )
        .add_systems(
            Update,
            (
                lobby_button_interaction,
                all_on_player_added,
            ).run_if(
                in_state(GamePhase::Setup)
                .and(in_state(AppState::StatefulGameSession))
                .and(
                    not(in_state(ClientState::Disconnected))//is client
                    .or(in_state(ServerState::Running))//is host
                ),
            ),
        )


        .add_observer(on_player_disconnect)

        
    ;
}