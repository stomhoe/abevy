
use crate::{common::common_components::DisplayName, game::{faction::faction_resources::FactionEntityMap, multiplayer::multiplayer_utils, player::{player_components::{CreatedCharacter, OfSelf, Player}, player_resources::PlayerData}, setup_menus::lobby::{lobby_components::{LobbyPlayerListing, LobbyPlayerUiNode}, lobby_events::HostStartedGame}, GamePhase, GameSetupScreen}, AppState};

use bevy::{ecs::world::OnDespawn, prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use bevy_replicon_renet::renet::{RenetClient, RenetServer};




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


pub fn host_setup(cmd: Commands, fac_map: ResMut<FactionEntityMap>) {

}

#[allow(unused_parens, dead_code)]
pub fn host_on_server_start_successful(
    cmd: Commands, 
) {
    

}

#[allow(unused_parens, dead_code)]
pub fn host_on_server_start_failed(commands: Commands){

}




#[allow(unused_parens)]
pub fn remove_player_name_ui_entry(mut commands: Commands, query: Query<(Entity),(With<LobbyPlayerUiNode>)>) {
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
                    info!("Starting game");
                    game_phase.set(GamePhase::ActiveGame);
                    cmd.server_trigger(ToClients {
                        mode: SendMode::Broadcast,
                        event: HostStartedGame,
                    });
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
    created_character_query: Query<(&CreatedCharacter, ), ()>, 
    players: Query<(&DisplayName, &LobbyPlayerUiNode), With<Player>>,

    mut commands: Commands)
{
    let result = players.get(trigger.target());
    let created_character = created_character_query.get(trigger.target());

    if let Ok((player_name, player_name_entry)) = result {
        info!("Client `{}` disconnected", player_name);
        commands.entity(player_name_entry.0).despawn();
        if let Ok((created_character, )) = created_character {
            commands.entity(created_character.0).despawn();
        }
    } else {
        info!("Failed to get player name for disconnected client: {}", trigger.target());
        return;
    }

}


#[allow(unused_parens)]
pub fn all_on_player_added(mut cmd: Commands, 
    my_data: Res<PlayerData>,
    player_listing: Single<Entity, With<LobbyPlayerListing>>, query: Query<(Entity, &DisplayName),(Added<DisplayName>, With<Player>)>) {
    
    for (player_ent, player_name) in query.iter() {
        if player_name.0 == my_data.name {
            cmd.entity(player_ent).insert(OfSelf);
        } 

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


        cmd.entity(player_ent).insert((
            
            LobbyPlayerUiNode(pne),
        ));
    }
}

#[allow(unused_parens)]
pub fn on_host_started_game(trigger: Trigger<HostStartedGame>, commands: Commands, mut state: ResMut<NextState<GamePhase>>) {
    info!(target: "lobby", "Host started game event received, transitioning to GamePhase::ActiveGame");
    state.set(GamePhase::ActiveGame);

}
