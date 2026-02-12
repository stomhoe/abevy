#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::time::time_types::Days;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, )]
pub struct RemainingDays(pub Days);