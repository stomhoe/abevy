
use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::{game::{multiplayer::{ConnectionAttempt, HostSystems}, player::player_components::CreatedCharacter, setup_menus::lobby::{lobby_events::*, lobby_layout::*, lobby_systems::*}, ClientSystems, GamePhase, GameSetupType, }, AppState};



// Module lobby
pub mod lobby_components;
mod lobby_systems;
mod lobby_layout;
mod lobby_events;
pub struct LobbyPlugin;
#[allow(unused_parens)]
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                OnEnter(ConnectionAttempt::Triggered),
                (
                    (layout_for_host, host_setup).in_set(HostSystems),
                ),
            )
            .add_systems(
                OnEnter(GamePhase::ActiveGame),
                (
                    remove_player_name_ui_entry
                ),
            )
            .add_systems(
                OnEnter(AppState::StatefulGameSession),
                (
                    (layout_for_client, ).in_set(ClientSystems),
                ),
            )
            .add_systems(Update, (
                (lobby_button_interaction, all_on_player_added,
                ).run_if(in_state(GamePhase::Setup).and(in_state(AppState::StatefulGameSession)).and(not(in_state(GameSetupType::Singleplayer)))),
                
                (host_on_server_start_successful).run_if(server_just_started)
                
                
                //,dbg_display_stuff.in_set(ClientSystems).run_if(on_timer(Duration::from_secs(3)))
            ))


            .add_observer(on_player_disconnect)

            .add_server_trigger::<HostStartedGame>(Channel::Ordered)
            .add_observer(on_host_started_game)
            .replicate::<CreatedCharacter>()
            
        ;
    }
}