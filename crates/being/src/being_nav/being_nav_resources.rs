use bevy::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashMap;
use bevy::platform::collections::HashSet;
use bevy::tasks::Task;
use bevy_northstar::prelude::*;
use ::tilemap_shared::DimensionRef;
use ::tilemap_shared::GlobalTilePos;

use super::being_nav_chase_structs::ChaserNavPlan;
use super::being_nav_structs::{AiNavGridCache, SharedChaseFlowField};

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

impl AiNavGridRebuildInput {
    pub fn build_ai_nav_grid_cache(self) -> AiNavGridRebuildResult {
        use bevy_northstar::{CardinalGrid, grid::GridSettingsBuilder, nav::Nav};

        let mut grid = CardinalGrid::new(
            &GridSettingsBuilder::new_2d(self.width.max(3), self.height.max(3)).build(),
        );
        for blocked_tile in self.blocked_tiles.iter().copied() {
            grid.set_nav(
                UVec3::new(blocked_tile.x, blocked_tile.y, 0),
                Nav::Impassable,
            );
        }
        grid.build();

        AiNavGridRebuildResult {
            dim: self.dim,
            generation: self.generation,
            center: self.center,
            cache: AiNavGridCache {
                min: self.min_tile,
                grid,
                occupied: HashMap::default(),
            },
        }
    }
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

impl AiNavGrids {
    pub fn can_pathfind_between(
        &self,
        from_pos: GlobalTilePos,
        to_pos: GlobalTilePos,
        dim: DimensionRef,
    ) -> bool {
        let Some(cache) = self.by_dim.get(&dim) else {
            return false;
        };
        let Some((start, goal)) = cache.local_path_points(from_pos, to_pos) else {
            return false;
        };
        let mut req = PathfindArgs::new(start, goal).astar();
        let Some(path) = cache.grid.pathfind(&mut req) else {
            return false;
        };
        !path.is_partial()
    }
}

#[derive(Resource, Default)]
pub struct ChaserNavPlans {
    pub by_ent: EntityHashMap<ChaserNavPlan>,
}
