#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::time::time_types::Days;

#[derive(Component, Debug, Default, )]
pub struct RemainingDays(pub Days);