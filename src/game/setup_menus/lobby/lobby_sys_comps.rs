
use crate::{game::GamePhase, pregame_menus::{PreGameState}, AppState};

use bevy::{prelude::*, render::mesh::PlaneMeshBuilder};


use std::time::SystemTime;
use std::{collections::HashMap, net::UdpSocket};


#[derive(Component)]
pub enum LobbyButtonId {
  Start,
  Leave,
  CreateCharacter,
  LobbyJoinability,
  Ready,
}

#[derive(Component)]
pub enum LobbyLineEdit {Chat, LobbyName}


#[derive(Component)]
pub enum LobbySlider {ChatHistory, Settings}

const PROTOCOL_ID: u64 = 7;

pub fn setup(mut commands: Commands){

}

pub fn setup_for_host(mut commands: Commands) {

}


pub fn setup_for_client(mut commands: Commands, ) {
}

pub fn lobby_button_interaction(
    interaction_query: Query<
    (&Interaction, &LobbyButtonId),
    Changed<Interaction>,
    >,
    //mut app_exit_events: EventWriter<AppExit>,
    mut pregame_state: ResMut<NextState<PreGameState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut mp_game_state:  ResMut<NextState<GamePhase>>,
) 
{
    for (interaction, menu_button_action) in &interaction_query {
        
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                LobbyButtonId::Leave => {
                    app_state.set(AppState::PreGame);
                    pregame_state.set(PreGameState::MainMenu);

                }
                LobbyButtonId::Start =>  {
                    //todo chequear si todos están listos
                    mp_game_state.set(GamePhase::InGame);
                },
                LobbyButtonId::CreateCharacter => {},
                LobbyButtonId::Ready => {},
                LobbyButtonId::LobbyJoinability => {},
            }
        }
    }
}