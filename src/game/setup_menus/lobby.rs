use bevy::prelude::*;


use crate::{game::{setup_menus::lobby::{lobby_layout::*, lobby_sys_comps::*}, GameMp, GamePhase, SelfMpKind}, AppState};

// Module lobby
mod lobby_sys_comps;
mod lobby_layout;
//mod lobby_events;
pub struct LobbyPlugin;
#[allow(unused_parens)]
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            //..add_systems(Update, somesystem)
            .add_systems(OnEnter(GameMp::Multiplayer), (setup))
            .add_systems(
                OnEnter(AppState::GameDomain),
                (
                    layout_for_host
                        .run_if(in_state(GamePhase::Setup).and(in_state(SelfMpKind::Host))),
                    layout_for_client
                        .run_if(in_state(GamePhase::Setup).and(in_state(SelfMpKind::Client))),
                ),
            )

            .add_systems(OnEnter(SelfMpKind::Client), (layout_for_client).run_if(in_state(GamePhase::Setup)))

            .add_systems(Update, (lobby_button_interaction)
                .run_if(in_state(GamePhase::Setup).and(in_state(GameMp::Multiplayer))))
        ;
    }
}