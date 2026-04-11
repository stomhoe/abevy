use bevy::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::entity::EntityHashSet;
use bevy::tasks::Task;

use super::being_nav_structs::{AiNavGridCache, ChaserNavPlan, SharedChaseFlowField};
use ::tilemap_shared::GlobalTilePos;

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

#[derive(Debug, Clone)]
pub struct AiNavGridRebuildInput {
    pub dim: Entity,
    pub min_tile: IVec2,
    pub width: u32,
    pub height: u32,
    pub center: IVec2,
    pub blocked_tiles: Vec<UVec2>,
    pub occupied: Vec<(Entity, GlobalTilePos)>,
}

pub struct AiNavGridRebuildResult {
    pub dim: Entity,
    pub center: IVec2,
    pub cache: AiNavGridCache,
}

#[derive(Resource, Debug, Default)]
pub struct AiNavGridRebuildTasks {
    pub tasks: Vec<Task<Vec<AiNavGridRebuildResult>>>,
    pub pending_dims: EntityHashSet,
}

#[derive(Clone)]
pub struct SharedChaseFlowFieldRebuildInput {
    pub target_ent: Entity,
    pub target_dim: Entity,
    pub target_pos: GlobalTilePos,
    pub min: IVec2,
    pub width: u32,
    pub height: u32,
    pub blocked_tiles: Vec<UVec2>,
    pub goal_tiles: Vec<GlobalTilePos>,
    pub slot_tiles: Vec<GlobalTilePos>,
    pub seed_goal_tiles: Vec<(GlobalTilePos, u32)>,
}

pub struct SharedChaseFlowFieldRebuildResult {
    pub target_ent: Entity,
    pub flow_field: Option<SharedChaseFlowField>,
}

#[derive(Resource, Debug, Default)]
pub struct SharedChaseFlowFieldRebuildTasks {
    pub tasks: Vec<Task<SharedChaseFlowFieldRebuildResult>>,
    pub pending_targets: EntityHashSet,
}

#[derive(Resource, Default)]
pub struct SharedChaseFlowFields {
    pub by_target: EntityHashMap<SharedChaseFlowField>,
}

#[derive(Resource, Default)]
pub struct ChaserNavPlans {
    pub by_ent: EntityHashMap<ChaserNavPlan>,
}
