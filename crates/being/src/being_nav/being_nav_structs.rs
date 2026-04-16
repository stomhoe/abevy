use bevy::prelude::*;
use bevy_northstar::CardinalGrid;
use ::tilemap_shared::{DimensionRef, GlobalTilePos};
use bevy::platform::collections::HashMap;
use std::{cmp::Reverse, collections::BinaryHeap};

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


