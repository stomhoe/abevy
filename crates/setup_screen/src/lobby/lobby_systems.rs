

use bevy::{prelude::*};
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use common::{common_components::StrId, common_states::*};
use game_common::game_common_states::GameSetupScreen;
use multiplayer_shared::{multiplayer_events::{HostStartedGameplay, AttemptHostServer}, multiplayer_resources::PendingGameStart};
use player_shared::{player_components::*, player_resources::PlayerData};
use crate::lobby::lobby_components::*;

pub fn host_setup(_: On<AttemptHostServer>, _cmd: Commands, ) {

}



#[allow(unused_parens, dead_code)]
pub fn host_on_server_start_failed(_cmd: Commands){

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
    asset_loading_state: Res<State<AssetLoading>>,
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
                    if *asset_loading_state.get() == AssetLoading::Finished {
                        game_phase.set(GamePhase::ActiveGame);
                        cmd.server_trigger(ToClients {
                            targets: SendTargets::All,
                            message: HostStartedGameplay,
                        });
                    } else {
                        cmd.insert_resource(PendingGameStart);
                        info!("Deferring game start until asset loading finishes");
                    }
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
    players: Query<(&StrId, &LobbyPlayerUiNode), With<Player>>,
    mut commands: Commands)
{
    if let Ok((player_name, player_name_entry)) = players.get(trigger.entity) {
        info!("Client `{}` disconnected", player_name);
        commands.entity(player_name_entry.0).try_despawn();
    } else {
        info!("Failed to get player name for disconnected client: {}", trigger.entity);
    }

}


#[allow(unused_parens)]
pub fn all_on_player_added(mut cmd: Commands,
    my_data: Res<PlayerData>,
    player_listing: Query<Entity, With<LobbyPlayerListing>>,
    query: Query<(Entity, &StrId),(Added<StrId>, With<Player>)>) {
    let Ok(player_listing) = player_listing.single() else {
        error!("Failed to get player listing");
        return;
    };

    for (player_ent, username) in query.iter() {
        if username == &my_data.username {
            cmd.entity(player_ent).insert(Mine);
        }

        let pne = cmd.spawn((
            ChildOf(player_listing),
            Node {
                width: Val::Percent(100.),
                height: Val::Px(50.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Text::new(username.to_string()),
            TextLayout::new_with_justify(Justify::Center),
        )).id();


        cmd.entity(player_ent).insert((
            LobbyPlayerUiNode(pne),
        ));
    }
}
