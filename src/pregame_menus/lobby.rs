use bevy::prelude::*;


use crate::{pregame_menus::{lobby::lobby_systems::*, PreGameState}, MpStatus};


mod lobby_styles;
// Module lobby
mod lobby_components;
mod lobby_systems;
mod lobby_layout;
//mod lobby_events;
pub struct LobbyPlugin;
#[allow(unused_parens)]
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app
            //..add_systems(Update, somesystem)
            .add_systems(OnEnter(PreGameState::Lobby), (setup))
            .add_systems(Update, (lobby_button_interaction).run_if(in_state(PreGameState::Lobby)))
        ;
    }
}