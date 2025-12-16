
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::{AppState, GamePhase, };
use multiplayer_shared::multiplayer_shared::{ClientSystems, HostSystems};

use crate::lobby::{lobby_layout::*, lobby_systems::*};




// Module lobby
pub mod lobby_components;
mod lobby_systems;
mod lobby_layout;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_observer(layout_for_host)
        .add_observer(host_setup)
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
        .add_systems(
            Update,
            (
                lobby_button_interaction,
                all_on_player_added,
            ).run_if(
                in_state(GamePhase::Setup)
                .and(in_state(AppState::StatefulGameSession))
                .and(
                    in_state(ClientState::Connected)
                    .or(in_state(ServerState::Running))
                ),
            ),
        )


        .add_observer(on_player_disconnect)

        
    ;
}