#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::modifier_components::{ModifierCategories, ApplyMode};



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, )]
pub struct Speed;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierCategories, )]
pub struct InvertMovement;