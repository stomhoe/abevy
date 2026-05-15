
use bevy::{prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Event, Deserialize, Serialize, Clone)]
pub struct InitDone;

#[derive(Event, Deserialize, Serialize, Clone)]
pub struct SpawnSettingsEntity;