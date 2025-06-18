use bevy::asset::AssetServer;
use bevy::audio::AudioPlayer;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::game::Player;
use crate::game::star::star_components::*;
use crate::game::star::star_events::*;
use crate::game::star::star_resources::*;
use crate::game::star::*;

pub fn update_score(score: Res<Score>){
    if score.is_changed(){
        println!("Score: {}", score.count);

    }
}


pub fn player_hit_star(

    mut commands: Commands,
    mut score: ResMut<Score>,
    player_query: Query<&Transform, With<Player>>,
    query: Query<(Entity, &Transform), With<Star>>,
    asset_server: Res<AssetServer>,
)
{
    if let Ok(player_transform) = player_query.single() {

        for (entity, transform) in query.iter() {
            if transform.translation.xy().distance(player_transform.translation.xy()) < 9.9 {
                commands.entity(entity).despawn();
                score.count += 1;
                commands.spawn((
                    AudioPlayer::new(asset_server.load("sound/howlingwolf.ogg")),
                ));
            }
        }

    }
}

pub fn tick_start_spawn_timer(mut star_timer: ResMut<StarSpawnTimer>, time: Res<Time>) {
    star_timer.timer.tick(time.delta());
}

pub fn spawn_stars_over_time(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut star_timer: ResMut<StarSpawnTimer>,)
{
    if star_timer.timer.finished() {
        let window = window_query.single().unwrap();
        let x = rand::random::<f32>() * window.width();
        let y = rand::random::<f32>() * window.height();

        commands.spawn((
            StateScoped(AppState::Game),
            Sprite {
                image: asset_server.load("textures/world/bushes/bush.png"),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Star {}
        )
        );
    }
}

pub fn spawn_stars(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
){
    let window = window_query.single().unwrap();
    for _ in 0..NUMBER_OF_STARS {
        let x = rand::random::<f32>() * window.width();
        let y = rand::random::<f32>() * window.height();

        commands.spawn((
            StateScoped(AppState::Game),
            Sprite {
                image: asset_server.load("textures/world/bushes/bush.png"),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            Star {},
        ));
    }
}

pub fn handle_game_over(mut event_reader: EventReader<GameOver>){
    for event in event_reader.read(){
        println!("Game Over! Score: {}", event.score);
    }
}