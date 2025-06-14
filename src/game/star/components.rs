use bevy::prelude::{Component, Resource, Timer, TimerMode};
use crate::game::star::STAR_SPAWN_INTERVAL;

#[derive(Component)]
pub struct Star{}

#[derive(Resource)]
pub struct StarSpawnTimer {
    pub timer: Timer,
}

impl Default for StarSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(STAR_SPAWN_INTERVAL, TimerMode::Repeating),
        }
    }
}