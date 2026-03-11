use bevy::prelude::*;
use param_sets::BlockingTileParamSet;
use serde::{Deserialize, Serialize};
use tilemap_shared::{DimensionRef, GlobalTilePos};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PendingMoveIntent {
    pub dir: MoveDir,
    pub ticks_since_prev_intent: u32,
}
impl PendingMoveIntent {
    pub fn new(dir: IVec2, prev_tick: u32, curr_tick: u32) -> Self {
        Self {
            dir: MoveDir::from_ivec2(dir),
            ticks_since_prev_intent: curr_tick - prev_tick,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum MoveDir {
    North,
    South,
    East,
    West,
    Stop,
}

impl Default for MoveDir {
    fn default() -> Self {
        Self::Stop
    }
}

impl MoveDir {
    pub fn from_ivec2(dir: IVec2) -> Self {
        let dir = dir.clamp(IVec2::NEG_ONE, IVec2::ONE);
        match dir {
            IVec2::X => Self::East,
            IVec2::NEG_X => Self::West,
            IVec2::Y => Self::North,
            IVec2::NEG_Y => Self::South,
            _ => Self::Stop,
        }
    }

    pub fn as_vec2(self) -> Vec2 {
        match self {
            Self::North => Vec2::Y,
            Self::South => Vec2::NEG_Y,
            Self::East => Vec2::X,
            Self::West => Vec2::NEG_X,
            Self::Stop => Vec2::ZERO,
        }
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct PendingMoveIntents(pub Vec<PendingMoveIntent>);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ServerMoveReplayState {
    pub active_dir: MoveDir,
    pub ticks_until_next_intent: u32,
}


#[derive(Component, Debug, Default, Clone)]
pub struct MoveVecMag {
    pub norm_move_dir: Vec2,
    pub speed_magnitude: f32,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct GridLockedMovement {
    pub visual_origin_tile: IVec2,
    pub step_dir: IVec2,
    pub progress_ticks: u16,
    pub step_ticks_total: u16,
    pub move_cooldown_secs_left: f32,
}

impl GridLockedMovement {
    pub fn is_stepping(&self) -> bool {
        self.step_dir != IVec2::ZERO && self.progress_ticks < self.step_ticks_total
    }

    pub fn ensure_grid_anchor(&mut self, tile_pos: GlobalTilePos) {
        if self.visual_origin_tile == IVec2::ZERO && self.step_dir == IVec2::ZERO {
            self.visual_origin_tile = tile_pos.0;
        }
    }

    pub fn clear_step(&mut self, tile_pos: GlobalTilePos) {
        self.visual_origin_tile = tile_pos.0;
        self.step_dir = IVec2::ZERO;
        self.progress_ticks = 0;
        self.step_ticks_total = 0;
    }

    pub fn start_step(
        &mut self,
        tile_pos: &mut GlobalTilePos,
        dir: IVec2,
        step_ticks_total: u16,
        move_duration_secs: f32,
    ) {
        self.visual_origin_tile = tile_pos.0;
        tile_pos.0 += dir;
        self.step_dir = dir;
        self.progress_ticks = 0;
        self.step_ticks_total = step_ticks_total.max(1);
        self.move_cooldown_secs_left = move_duration_secs.max(0.0);
    }

    pub fn try_start_step(
        &mut self,
        blocking_tiles: &BlockingTileParamSet,
        to_drain: &mut Vec<Entity>,
        dim_ref: DimensionRef,
        being_ent: Entity,
        tile_pos: &mut GlobalTilePos,
        dir: IVec2,
        move_duration_secs: f32,
        step_ticks_total: u16,
    ) -> bool {
        if dir == IVec2::ZERO || self.move_cooldown_secs_left > 0.0 || step_ticks_total == 0 {
            return false;
        }
        let next_tile = GlobalTilePos(tile_pos.0 + dir);
        if blocking_tiles.is_blocked_at(to_drain, dim_ref, next_tile, being_ent) {
            return false;
        }
        self.start_step(tile_pos, dir, step_ticks_total, move_duration_secs);
        true
    }

    pub fn progress_grid_step(&mut self, tile_pos: GlobalTilePos, delta_secs: f32) {
        self.move_cooldown_secs_left = (self.move_cooldown_secs_left - delta_secs).max(0.0);
        if !self.is_stepping() {
            self.clear_step(tile_pos);
            return;
        }
        self.progress_ticks = self.progress_ticks.saturating_add(1);
        if self.progress_ticks >= self.step_ticks_total {
            self.clear_step(tile_pos);
        }
    }

    pub fn grid_translation(&self, tile_pos: GlobalTilePos, z: f32) -> Vec3 {
        let origin = GlobalTilePos(self.visual_origin_tile).to_translation(z);
        if !self.is_stepping() || self.step_ticks_total == 0 {
            return tile_pos.to_translation(z);
        }
        let t = (self.progress_ticks as f32 / self.step_ticks_total as f32).clamp(0.0, 1.0);
        origin.lerp(tile_pos.to_translation(z), t)
    }
}
