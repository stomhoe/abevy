use bevy::asset::AssetServer;
use bevy::audio::AudioPlayer;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::enemy::enemy_components::Enemy;
use crate::game::enemy::*;
use crate::game::Player;
use crate::game::star::star_events::GameOver;
use crate::game::star::star_resources::*;

pub fn spawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.single().unwrap();

    for _ in 0..NUMBER_OF_ENEMIES {
        let x = rand::random::<f32>() * window.width();
        let y = rand::random::<f32>() * window.height();

        commands.spawn((
            StateScoped(AppState::Game),
            Sprite {
                image: asset_server.load("textures\\wear\\moss_short_tunic_icon.png"),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Enemy {
                direction: Vec2::new(
                    rand::random::<f32>() * 2.0 - 1.0, // Random x direction between -1 and 1
                    rand::random::<f32>() * 2.0 - 1.0, // Random y direction between -1 and 1
                )
                    .normalize(), // Normalize to ensure consistent speed
            },
        ));
    }
}
pub fn update_enemy_direction(
    mut enemy_query: Query<(&mut Transform, &mut Enemy), Without<Player>>,
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut game_over_event_writer: EventWriter<GameOver>,
    score: Res<Score>,

) {
    let window = window_query.single().unwrap();
    let _half_enemy_size: f32 = ENEMY_SIZE / 2.0;

    if let Ok((player_entity, player_transform)) = player_query.single_mut() {
        for (mut transform, mut enemy) in enemy_query.iter_mut() {
            let position = &mut transform.translation;

            enemy.direction = Vec2::new(
                player_transform.translation.x - position.x,
                player_transform.translation.y - position.y,
            )
                .normalize();

            position.x = position.x.clamp(0.0, window.width());
            position.y = position.y.clamp(0.0, window.height());

            if position.xy().distance(player_transform.translation.xy()) < 0.1 {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("sound/howlingwolf.ogg")),
                ));

                commands.entity(player_entity).despawn();

                game_over_event_writer.write(GameOver{score: score.count});
            }
        }
    }
}

pub fn enemy_movement(mut enemy_query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    for (mut transform, enemy) in enemy_query.iter_mut() {
        // Move the enemy in its direction at a speed of 100 units per second
        transform.translation.x += enemy.direction.x * 100.0 * time.delta_secs();
        transform.translation.y += enemy.direction.y * 100.0 * time.delta_secs();
    }
}