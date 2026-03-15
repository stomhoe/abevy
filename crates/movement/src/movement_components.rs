use bevy::prelude::*;
use param_sets::BlockingTileParamSet;
use serde::{Deserialize, Serialize};
use tilemap_shared::{DimensionRef, GlobalTilePos};

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct InputMoveDir(pub Vec2);

impl InputMoveDir {
    pub fn normalize_to_axis_dir(self) -> IVec2 {
        if self.0 == Vec2::ZERO {
            IVec2::ZERO
        } else if self.0.x.abs() >= self.0.y.abs() {
            IVec2::new(self.0.x.signum() as i32, 0)
        } else {
            IVec2::new(0, self.0.y.signum() as i32)
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PendingTileCorrection {
    pub gpos: GlobalTilePos,
    pub secs_left: f32,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct NormMoveDir(pub Vec2);

#[derive(Component, Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, )]
pub struct SpeedMagnitude(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryStartStepOutcome {
    Successful,
    IVec2ZeroDir,
    AlreadyStepping,
    ZeroStepTicks,
    Blocked,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct GridLockedMovement {
    pub visual_origin_tile: IVec2,
    pub step_dir: IVec2,
    pub progress_ticks: u16,
    pub step_ticks_total: u16,
    pub moved_this_tick: bool,
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
    ) {
        self.visual_origin_tile = tile_pos.0;
        tile_pos.0 += dir;
        self.step_dir = dir;
        self.progress_ticks = 0;
        self.step_ticks_total = step_ticks_total.max(1);
        self.moved_this_tick = true;
    }

    pub fn try_start_step(
        &mut self,
        blocking_tiles: &BlockingTileParamSet,
        to_drain: &mut Vec<Entity>,
        dim_ref: DimensionRef,
        being_ent: Entity,
        tile_pos: &mut GlobalTilePos,
        dir: IVec2,
        step_ticks_total: u16,
    ) -> TryStartStepOutcome {
        if dir == IVec2::ZERO || self.is_stepping() || step_ticks_total == 0 {
            return if dir == IVec2::ZERO {
                TryStartStepOutcome::IVec2ZeroDir
            } else if self.is_stepping() {
                TryStartStepOutcome::AlreadyStepping
            } else {
                TryStartStepOutcome::ZeroStepTicks
            };
        }
        let next_tile = GlobalTilePos(tile_pos.0 + dir);
        if blocking_tiles.is_blocked_at(to_drain, dim_ref, next_tile, being_ent) {
            return TryStartStepOutcome::Blocked;
        }
        self.start_step(tile_pos, dir, step_ticks_total);
        TryStartStepOutcome::Successful
    }

    pub fn advance_steps_immediate(
        &mut self,
        blocking_tiles: &BlockingTileParamSet,
        to_drain: &mut Vec<Entity>,
        dim_ref: DimensionRef,
        being_ent: Entity,
        tile_pos: &mut GlobalTilePos,
        dir: IVec2,
        steps: u16,
    ) -> u16 {
        if dir == IVec2::ZERO || steps == 0 || self.is_stepping() {
            return 0;
        }
        let mut steps_taken = 0;
        for _ in 0..steps {
            let next_tile = GlobalTilePos(tile_pos.0 + dir);
            if blocking_tiles.is_blocked_at(to_drain, dim_ref, next_tile, being_ent) {
                break;
            }
            tile_pos.0 += dir;
            steps_taken += 1;
        }
        if steps_taken > 0 {
            self.clear_step(*tile_pos);
            self.moved_this_tick = true;
        }
        steps_taken
    }

    pub fn consume_recent_motion(&mut self) -> bool {
        let moved_this_tick = self.moved_this_tick;
        self.moved_this_tick = false;
        moved_this_tick
    }

    pub fn progress_grid_step(&mut self, tile_pos: GlobalTilePos) {
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
