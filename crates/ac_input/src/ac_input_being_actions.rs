use bevy::prelude::Vec2;
use bevy_enhanced_input::prelude::*;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct BeingWasdAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct BeingMeleeAttackAction;
