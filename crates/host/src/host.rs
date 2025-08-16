#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetServerPlugin;
use common::common_states::{AppState, ConnectionAttempt};
use game_common::game_common::GameplaySystems;
use multiplayer_shared::multiplayer_shared::HostSystems;

use crate::host_systems::*;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_plugins((ServerPlugin::default(), ServerEventPlugin, RepliconRenetServerPlugin, ))
            
    .add_observer(host_on_player_connect)
    .add_observer(host_receive_client_name)
    

    .add_systems(Update, (
        host_on_player_added.in_set(HostSystems),
    ))
   
   .add_systems(
        OnExit(AppState::StatefulGameSession),
        (
            server_cleanup,
        ).in_set(HostSystems),
    )
    .add_systems(
        OnEnter(ConnectionAttempt::Triggered),
        (
            (attempt_host,).in_set(HostSystems),
        ),
    )

    ;
}
