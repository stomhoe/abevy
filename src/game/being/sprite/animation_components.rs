
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game::{being::sprite::{
    animation_constants::*, animation_resources::*
}};

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
