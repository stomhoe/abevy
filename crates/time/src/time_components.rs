#[allow(unused_imports)] use bevy::prelude::*;

use crate::time_types::Days;

#[derive(Component, Debug, Default, Clone)]
pub struct RemainingDays(pub Days);
