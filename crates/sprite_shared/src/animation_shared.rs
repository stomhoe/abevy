use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_types::FixedStr;
use serde::{Deserialize, Serialize};

pub const DOWN: &str = "down";
pub const LEFT: &str = "left";
pub const UP: &str = "up";
pub const RIGHT: &str = "right";
pub const IDLE: &str = "idle";
pub const WALK: &str = "walk";
pub const SWIM : &str = "swim";
pub const FLY: &str = "fly";


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct AnimationIdPrefix(pub FixedStr<32>);

impl<T: AsRef<str>> From<T> for AnimationIdPrefix {
    fn from(s: T) -> Self {
        AnimationIdPrefix(FixedStr::from(s.as_ref()))
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct MoveAnimActive(pub bool);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
//NO VA REPLICATED, SE HACE LOCALMENTE EN CADA PC SEGÚN LOS INPUTS RECIBIDOS DE OTROS PLAYERS
pub struct AnimationState(pub String);
impl AnimationState {
    pub fn new_idle() -> Self { Self(IDLE.into()) }
    pub fn set_idle(&mut self) { self.0 = IDLE.into(); }
    pub fn set_walk(&mut self) { self.0 = WALK.into(); }
    pub fn set_swim(&mut self) { self.0 = SWIM.into(); }
    pub fn set_fly(&mut self) { self.0 = FLY.into(); }

}impl std::fmt::Display for AnimationState {
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AnimationSystems;