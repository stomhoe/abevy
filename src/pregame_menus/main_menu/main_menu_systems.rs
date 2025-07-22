use bevy::prelude::*;


use bevy_ui_text_input::{*,
};

use crate::game::multiplayer::ConnectionAttempt;
use crate::game::{GamePhase, GameSetupType};
use crate::pregame_menus::main_menu::main_menu_components::*;
use crate::ui::ui_components::CurrentText;
use crate::{AppState};
use crate::pregame_menus::main_menu::*;



pub fn setup(){

}


pub fn menu_button_interaction(
    interaction_query: Query<
    (&Interaction, &MainMenuButton),
    Changed<Interaction>,
    >,
    //mut app_exit_events: EventWriter<AppExit>,
    
    mut pregame_state: ResMut<NextState<PreGameState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut lobby_state: ResMut<NextState<ConnectionAttempt>>,
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_setup_type: ResMut<NextState<GameSetupType>>,
     
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MainMenuButton::QuickStart => {
                    app_state.set(AppState::StatefulGameSession);
                }
                MainMenuButton::Host => {
                    game_setup_type.set(GameSetupType::HostLobby);
                    game_phase.set(GamePhase::Setup);
                    lobby_state.set(ConnectionAttempt::Triggered);//TODO mover esto a algún botón del lobby para el host
                    app_state.set(AppState::StatefulGameSession);
                }
                MainMenuButton::Join => {
                    game_setup_type.set(GameSetupType::JoinerLobby);
                    game_phase.set(GamePhase::Setup);
                    lobby_state.set(ConnectionAttempt::Triggered);
                }
                MainMenuButton::Settings => {
                    pregame_state.set(PreGameState::Settings);
                }
            } 
        }
    }
}

pub fn handle_line_edits_interaction(
    mut events: EventReader<TextSubmitEvent>,
    mut line_edit_query: Query<(&mut CurrentText, &mut TextInputPrompt, &mut Outline), With<MainMenuIpLineEdit>>,
) {
    for event in events.read() {
        if let Ok((mut curr_text, mut input_prompt, mut outline)) = line_edit_query.get_mut(event.entity) {
            
            let (valid, prompt) = if event.text.contains(':') {
                (
                    event.text.parse::<std::net::SocketAddr>().is_ok(),
                    "IP:PORT?",
                )
            } else {
                (
                    event.text.parse::<std::net::Ipv4Addr>().is_ok(),
                    "IP address?",
                )
            };

            curr_text.0 = event.text.clone();
            if valid {
                outline.color = bevy::color::palettes::css::LIGHT_GOLDENROD_YELLOW.into();
            } else {
                input_prompt.text = prompt.to_string();
                outline.color = bevy::color::palettes::css::DARK_RED.into();
            }
        }
    }
}