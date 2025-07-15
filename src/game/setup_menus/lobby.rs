use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::{game::{setup_menus::lobby::{lobby_events::*, lobby_layout::*, lobby_systems::*}, ClientSystems, GamePhase, GameSetupType }, AppState};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum JoiningState {#[default]Not, PreAttempt, PostAttempt,}


// Module lobby
mod lobby_systems;
pub mod lobby_components;
mod lobby_layout;
pub mod lobby_events;
pub struct LobbyPlugin;
#[allow(unused_parens)]
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(JoiningState::PreAttempt),
                (
                    (attempt_join_lobby)
                        .run_if(in_state(GamePhase::Setup).and(in_state(GameSetupType::JoinerLobby))),
                ),
            )
            .add_systems(
                OnEnter(AppState::GameSession),
                (
                    (layout_for_host, setup_for_host)
                        .run_if(in_state(GamePhase::Setup).and(in_state(GameSetupType::HostLobby))),
                    (layout_for_client)
                        .run_if(in_state(GamePhase::Setup).and(in_state(GameSetupType::JoinerLobby))),
                ),
            )
            .add_systems(Update, (
                (lobby_button_interaction).run_if(in_state(GamePhase::Setup).and(not(in_state(GameSetupType::Singleplayer)))),
                (client_on_connect_successful).run_if(client_just_connected),
                (client_on_connect_failed).run_if(
                    in_state(GamePhase::Setup)
                        .and(in_state(GameSetupType::JoinerLobby))
                        .and(in_state(JoiningState::PostAttempt))
                        .and(not(client_connecting))
                        .and(not(client_connected))
                )
                ,
                display_stuff.in_set(ClientSystems)
            ))
            .init_state::<JoiningState>()

//            .add_server_trigger::<ConnectedEvent>(Channel::Ordered)
            .add_client_trigger::<SendPlayerName>(Channel::Ordered)

            // .add_systems(OnEnter(SelfMpKind::Client), (layout_for_client).run_if(in_state(GamePhase::Setup)))


            .add_observer(add_player_comp)
            .add_observer(server_receive_player_name)

            //.add_server_trigger::<HostStartedGame>(Channel::Ordered)
            
        ;
    }
}