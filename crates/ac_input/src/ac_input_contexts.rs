use bevy::prelude::*;

#[derive(Component, Default)]
#[require(Name::new("InputContextsHolder"))]
pub struct InputContextsHolder;

#[derive(Component, Default, Clone)]
pub struct BeingDirectControlInputContext;

#[derive(Component, Default, Clone)]
pub struct DebugInputContext;
