#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetServerPlugin;
use ::common::*;

use crate::host_systems::*;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app

    .add_plugins((ServerPlugin::default(), ServerMessagePlugin, RepliconRenetServerPlugin, ))

    .add_observer(host_on_player_connect)
    .add_observer(host_receive_client_name)
    .add_observer(attempt_host)
    .add_observer(on_enter_setup)

    .add_systems(
        OnEnter(ServerState::Running),
        (
            on_server_start_successful,

        ).chain(),
    )
    .add_systems(
        OnEnter(AssetLoading::Finished),
        (
            on_assets_finish_loading,
        )
    )

   .add_systems(
        OnExit(AppState::StatefulGameSession),
        (
            server_cleanup,
        ).run_if(in_state(ServerState::Running)),
    )


    ;
}
