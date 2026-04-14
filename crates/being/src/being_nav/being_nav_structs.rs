use bevy::prelude::*;
use bevy_northstar::CardinalGrid;
use bevy_northstar::prelude::*;
use ::tilemap_shared::{DimensionRef, GlobalTilePos};
use bevy::platform::collections::HashMap;
use std::{cmp::Reverse, collections::{BinaryHeap, }, time::Duration};

pub struct AiNavGridCache {
    pub min: IVec2,
    pub grid: CardinalGrid,
    pub occupied: HashMap<UVec3, Entity>,
}

impl AiNavGridCache {
    pub fn local_from_gpos(
        &self,
        pos: GlobalTilePos,
    ) -> Option<UVec3> {
        let local = pos.0 - self.min;
        if local.x < 0 || local.y < 0 {
            return None;
        }
        let local = UVec3::new(local.x as u32, local.y as u32, 0);
        if local.x >= self.grid.width() || local.y >= self.grid.height() {
            return None;
        }
        Some(local)
    }

    pub fn gpos_from_local(
        &self,
        local: UVec3,
    ) -> GlobalTilePos {
        GlobalTilePos(local.xy().as_ivec2() + self.min)
    }

    pub fn local_path_points(
        &self,
        chaser_pos: GlobalTilePos,
        target_pos: GlobalTilePos,
    ) -> Option<(UVec3, UVec3)> {
        let start = self.local_from_gpos(chaser_pos)?;
        let goal = self.local_from_gpos(target_pos)?;
        Some((start, goal))
    }
}

pub struct SharedChaseFlowField {
    pub dim: DimensionRef,
    pub target_pos: GlobalTilePos,
    pub goal_tiles: Vec<GlobalTilePos>,
    pub slot_tiles: Vec<GlobalTilePos>,
    pub seed_goal_tiles: Vec<(GlobalTilePos, u32)>,
    pub min: IVec2,
    pub width: u32,
    pub height: u32,
    pub distances: Vec<u32>,
}

impl SharedChaseFlowField {
    pub fn matches_grid(
        &self,
        cache: &AiNavGridCache,
        dim: DimensionRef,
        target_pos: GlobalTilePos,
    ) -> bool {
        self.dim == dim
            && self.target_pos == target_pos
            && self.min == cache.min
            && self.width == cache.grid.width()
            && self.height == cache.grid.height()
    }

    pub fn build(
        cache: &AiNavGridCache,
        dim: DimensionRef,
        target_pos: GlobalTilePos,
        goal_tiles: &[GlobalTilePos],
        slot_tiles: &[GlobalTilePos],
        seed_goal_tiles: &[(GlobalTilePos, u32)],
    ) -> Option<Self> {
        let tile_count = cache.grid.width() as usize * cache.grid.height() as usize;
        let mut distances = vec![u32::MAX; tile_count];
        let mut frontier = BinaryHeap::new();
        let mut valid_goal_tiles = Vec::with_capacity(goal_tiles.len());
        let mut valid_slot_tiles = Vec::with_capacity(slot_tiles.len());
        let mut valid_seed_goal_tiles = Vec::with_capacity(seed_goal_tiles.len());

        for goal_tile in goal_tiles.iter().copied() {
            let Some(local) = cache.local_from_gpos(goal_tile) else {
                continue;
            };
            if !cache.grid.is_passable(local) {
                continue;
            }
            valid_goal_tiles.push(goal_tile);
        }
        for slot_tile in slot_tiles.iter().copied() {
            let Some(local) = cache.local_from_gpos(slot_tile) else {
                continue;
            };
            if !cache.grid.is_passable(local) {
                continue;
            }
            valid_slot_tiles.push(slot_tile);
        }

        for (goal_tile, seed_cost, ) in seed_goal_tiles.iter().copied() {
            let Some(local) = cache.local_from_gpos(goal_tile) else {
                continue;
            };
            if !cache.grid.is_passable(local) {
                continue;
            }
            let ix = Self::tile_index_for(cache.grid.width(), local);
            if distances[ix] <= seed_cost {
                continue;
            }
            distances[ix] = seed_cost;
            frontier.push(Reverse((seed_cost, ix, )));
            valid_seed_goal_tiles.push((goal_tile, seed_cost, ));
        }

        valid_goal_tiles.dedup();
        valid_slot_tiles.dedup();

        if valid_goal_tiles.is_empty() || valid_slot_tiles.is_empty() || valid_seed_goal_tiles.is_empty() {
            return None;
        }

        while let Some(Reverse((curr_dist, curr_ix, ))) = frontier.pop() {
            if distances[curr_ix] != curr_dist {
                continue;
            }
            let curr = Self::local_from_index(cache.grid.width(), curr_ix);
            let next_dist = curr_dist.saturating_add(1);

            for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let next_i = curr.xy().as_ivec2() + delta;
                if next_i.x < 0
                    || next_i.y < 0
                    || next_i.x >= cache.grid.width() as i32
                    || next_i.y >= cache.grid.height() as i32
                {
                    continue;
                }
                let next = UVec3::new(next_i.x as u32, next_i.y as u32, 0);
                if !cache.grid.is_passable(next) {
                    continue;
                }
                let next_ix = Self::tile_index_for(cache.grid.width(), next);
                if distances[next_ix] <= next_dist {
                    continue;
                }
                distances[next_ix] = next_dist;
                frontier.push(Reverse((next_dist, next_ix, )));
            }
        }

