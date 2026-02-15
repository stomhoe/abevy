use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;
use common::common_states::{AppState, };
use multiplayer_shared::{multiplayer_events::SendUsername, };

use crate::client_systems::*;




#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((ClientPlugin, ClientMessagePlugin, RepliconRenetClientPlugin ))

    .add_observer(attempt_join)

    .add_systems(
        OnExit(ClientState::Connected),
        (
            client_cleanup,
        )
    )
    //.add_systems(Update, add_mine_to_player)

    .add_systems(OnEnter(ClientState::Connected), (client_on_connect_successful, ))
    .add_systems(OnEnter(ClientState::Disconnected), (client_on_disconnect.run_if(not(in_state(ServerState::Running)))))



    .add_observer(client_on_game_started)

    ;
}
