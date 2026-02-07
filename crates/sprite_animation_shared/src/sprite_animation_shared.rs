use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use bevy_spritesheet_animation::prelude::{Animation, AnimationProgress, Spritesheet};
use common::{common_components::{AssetScoped, Prefix, SparedFromHotReloading, StrId}, common_types::*};
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

#[derive(Component, Debug, Default, Clone, Reflect)]
//va en cada sprite, no en las entities de las animations porque estas son compartidas por multiples sprites
pub struct AcAnimationProgresses(
    pub HashMap<Handle<Animation>, AnimationProgress>,
);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, Hash, PartialEq, Eq, Default)]
pub struct MoveAnimActive(bool);
impl MoveAnimActive {
    pub fn set(&mut self, state: bool, being_ent: Entity, hash_set: &mut HashSet<BeingChangedMoveState>) {
        if self.0 != state {
            self.0 = state;
            hash_set.insert(BeingChangedMoveState(being_ent));
        }
    }
    pub fn get(&self) -> bool {
        self.0
    }
}

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
        Self(StrId::trunc(state.as_ref()))
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Reflect)]
#[require(SparedFromHotReloading, AssetScoped, Replicated, Prefix::trunc("Animation"),   )]
pub struct AnimationComp;



common::define_entity_map_systems!(
    AnimationComp
);


#[derive(Message, Clone, PartialEq, Eq, Hash)]
pub struct BeingChangedMoveState(pub Entity);
