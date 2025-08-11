use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;

use crate::{common::common_components::{DisplayName, EntityPrefix}, game::{multiplayer::{client_systems::*, host_systems::*, multiplayer_events::*}, GamePhase, GameSetupType, }, AppState};

// Module multiplayer
pub mod multiplayer_components;
mod host_systems;
mod client_systems;
pub mod multiplayer_events;
pub mod multiplayer_utils;
mod multiplayer_resources;




pub struct MpPlugin;
#[allow(unused_parens, path_statements)]
impl Plugin for MpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((RepliconPlugins, RepliconRenetPlugins, ))
            
            .add_observer(host_on_player_connect)
            .add_observer(host_receive_client_name)
            

            .add_systems(OnExit(AppState::StatefulGameSession), (
                clean_resources
            ))
            .add_systems(
                OnEnter(ConnectionAttempt::Triggered),
                (
                    (attempt_host,).in_set(HostSystems),
                ),
            )
            .add_client_trigger::<SendPlayerName>(Channel::Ordered)

            .configure_sets(OnEnter(ConnectionAttempt::Triggered), (
                HostSystems.run_if(in_state(GameSetupType::HostLobby)),
            ))
            .configure_sets(OnEnter(AppState::StatefulGameSession), (
                HostSystems.run_if(in_state(GameSetupType::HostLobby)),
            ))
            .init_state::<ConnectionAttempt>()

            .replicate::<Name>()
            .replicate::<EntityPrefix>()
            .replicate::<DisplayName>()

        ;
    }
}

/*
    https://docs.rs/bevy_replicon/latest/bevy_replicon/shared/replication/replication_rules/trait.AppRuleExt.html#method.replicate_with
    
 .replicate_with((
        RuleFns::<Being>::default(),
        (RuleFns::<Transform>::default(), SendRate::Once),
    ))
*/