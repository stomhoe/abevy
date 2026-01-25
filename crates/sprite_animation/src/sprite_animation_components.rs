#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize};


#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy, Reflect)]
#[require(Prefix::trunc("Animations"), Replicated, AssetScoped, AppStateScoped, )]
pub struct AnimationsHolder;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Reflect)]
#[require(Prefix::trunc("Animation"), Replicated, AssetScoped, AppStateScoped, )]
pub struct AnimationComp;



