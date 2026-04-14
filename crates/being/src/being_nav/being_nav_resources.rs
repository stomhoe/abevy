use bevy::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashMap;
use bevy::platform::collections::HashSet;
use bevy::tasks::Task;
use ::tilemap_shared::DimensionRef;

use super::being_nav_structs::{AiNavGridCache, ChaserNavPlan, SharedChaseFlowField};

#[derive(Resource)]
pub struct AiNavGrids {
    pub by_dim: HashMap<DimensionRef, AiNavGridCache>,
    pub center_by_dim: HashMap<DimensionRef, IVec2>,
    pub rebuild_timer: Timer,
}

impl Default for AiNavGrids {
    fn default() -> Self {
        Self {
            by_dim: HashMap::default(),
            center_by_dim: HashMap::default(),
            rebuild_timer: Timer::from_seconds(0.35, TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiNavGridRebuildInput {
    pub dim: DimensionRef,
    pub generation: u64,
    pub min_tile: IVec2,
    pub width: u32,
    pub height: u32,
    pub center: IVec2,
    pub blocked_tiles: Vec<UVec2>,
}

pub struct AiNavGridRebuildResult {
    pub dim: DimensionRef,
    pub generation: u64,
    pub center: IVec2,
    pub cache: AiNavGridCache,
}

#[derive(Resource, Debug, Default)]
pub struct AiNavGridRebuildTasks {
    pub tasks: Vec<Task<Vec<AiNavGridRebuildResult>>>,
    pub pending_dims: HashSet<DimensionRef>,
    pub pending_generation_by_dim: HashMap<DimensionRef, u64>,
    pub next_generation: u64,
}

#[derive(Resource, Default)]
pub struct SharedChaseFlowFields {
    pub by_target: EntityHashMap<SharedChaseFlowField>,
}

#[derive(Resource, Default)]
pub struct ChaserNavPlans {
    pub by_ent: EntityHashMap<ChaserNavPlan>,
}
