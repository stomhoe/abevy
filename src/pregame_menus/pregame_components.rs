use bevy::prelude::*;
use crate::{pregame_menus::PreGameState, AppState};




#[derive(Component)]
#[require(StateScoped::<PreGameState>)]
pub struct PreGameScoped {}