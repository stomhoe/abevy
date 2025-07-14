use bevy::prelude::*;


use bevy_ui_text_input::{*,
};

use crate::game::{GameMp, GamePhase, SelfMpKind};
use crate::{AppState};
use crate::pregame_menus::main_menu::*;

#[derive(Component)]
pub enum MainMenuButton {QuickStart, Host, Join, Settings}


#[derive(Component)]
pub enum MainMenuLineEdit {Ip}

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
    mut game_phase: ResMut<NextState<GamePhase>>,
    mut game_mp: ResMut<NextState<GameMp>>,
    mut self_mp_kind: ResMut<NextState<SelfMpKind>>,
     line_edit_query: Query<(&TextInputContents, &MainMenuLineEdit)>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MainMenuButton::QuickStart => {
                    app_state.set(AppState::GameSession);
                    game_mp.set(GameMp::Singleplayer);
                }
                MainMenuButton::Host => {
                    app_state.set(AppState::GameSession);
                    game_mp.set(GameMp::Multiplayer);
                    self_mp_kind.set(SelfMpKind::Host);
                    game_phase.set(GamePhase::Setup);
                }
                MainMenuButton::Join => {
                    app_state.set(AppState::GameSession);
                    game_mp.set(GameMp::Multiplayer);
                    self_mp_kind.set(SelfMpKind::Client);
                    game_phase.set(GamePhase::Setup);
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
    line_edit_query: Query<&MainMenuLineEdit>,
) {
    for event in events.read() {
        let entity = event.entity;
        if let Ok(line_edit_type) = line_edit_query.get(entity) {
            match line_edit_type {
                MainMenuLineEdit::Ip => {
                    if event.text.len() > 15 {
                        continue;
                    }
                    

                }
            }
        }
    }
}