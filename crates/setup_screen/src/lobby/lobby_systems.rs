

use bevy::{ecs::world::OnDespawn, prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use common::{common_components::StrId, common_states::*};
use game_common::game_common_states::GameSetupScreen;
use multiplayer_shared::multiplayer_events::HostStartedGame;
use player::{player_components::*, player_resources::PlayerData};
use crate::lobby::lobby_components::*;

pub fn host_setup(cmd: Commands, ) {

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
) 
{
    for (interaction, menu_button_action) in &interaction_query {
        
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                LobbyButtonId::Leave => {
                    app_state.set(AppState::NoSession);

                }
                LobbyButtonId::Start =>  {
                    //todo chequear si todos están listos
                    info!("Starting game");
                    game_phase.set(GamePhase::ActiveGame);
                    cmd.server_trigger(ToClients {
                        mode: SendMode::Broadcast,
                        message: HostStartedGame,
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
    trigger: On<Despawn, Player>,
    //state:
    //created_character_query: Query<(&CreatedCharacter, ), ()>, 
    players: Query<(&StrId, &LobbyPlayerUiNode), With<Player>>,
    mut commands: Commands)
{
    let result = players.get(trigger.target());
    //let created_character = created_character_query.get(trigger.target());

    if let Ok((player_name, player_name_entry)) = result {
        info!("Client `{}` disconnected", player_name);
        commands.entity(player_name_entry.0).despawn();
    
        //TODO MARCAR SU BEING PARA DESPAWN PARA CUANDO EMPEIZA LA PARTIDA
        //ASI SI SE REUNE PUEDE RECUPERARLO EN SU ESTADO ORIGINAL
    } else {
        info!("Failed to get player name for disconnected client: {}", trigger.target());
        return;
    }

}


#[allow(unused_parens)]
pub fn all_on_player_added(mut cmd: Commands, 
    my_data: Res<PlayerData>,
    player_listing: Single<Entity, With<LobbyPlayerListing>>, 
    query: Query<(Entity, &StrId),(Added<StrId>, With<Player>)>) {
    
    for (player_ent, username) in query.iter() {
        if username == &my_data.username {
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
            Text::new(username.to_string()),
            TextLayout::new_with_justify(JustifyText::Center),
        )).id();


        cmd.entity(player_ent).insert((
            LobbyPlayerUiNode(pne), 
        ));
    }
}

