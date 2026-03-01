use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use bevy_spritesheet_animation::prelude::{Animation, AnimationProgress, Spritesheet};
use common::{common_components::*, };
use serde::{Deserialize, Serialize};
use crate::sprite_animation_messages::BeingChangedMoveState;




#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Reflect, )]
pub struct PlayingSpeed(pub f32);

impl Default for PlayingSpeed {
    fn default() -> Self {
        PlayingSpeed(1.0)
    }
}

#[derive(Component, Debug, Default, Clone)]
//va en cada sprite, no en las entities de las animations porque estas son compartidas por multiples sprites
pub struct AcAnimationProgresses(
    pub HashMap<Handle<Animation>, AnimationProgress>,
);

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Default)]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
//not replicated
pub struct AnimExtraState(pub HashId);
impl AnimExtraState {
    pub fn new(id: impl Into<HashId>) -> Self {
        AnimExtraState(id.into())
    }
}

impl std::fmt::Display for AnimExtraState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Component, Debug, Default, Clone, )]
pub struct AnimationHandle(pub Handle<Animation>,);

#[derive(Component, Debug, Clone, )]
pub struct AnimationSheet(pub Spritesheet,);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct ClipStartFrames(pub Vec<usize>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct AlternatingStartFramesConfig(pub Vec<Option<(usize, usize)>>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct AlternatingStartFramesState(pub Vec<usize>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct SaveAnimationProgress;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
#[require(HotReload, AssetScoped, Replicated, Prefix::trunc("Animation"),   )]
pub struct AcAnimation;



common::define_entity_map_systems!(
    AcAnimation
);
