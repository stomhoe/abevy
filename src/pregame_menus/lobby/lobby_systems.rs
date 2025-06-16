
use bevy::prelude::*;
use crate::{pregame_menus::{lobby::{lobby_components::{LobbyButtonId, LobbyLineEdit}, lobby_layout::{do_shared_layout, layout_for_client, layout_for_host, SharedLayout}, lobby_styles::lobby_button}, PreGameState}, ui::ui_components::LineEdit, AppState, MpStatus};


pub fn setup(mut commands: Commands, mp_state: Res<State<MpStatus>>) {
    let shared_layout = do_shared_layout(&mut commands);
    
    match mp_state.get() {
        MpStatus::Host => setup_for_host(&mut commands, &shared_layout),
        MpStatus::Client => setup_for_client(&mut commands, &shared_layout),
        _ => {}
        
    }
}

pub fn setup_for_host(mut commands: &mut Commands, shared_layout: &SharedLayout) {

    layout_for_host(&mut commands, &shared_layout);

    
}


pub fn setup_for_client(mut commands: &mut Commands, shared_layout: &SharedLayout) {
    layout_for_client(&mut commands, &shared_layout);
}

pub fn lobby_button_interaction(
    interaction_query: Query<
    (&Interaction, &LobbyButtonId),
    Changed<Interaction>,
    >,
    //mut app_exit_events: EventWriter<AppExit>,
    mut pregame_state: ResMut<NextState<PreGameState>>,
    mut app_state: ResMut<NextState<AppState>>,
) 
{
    for (interaction, menu_button_action) in &interaction_query {
        
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                LobbyButtonId::Leave => {
                    pregame_state.set(PreGameState::MainMenu)
                }
                LobbyButtonId::Start =>  {
                    //todo chequear si todos están listos
                    app_state.set(AppState::Game);
                },
                LobbyButtonId::CreateCharacter => {},
                LobbyButtonId::Ready => {},
                LobbyButtonId::LobbyJoinability => {},
            }
        }
    }
}