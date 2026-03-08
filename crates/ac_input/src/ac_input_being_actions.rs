use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
pub struct BeingInputContext;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct BeingMoveAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct BeingMeleeAttackAction;
