use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use ::being_shared::*;
use super::being_nav_structs::{AiNavGridCache, ChaserNavPlan};

#[derive(Resource)]
pub struct AiNavGrids {
    pub by_dim: EntityHashMap<AiNavGridCache>,
    pub center_by_dim: EntityHashMap<IVec2>,
    pub rebuild_timer: Timer,
}

impl Default for AiNavGrids {
    fn default() -> Self {
        Self {
            by_dim: EntityHashMap::default(),
            center_by_dim: EntityHashMap::default(),
            rebuild_timer: Timer::from_seconds(0.35, TimerMode::Repeating),
        }
    }
}

#[derive(Resource, Default)]
pub struct ChaserNavPlans {
    pub by_ent: EntityHashMap<ChaserNavPlan>,
}
