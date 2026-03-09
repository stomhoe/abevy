#[allow(unused_imports, )]
use bevy::{ecs::entity::{EntityHashSet, MapEntities}, platform::collections::HashMap, prelude::*};
use bevy_replicon::prelude::{ Replicated};
use ::being_shared::*;

use modifier_shared::modifier_components::AppliedModifiers;
use movement::movement_components::*;

use common::common_components::*;
use sprite_animation_shared::MoveAnimActive;
use serde::{Deserialize, Serialize};
use ::tilemap_shared::*;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Default)]
#[require(MoveVecMag,
Replicated, MoveAnimActive,
Grounding, Visibility, CardinalDirection, AppliedModifiers,
Prefix::trunc("Being"), DimensionStrIdRef::overworld_fallback(), AssetScoped,
GridLockedMovement )]//don't add Transform so I can tell if it's missing instead of the being going to 0,0
pub struct Being;
impl Being {
    pub const Z_LEVEL: f32 = 1_000.;
}

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
