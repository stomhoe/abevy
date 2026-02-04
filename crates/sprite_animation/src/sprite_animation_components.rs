#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy, Reflect)]
#[require(SparedFromHotReloading, AssetScoped, Replicated,Prefix::trunc("Animations"),   )]
pub struct AnimationsHolder;



