use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::{Animation, Spritesheet};
use common::{common_components::StrId, common_types::*};
use serde::{Deserialize, Serialize};


#[allow(unused_imports)] use {bevy::prelude::*, };

pub fn plugin(app: &mut App) {
    
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, )]
pub struct PlayingSpeed(pub f32);
impl PlayingSpeed {
    pub fn new(speed: f32) -> Self {
        Self(speed)
    }
}
impl Default for PlayingSpeed {
    fn default() -> Self {
        PlayingSpeed(1.0)
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, Hash, PartialEq, Eq, Default)]
pub struct MoveAnimActive(pub bool);
impl From<&str> for MoveAnimActive {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "move" | "walk"=> MoveAnimActive(true),
            _ => MoveAnimActive(false),
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, PartialEq, Eq, Hash)]
//NO VA REPLICATED, SE HACE LOCALMENTE EN CADA PC SEGÚN LOS INPUTS RECIBIDOS DE OTROS PLAYERS
pub struct AnimationState(pub StrId);
impl AnimationState {
    
    pub fn new<S: AsRef<str>>(state: S) -> Self {
        Self(StrId::new_truncated(state.as_ref()))
    }
}
impl std::fmt::Display for AnimationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Component, Debug, Default, Clone, Reflect)]
pub struct AnimationHandle(pub Handle<Animation>,);

#[derive(Component, Debug, Clone, )]
pub struct AnimationSheet(pub Spritesheet,);


#[derive(Resource, Debug, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct AnimationLibrary ( pub HashMap<StrId, Entity>, );