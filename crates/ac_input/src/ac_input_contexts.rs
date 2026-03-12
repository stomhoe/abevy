use bevy::prelude::*;

#[derive(Component, Default)]
#[require(Name::new("InputContextsHolder"))]
pub struct InputContextsHolder;

#[derive(Component)]
pub struct BeingDirectControlInputContext;

#[derive(Component)]
pub struct DebugInputContext;
