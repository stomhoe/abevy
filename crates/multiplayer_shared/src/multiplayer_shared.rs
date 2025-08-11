

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::shared::RepliconSharedPlugin;




pub const PROTOCOL_ID: u64 = 7;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct HostSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClientSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((RepliconSharedPlugin::default()))
    // .add_systems(OnExit(AppState::StatefulGameSession), (
    // clean_resources
    // ))
    .init_state::<GamePhase>()
    .init_state::<GameSetupType>()
    .init_state::<GameSetupScreen>()
    .init_state::<SimulationState>()

    .configure_sets(OnEnter(ConnectionAttempt::Triggered), (
        HostSystems.run_if(in_state(GameSetupType::HostLobby)),
    ))
    .configure_sets(OnEnter(AppState::StatefulGameSession), (
        HostSystems.run_if(in_state(GameSetupType::HostLobby)),
    ))

    ;
}

