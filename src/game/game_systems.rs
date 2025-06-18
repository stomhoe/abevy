use bevy::asset::AssetServer;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::window::PrimaryWindow;
use bevy::prelude::*;
use crate::game::player::player_components::Player;
use crate::game::{SimulationState};

pub fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.single().unwrap();

    commands.spawn((
        Player {
            //name: "hola".to_string(),
            ..default()
        },
        Sprite {
            image: asset_server.load("textures\\wear\\moss_short_tunic_icon.png"),
            ..default()
        },
        Transform::from_translation(Vec3::new(window.width() / 2.0, window.height() / 2.0, 0.0)),
    ));
}



pub fn toggle_simulation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<SimulationState>>, mut next_state: ResMut<NextState<SimulationState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match current_state.get() {
            SimulationState::Paused => {
                println!("Switching to Running state");
                next_state.set(SimulationState::Running)
            },
            SimulationState::Running => {
                println!("Switching to Paused state");
                next_state.set(SimulationState::Paused)
            },
        }
    }
}