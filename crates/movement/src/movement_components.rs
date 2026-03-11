use bevy::prelude::*;
use param_sets::BlockingTileParamSet;
use serde::{Deserialize, Serialize};
use tilemap_shared::{DimensionRef, GlobalTilePos};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PendingMoveIntent {
    pub dir: IVec2,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct InputMoveDir(pub Vec2);


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
    ) -> bool {
        if dir == IVec2::ZERO || self.is_stepping() || step_ticks_total == 0 {
            return false;
        }
        let next_tile = GlobalTilePos(tile_pos.0 + dir);
        if blocking_tiles.is_blocked_at(to_drain, dim_ref, next_tile, being_ent) {
            return false;
        }
        self.start_step(tile_pos, dir, step_ticks_total);
        true
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
