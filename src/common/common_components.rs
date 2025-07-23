use bevy::prelude::*;
#[allow(unused_imports)] 
use serde::{Deserialize, Serialize};






#[derive(Component, Debug,)]
pub struct GameZindex(pub i32);