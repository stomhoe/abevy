use bevy::prelude::*;

#[derive(Component)]
#[require(Transform)]
/// timer gets reset when camera moves
pub struct CameraTarget {
    pub update_chunks_around: Timer,
}

impl Default for CameraTarget {
    fn default() -> Self {
        CameraTarget {
            update_chunks_around: Timer::from_seconds(0.2, TimerMode::Repeating),
        }
    }
}
