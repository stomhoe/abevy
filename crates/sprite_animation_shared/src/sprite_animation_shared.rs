use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::{Animation, Spritesheet};
use common::{common_components::StrId, common_types::*};
use serde::{Deserialize, Serialize};





#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct MoveAnimActive(pub bool);

pub const DOWN: &str = "down";
pub const LEFT: &str = "left";
pub const UP: &str = "up";
pub const RIGHT: &str = "right";
pub const IDLE: &str = "idle";
pub const WALK: &str = "walk";
pub const SWIM : &str = "swim";
pub const FLY: &str = "fly";


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect, PartialEq, Eq, Hash)]
//NO VA REPLICATED, SE HACE LOCALMENTE EN CADA PC SEGÚN LOS INPUTS RECIBIDOS DE OTROS PLAYERS
pub struct AnimationState(pub StrId);
impl AnimationState {
    // pub fn new_idle() -> Self { Self(IDLE.into()) }
    // pub fn set_idle(&mut self) { self.0 = IDLE.into(); }
    // pub fn set_walk(&mut self) { self.0 = WALK.into(); }
    // pub fn set_swim(&mut self) { self.0 = SWIM.into(); }
    // pub fn set_fly(&mut self) { self.0 = FLY.into(); }
    pub fn new<S: AsRef<str>>(state: S) -> Self {
        Self(StrId::new_truncated(state.as_ref()))
    }
}impl std::fmt::Display for AnimationState {
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Resource, Debug, Default, )]
/// Global Animation Library
pub struct AnimationLibrary (
    pub HashMap<StrId, (Spritesheet, Handle<Animation>)>,
);

