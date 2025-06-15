use bevy::prelude::*;
use crate::{pregame_menus::pregame_components::{PreGameScoped}, AppState, StateScopedCamera};


pub fn setup(mut commands: Commands) {
    commands.spawn(StateScopedCamera);
}
