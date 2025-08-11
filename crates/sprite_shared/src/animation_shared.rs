use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

pub const DOWN: &str = "_down";
pub const LEFT: &str = "_left";
pub const UP: &str = "_up";
pub const RIGHT: &str = "_right";
pub const IDLE: &str = "_idle";
pub const WALK: &str = "_walk";
pub const SWIM : &str = "_swim";
pub const FLY: &str = "_fly";



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