use bevy::prelude::*;
use serde::{Deserialize, Serialize};
pub use common::common_components::{TemplHashIdRef, TemplEntiRef};

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct Templ;
