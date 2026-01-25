#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy, Reflect)]
#[require(SparedFromHotReloading, AssetScoped, Replicated,Prefix::trunc("Animations"),   )]
pub struct AnimationsHolder;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Reflect)]
#[require(SparedFromHotReloading, AssetScoped, Replicated,Prefix::trunc("Animation"),   )]
pub struct AnimationComp;



