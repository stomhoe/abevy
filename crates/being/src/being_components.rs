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
pub struct Chaser {
    pub target: Entity,
    pub stop_distance: f32,
}
impl Chaser {
    pub fn new(target: Entity, stop_distance: f32) -> Self {
        Self {
            target,
            stop_distance: stop_distance.max(0.0),
        }
    }

    pub fn chase_target_pos(
        &self,
        chaser_ent: Entity,
        chaser_dim: ::tilemap_shared::DimensionRef,
        targets_query: &Query<(Entity, &GlobalTilePos, &::tilemap_shared::DimensionRef)>,
    ) -> Option<GlobalTilePos> {
        if self.target == chaser_ent {
            return None;
        }
        let Ok((_target_ent, target_gpos, &target_dim)) = targets_query.get(self.target) else {
            return None;
        };
        if target_dim != chaser_dim {
            return None;
        }
        Some(*target_gpos)
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

