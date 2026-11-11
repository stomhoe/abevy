use bevy::prelude::*;


use ui_shared::SubmitText;
use common::common_states::*;
use multiplayer_shared::{multiplayer_events::{AttemptHostServer, JoinServer}, multiplayer_resources::TargetJoinServer};
use ui_shared::ui_components::CurrentText;

use crate::main_menu_components::{MainMenuButton, MainMenuIpLineEdit};





#[allow(unused_parens, )]
pub fn menu_button_interaction(
    activation: On<bevy::ui_widgets::Activate>,
    mut cmd: Commands,
    interaction_query: Query<(&MainMenuButton, ), ()>,

    mut pregame_state: ResMut<NextState<PreGameState>>,
    mut ____________game_phase: ResMut<NextState<GamePhase>>,
) {
    let Ok((menu_button_action, )) = interaction_query.get(activation.entity) else { return; };
            match menu_button_action {
                MainMenuButton::QuickStart => {
                    cmd.trigger(AttemptHostServer::default());
                }
                MainMenuButton::Host => {
                    cmd.trigger(AttemptHostServer::default());
                }
                MainMenuButton::Join => {
                    cmd.trigger(JoinServer::default());

                }
                MainMenuButton::Settings => {
                    pregame_state.set(PreGameState::Settings);
                }
            }
}

pub fn handle_line_edits_interaction(
    mut cmd: Commands,
    mut events: MessageReader<SubmitText>,
    mut line_edit_query: Query<(&mut CurrentText, &mut Outline), With<MainMenuIpLineEdit>>,

) {
    for event in events.read() {
        if let Ok((mut curr_text, mut outline)) = line_edit_query.get_mut(event.entity) {

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
                let _ = prompt;
                outline.color = bevy::color::palettes::css::DARK_RED.into();

                cmd.remove_resource::<TargetJoinServer>();
            }
        }
    }
}
