#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetServerPlugin;
use common::common_states::{AppState, };
use game_common::game_common::GameplaySystems;

use crate::host_systems::*;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_plugins((ServerPlugin::default(), ServerMessagePlugin, RepliconRenetServerPlugin, ))
            
    .add_observer(host_on_player_connect)
    .add_observer(host_receive_client_name)
    .add_observer(attempt_host)
    
    



    .add_systems(
        OnEnter(ServerState::Running),
        (
            on_server_start_successful,
        ),
    )
   
   .add_systems(
        OnExit(AppState::StatefulGameSession),
        (
            server_cleanup,
        ).run_if(in_state(ServerState::Running)),
    )


    ;
}
