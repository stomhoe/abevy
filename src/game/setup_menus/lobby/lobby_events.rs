#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::game::game_components::DisplayName;

#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct HostStartedGame { }




// No olvidarse de agregarlo al Plugin del módulo
// 

// #[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
// pub struct ConnectedEvent;
