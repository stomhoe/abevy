#[allow(unused_imports)]
use bevy::{
    ecs::entity::{EntityHashSet, MapEntities},
    platform::collections::HashMap,
    prelude::*,
};


use ::tilemap_shared::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};

pub use ::being_shared::{Being, FactionLeader};

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct Chasing {
    pub target: Entity,
    pub stop_distance: f32,
}
impl Chasing {
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
