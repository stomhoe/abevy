
use crate::{common::common_components::DisplayName, game::{multiplayer::multiplayer_utils, player::player_components::{HostPlayer, Player}, setup_menus::lobby::{lobby_components::{LobbyPlayerListing, LobbyPlayerUiNode}, }, GamePhase, GameSetupScreen}, pregame_menus::{main_menu::main_menu_components::MainMenuIpLineEdit, PreGameState}, ui::ui_components::CurrentText, AppState};

use bevy::{ecs::world::OnDespawn, prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon_renet::{
    netcode::
        NetcodeClientTransport
    ,
    renet::{RenetClient, RenetServer},
};
use bevy_ui_text_input::TextInputContents;

use std::{mem, net::{Ipv4Addr, SocketAddr}, time::SystemTime, net::UdpSocket};



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


pub fn host_setup(mut commands: Commands, mut app_state: ResMut<NextState<AppState>>,){

}

#[allow(unused_parens, dead_code)]
pub fn host_on_server_start_successful(mut commands: Commands){

}

#[allow(unused_parens, dead_code)]
pub fn host_on_server_start_failed(mut commands: Commands){

}




#[allow(unused_parens)]
pub fn remove_player_name_ui_entry(mut commands: Commands, mut query: Query<(Entity),(With<LobbyPlayerUiNode>)>) {
    for ent in query.iter() {
        commands.entity(ent).remove::<LobbyPlayerUiNode>();
    }
}

pub fn lobby_button_interaction(
    mut cmd: Commands,
    interaction_query: Query<(&Interaction, &LobbyButtonId), Changed<Interaction>,>,
    mut game_setup_screen: ResMut<NextState<GameSetupScreen>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_phase:  ResMut<NextState<GamePhase>>,
    mut server: Option<ResMut<RenetServer>>,
    mut client: Option<ResMut<RenetClient>>,
) 
{
    for (interaction, menu_button_action) in &interaction_query {
        
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                LobbyButtonId::Leave => {
                    app_state.set(AppState::PreGame);

                    if let Some(ref mut server) = server {
                        multiplayer_utils::stop_server(&mut cmd, server);
                    }

                    if let Some(ref mut client) = client {
                        multiplayer_utils::disconnect_from_server(&mut cmd, client);
                    }
                }
                LobbyButtonId::Start =>  {
                    //todo chequear si todos están listos
                    game_phase.set(GamePhase::ActiveGame);
                },
                LobbyButtonId::CreateCharacter => {
                    game_setup_screen.set(GameSetupScreen::CharacterCreation);
                },
                LobbyButtonId::Ready => {},
                LobbyButtonId::LobbyJoinability => {},
            }
        }
    }
}

pub fn on_player_disconnect(
    trigger: Trigger<OnDespawn, Player>, 
    players: Query<(&DisplayName, &LobbyPlayerUiNode), With<Player>>,
    mut commands: Commands)
{
    let result = players.get(trigger.target());

    if let Ok((player_name, player_name_entry)) = result {
        info!("Client `{}` disconnected", player_name.0);
        commands.entity(player_name_entry.0).despawn();
    } else {
        info!("Failed to get player name for disconnected client: {}", trigger.target());
        return;
    }

}

#[allow(unused_parens)]
pub fn dbg_display_stuff(
    query: Query<(&DisplayName),(With<Player>)>

) {
    for (player_name) in query.iter() {
        info!("Player name: {}", player_name.0);
    }
}

#[allow(unused_parens)]
pub fn on_player_added(mut cmd: Commands, player_listing: Single<Entity, With<LobbyPlayerListing>>, query: Query<(Entity, &DisplayName),(Added<DisplayName>, With<Player>)>) {
    
    for (ent, player_name) in query.iter() {
        let pne = cmd.spawn((
            ChildOf(*player_listing),
            Node {
                width: Val::Percent(100.),
                height: Val::Px(50.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Text::new(player_name.0.clone()),
            TextLayout::new_with_justify(JustifyText::Center),
        )).id();
        cmd.entity(ent).insert((
            StateScoped(AppState::StatefulGameSession),
            LobbyPlayerUiNode(pne)
        ));
    }
}