        Some(Self {
            dim,
            target_pos,
            goal_tiles: valid_goal_tiles,
            slot_tiles: valid_slot_tiles,
            seed_goal_tiles: valid_seed_goal_tiles,
            min: cache.min,
            width: cache.grid.width(),
            height: cache.grid.height(),
            distances,
        })
    }

    pub fn is_goal_tile(
        &self,
        pos: GlobalTilePos,
    ) -> bool {
        self.goal_tiles.contains(&pos)
    }

    pub fn is_slot_tile(
        &self,
        pos: GlobalTilePos,
    ) -> bool {
        self.slot_tiles.contains(&pos)
    }

    pub fn distance_at_gpos(
        &self,
        cache: &AiNavGridCache,
        pos: GlobalTilePos,
    ) -> Option<u32> {
        let local = cache.local_from_gpos(pos)?;
        let distance = self.distances.get(Self::tile_index_for(self.width, local)).copied()?;
        (distance != u32::MAX).then_some(distance)
    }

    pub fn reconstruct_path_tiles(
        &self,
        cache: &AiNavGridCache,
        start_pos: GlobalTilePos,
        out: &mut Vec<GlobalTilePos>,
    ) -> bool {
        out.clear();

        let Some(mut curr) = cache.local_from_gpos(start_pos) else {
            return false;
        };
        let Some(mut curr_dist) = self.distance_at_gpos(cache, start_pos) else {
            return false;
        };
        if curr_dist == 0 {
            return true;
        }

        out.reserve(curr_dist.min(128) as usize);
        while curr_dist > 0 {
            let mut best_next = None;
            let mut best_dist = curr_dist;
            let mut best_target_dist = i32::MAX;

            for delta in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let next_i = curr.xy().as_ivec2() + delta;
                if next_i.x < 0
                    || next_i.y < 0
                    || next_i.x >= self.width as i32
                    || next_i.y >= self.height as i32
                {
                    continue;
                }
                let next = UVec3::new(next_i.x as u32, next_i.y as u32, 0);
                let next_dist = self.distances[Self::tile_index_for(self.width, next)];
                if next_dist == u32::MAX || next_dist >= best_dist {
                    continue;
                }
                let next_target_dist = cache
                    .gpos_from_local(next)
                    .taxicab_tile_distance(self.target_pos) as i32;
                if next_dist < best_dist || next_target_dist < best_target_dist {
                    best_dist = next_dist;
                    best_target_dist = next_target_dist;
                    best_next = Some(next);
                }
            }

            let Some(next) = best_next else {
                out.clear();
                return false;
            };
            out.push(cache.gpos_from_local(next));
            curr = next;
            curr_dist = best_dist;
        }

        true
    }

    fn tile_index_for(
        width: u32,
        pos: UVec3,
    ) -> usize {
        (pos.y * width + pos.x) as usize
    }

    fn local_from_index(
        width: u32,
        ix: usize,
    ) -> UVec3 {
        let x = ix as u32 % width;
        let y = ix as u32 / width;
        UVec3::new(x, y, 0)
    }
}

impl crate::being_nav::being_nav_resources::AiNavGrids {
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

pub struct ChaserNavPlan {
    pub path_tiles: Vec<GlobalTilePos>,
    pub next_step_ix: usize,
    pub rebuild_timer: Timer,
    pub last_target_pos: Option<GlobalTilePos>,
    pub holds_at_partial_endpoint: bool,
    pub reserved_shared_goal: Option<GlobalTilePos>,
}

impl Default for ChaserNavPlan {
    fn default() -> Self {
        Self {
            path_tiles: Vec::new(),
            next_step_ix: 0,
            rebuild_timer: Timer::from_seconds(0.1, TimerMode::Once),
            last_target_pos: None,
            holds_at_partial_endpoint: false,
            reserved_shared_goal: None,
        }
    }
}

impl ChaserNavPlan {
    pub fn rebuild_interval(chaser_speed: f32, prey_speed: f32, distance: f32) -> Duration {
        let chaser_speed = chaser_speed.max(0.05);
        let speed_ratio = (prey_speed.max(0.0) / chaser_speed).clamp(0.35, 2.25);
        let distance_factor = (8.0 / distance.max(1.0)).clamp(0.45, 2.0);
        let urgency = (speed_ratio * distance_factor).clamp(0.35, 3.5);
        Duration::from_secs_f32((0.45 / urgency).clamp(0.08, 0.9))
    }

    pub fn reset(&mut self, interval: Duration) {
        self.path_tiles.clear();
        self.next_step_ix = 0;
        self.last_target_pos = None;
        self.holds_at_partial_endpoint = false;
        self.reserved_shared_goal = None;
        self.rebuild_timer = Timer::new(interval, TimerMode::Once);
    }

    pub fn clear_shared_goal(&mut self) {
        self.reserved_shared_goal = None;
    }

    pub fn clear_path_and_retry(&mut self, interval: Duration, target_pos: GlobalTilePos) {
        self.path_tiles.clear();
        self.next_step_ix = 0;
        self.last_target_pos = Some(target_pos);
        self.holds_at_partial_endpoint = false;
        self.reserved_shared_goal = None;
        self.rebuild_timer = Timer::new(interval, TimerMode::Once);
    }

    pub fn next_step(&mut self, chaser_pos: GlobalTilePos) -> Option<GlobalTilePos> {
        while self.next_step_ix < self.path_tiles.len() && self.path_tiles[self.next_step_ix] == chaser_pos {
            self.next_step_ix += 1;
        }
        self.path_tiles.get(self.next_step_ix).copied()
    }
}
