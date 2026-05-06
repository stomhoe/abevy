
use bevy::prelude::*;

pub use game_common::CameraTarget;

#[derive(Resource, Clone, Copy)]
pub struct DaylightDirectionalLightEntity(pub Entity);
