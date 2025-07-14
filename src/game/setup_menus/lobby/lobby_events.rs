#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::common::common_components::DisplayName;

#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct HostStartedGame { }


#[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct SendPlayerName (pub DisplayName);

// No olvidarse de agregarlo al Plugin del módulo
// 