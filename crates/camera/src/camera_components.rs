use bevy::prelude::*;


#[derive(Component, )] 
#[require(Transform)]
/// timer gets reset when camera moves
pub struct CameraTarget { pub updateChunksAround: Timer }

impl Default for CameraTarget {
    fn default() -> Self {
        CameraTarget { updateChunksAround: Timer::from_seconds(0.2, TimerMode::Repeating) }
    }
}
