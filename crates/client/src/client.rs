use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;
use common::common_states::{AppState, ConnectionAttempt, };
use multiplayer_shared::{multiplayer_events::SendUsername, multiplayer_shared::ClientSystems};

use crate::client_systems::*;




#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((ClientPlugin, ClientMessagePlugin, RepliconRenetClientPlugin ))
    

    .add_systems(
        OnEnter(ConnectionAttempt::Triggered),
        (
            (client_init_resources, attempt_join).chain(),
        ).in_set(ClientSystems),
    )
    .add_systems(
        OnExit(AppState::StatefulGameSession),
        (
            client_cleanup,
        ).in_set(ClientSystems),
    )
    .add_systems(OnEnter(ClientState::Connected), (client_on_connect_succesful)) 
    .add_systems(OnEnter(ClientState::Disconnected), (client_on_disconnect.run_if(not(in_state(ServerState::Running))))) 



    .add_observer(client_on_game_started)

    ;
}