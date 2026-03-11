use ::being_shared::*;
#[allow(unused_imports)]
use bevy::{
    ecs::entity::{EntityHashSet, MapEntities},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_replicon::prelude::Replicated;

use modifier_shared::modifier_components::AppliedModifiers;
use movement::movement_components::*;

use ::tilemap_shared::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};
use sprite_animation_shared::MoveAnimActive;

pub use ::being_shared::Being;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct ToChase {
    pub target: Entity,
    pub stop_distance: f32,
}
impl ToChase {
    pub fn new(target: Entity, stop_distance: f32) -> Self {
        Self {
            target,
            stop_distance: stop_distance.max(0.0),
        }
    }
}

pub const COLLISION_MASK_HASHID: HashId = HashId::hash("collision_mask");
pub const HITBOX_HASHID: HashId = HashId::hash("hitbox");

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct HitboxReceiver(pub HashId);
impl Default for HitboxReceiver {
    fn default() -> Self {
        Self(COLLISION_MASK_HASHID)
    }
}
