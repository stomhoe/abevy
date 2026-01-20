#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::modifier_components::{ModifierTags, ApplyMode};



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct Speed;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(ModifierTags, )]
pub struct InvertMovement;