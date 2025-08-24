#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::modifier_components::{ModifierCategories, OperationType};


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories)]
pub struct MovementModifier;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, MovementModifier,)]
pub struct Speed;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, MovementModifier)]
pub struct InvertMovement;