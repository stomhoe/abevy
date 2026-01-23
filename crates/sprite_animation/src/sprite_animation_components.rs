use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use bevy_spritesheet_animation::prelude::{Animation, AnimationProgress};
use common::common_components::{AssetScoped, Prefix, SessionScoped};
use serde::{Deserialize, Serialize};
use sprite::sprite_components::AnimType;


#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy, Reflect)]
#[require(Prefix::trunc("Animations"), Replicated, AssetScoped, SessionScoped, )]
pub struct AnimationsHolder;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Reflect)]
#[require(Prefix::trunc("Animation"), Replicated, AssetScoped, SessionScoped, )]
pub struct AnimationComp;



