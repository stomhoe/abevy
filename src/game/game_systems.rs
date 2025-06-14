use bevy::asset::AssetServer;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::window::PrimaryWindow;
use bevy::prelude::*;
use crate::AppState;
use crate::game::{Player, SimulationState};

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

pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.single().unwrap();

    commands.spawn((
        StateScoped(AppState::Game),
        Camera2d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(window.width() / 2.0, window.height() / 2.0, 0.0),
            ..default()
        },
    ));
}

pub fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(mut player_transform) = player_query.single_mut() {
        let mut direction = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW) {direction.y += 1.0;}
        if keys.pressed(KeyCode::KeyS) {direction.y -= 1.0;}
        if keys.pressed(KeyCode::KeyA) {direction.x -= 1.0;}
        if keys.pressed(KeyCode::KeyD) {direction.x += 1.0;}
        if direction != Vec3::ZERO {
            direction = direction.normalize() * 150.0 * time.delta_secs(); // Speed of the player
            player_transform.translation += direction;
        }
    }
}
pub fn confine_player_to_window(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().unwrap();
    if let Ok(mut player_transform) = player_query.single_mut() {
        let position = &mut player_transform.translation;
        position.x = position.x.clamp(0.0, window.width());
        position.y = position.y.clamp(0.0, window.height());
    }
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