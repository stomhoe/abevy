use std::time::SystemTime;

#[allow(unused_imports)] use bevy::prelude::*;

use crate::common::common_components::DisplayName;






 #[derive(serde::Deserialize, Event, serde::Serialize, Clone)]
pub struct SendPlayerName (pub DisplayName);