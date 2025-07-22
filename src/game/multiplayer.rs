use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;

use crate::{game::{multiplayer::{client_systems::*, host_systems::*, multiplayer_events::*}, ClientSystems, GamePhase, GameSetupType, HostSystems }, AppState};

// Module multiplayer
pub mod multiplayer_components;
mod host_systems;
mod client_systems;
pub mod multiplayer_events;
pub mod multiplayer_utils;
mod multiplayer_resources;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum ConnectionAttempt {#[default]Not, Triggered, PostAttempt,}

pub struct MpPlugin;
#[allow(unused_parens, path_statements)]
impl Plugin for MpPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((RepliconPlugins, RepliconRenetPlugins, ))
            
            .add_observer(receive_transf_from_client)
            .add_observer(host_on_player_connect)
            .add_observer(host_receive_client_name)
            

            .add_systems(OnExit(AppState::StatefulGameSession), (
                clean_resources
            ))
            .add_systems(
                OnEnter(ConnectionAttempt::Triggered),
                (
                    (attempt_host,).in_set(HostSystems),
                    (attempt_join,).in_set(ClientSystems),
                ),
            )
            .add_systems(Update, (
                client_on_connect_successful.run_if(client_just_connected),
                
                (client_on_connect_failed).run_if(
                    in_state(GamePhase::Setup)
                    .and(in_state(GameSetupType::JoinerLobby))
                    .and(in_state(ConnectionAttempt::PostAttempt))
                    .and(not(client_connecting))
                    .and(not(client_connected))
                ),
                client_on_disconnect.run_if((client_just_disconnected),),
            ))
            .add_server_trigger::<TransformFromServer>(Channel::Unreliable)
            .add_client_trigger::<TransformFromClient>(Channel::Unreliable)
            .add_client_trigger::<SendPlayerName>(Channel::Ordered)

            .configure_sets(OnEnter(ConnectionAttempt::Triggered), (
                HostSystems.run_if(in_state(GameSetupType::HostLobby)),
                ClientSystems.run_if(in_state(GameSetupType::JoinerLobby)),
            ))
            .configure_sets(OnEnter(AppState::StatefulGameSession), (
                HostSystems.run_if(in_state(GameSetupType::HostLobby)),
                ClientSystems.run_if(in_state(GameSetupType::JoinerLobby)),
            ))
            .init_state::<ConnectionAttempt>()

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