use bevy::prelude::*;
use tilemap_shared::GlobalTilePos;

pub const MAX_GRID_STEPS_PER_FIXED_TICK: u16 = 256;

pub fn tiles_per_tick(speed: f32, delta: f32, dir: IVec2) -> f32 {
    if speed <= 0.0 || delta <= 0.0 || dir == IVec2::ZERO {
        return 0.0;
    }
    let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
    let distance = if dir.x != 0 { tile_size.x } else { tile_size.y }.max(1.0);
    (speed * delta) / distance
}

pub fn ticks_per_tile(speed: f32, delta: f32, dir: IVec2) -> u16 {
    if speed <= 0.0 || delta <= 0.0 || dir == IVec2::ZERO {
        return 0;
    }
    let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
    let distance = if dir.x != 0 { tile_size.x } else { tile_size.y }.max(1.0);
    ((distance / (speed * delta)).ceil() as u16).max(1)
}

pub fn secs_per_tile(speed: f32, delta: f32, dir: IVec2) -> f32 {
    ticks_per_tile(speed, delta, dir) as f32 * delta
}
