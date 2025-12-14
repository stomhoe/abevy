use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use bevy_spritesheet_animation::prelude::{Animation, AnimationProgress};
use common::common_components::{EntityPrefix};
use serde::{Deserialize, Serialize};
use sprite::sprite_components::AnimType;


#[derive(Component, Debug, Default, Serialize, Deserialize, Clone, Copy, Reflect)]
#[require(EntityPrefix::new_truncated("Animations"), Replicated, )]
pub struct AnimationsHolder;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Reflect)]
#[require(EntityPrefix::new_truncated("Animation"), Replicated, )]
pub struct AnimationMain;



#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct AnimationProgresses(
    pub HashMap<Handle<Animation>, AnimationProgress>,
);