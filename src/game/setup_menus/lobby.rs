use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::{game::{setup_menus::lobby::{lobby_events::*, lobby_layout::*, lobby_sys_comps::*},  GamePhase, GameSetupType, }, AppState};

// Module lobby
mod lobby_sys_comps;
mod lobby_layout;
pub mod lobby_events;
pub struct LobbyPlugin;
#[allow(unused_parens)]
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            //..add_systems(Update, somesystem)
            .add_systems(
                OnEnter(AppState::GameSession),
                (
                    (layout_for_host, setup_for_host)
                        .run_if(in_state(GamePhase::Setup).and(in_state(GameSetupType::HostLobby))),

                    (layout_for_client, attempt_join_lobby)
                        .run_if(in_state(GamePhase::Setup).and(in_state(GameSetupType::JoinerLobby))),
                ),
            )
            .add_systems(Update, (
                (lobby_button_interaction).run_if(in_state(GamePhase::Setup).and(not(in_state(GameSetupType::Singleplayer)))),
                (setup_for_connected_client).run_if(client_just_connected)

            ))

//            .add_server_trigger::<ConnectedEvent>(Channel::Ordered)
            .add_client_trigger::<SendPlayerName>(Channel::Ordered)

            // .add_systems(OnEnter(SelfMpKind::Client), (layout_for_client).run_if(in_state(GamePhase::Setup)))


            .add_observer(spawn_clients)
            .add_observer(server_receive_player_name)

            //.add_server_trigger::<HostStartedGame>(Channel::Ordered)
            
        ;
    }
}