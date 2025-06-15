use bevy::prelude::*;
use crate::{AppState, StateScopedCamera};


pub fn setup(mut commands: Commands) {
    commands.spawn(StateScopedCamera);
}
