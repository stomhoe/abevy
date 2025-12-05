#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use common::common_components::EntityPrefix;
use serde::{Deserialize, Serialize};




#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Reflect)]
#[require(EntityPrefix::new_truncated("Animation"), Replicated, /*SessionScoped*/ )]
pub struct AnimationMain;


// #[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
// pub struct AnimationProgresses(
//     HashMap<Anim
// );