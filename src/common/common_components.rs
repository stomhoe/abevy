use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayName(pub String);

impl DisplayName {
    pub fn new(name: impl Into<String>) -> Self {
        DisplayName(name.into())
    }
}

#[derive(Component, Debug,)]
pub struct Description(String);

#[derive(Component, Debug,)]
pub struct Sid(String);



#[derive(Component, Debug,)]
pub struct GameZindex(pub f32);