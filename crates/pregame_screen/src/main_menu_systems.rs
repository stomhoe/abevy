use bevy::prelude::*;


use bevy_ui_text_input::{*,
};
use common::common_states::*;
use multiplayer_shared::{multiplayer_events::{HostServer, JoinServer}, multiplayer_resources::TargetJoinServer};
use ui_shared::ui_components::CurrentText;

use crate::main_menu_components::{MainMenuButton, MainMenuIpLineEdit};





pub fn menu_button_interaction(
    mut cmd: Commands,
    interaction_query: Query<(&Interaction, &MainMenuButton),
    Changed<Interaction>,>,
    
    mut pregame_state: ResMut<NextState<PreGameState>>,
    mut game_phase: ResMut<NextState<GamePhase>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MainMenuButton::QuickStart => {
                    //assets_loading_state.set(AssetsLoadingState::LoadingReplicatedCollections);
                    //game_phase.set(GamePhase::Setup);


                }
                MainMenuButton::Host => {
                    cmd.trigger(HostServer::default());
                }
                MainMenuButton::Join => {
                    cmd.trigger(JoinServer::default());

                }
                MainMenuButton::Settings => {
                    pregame_state.set(PreGameState::Settings);
                }
            } 
        }
    }
}

pub fn handle_line_edits_interaction(
    mut cmd: Commands, 
    mut events: MessageReader<SubmitText>,
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
                let Ok(target) = TargetJoinServer::new(event.text.clone())
                else { continue };
                
                cmd.insert_resource(target);

            } else {
                input_prompt.text = prompt.to_string();
                outline.color = bevy::color::palettes::css::DARK_RED.into();

                cmd.remove_resource::<TargetJoinServer>();
            }
        }
    }
}