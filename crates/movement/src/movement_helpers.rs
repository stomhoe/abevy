use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use sprite_animation_shared::{BeingChangedMoveState, MoveAnimActive};
use tilemap_shared::GlobalTilePos;

pub fn normalize_to_axis_dir(input: Vec2) -> IVec2 {
    if input == Vec2::ZERO {
        IVec2::ZERO
    } else if input.x.abs() >= input.y.abs() {
        IVec2::new(input.x.signum() as i32, 0)
    } else {
        IVec2::new(0, input.y.signum() as i32)
    }
}

pub fn ticks_per_tile(speed: f32, delta: f32, dir: IVec2) -> u16 {
    if speed <= 0.0 || delta <= 0.0 || dir == IVec2::ZERO {
        return 0;
    }
    let tile_size = GlobalTilePos::TILE_SIZE_PXS.as_vec2();
    let distance = if dir.x != 0 { tile_size.x } else { tile_size.y }.max(1.0);
    ((distance / (speed * delta)).ceil() as u16).max(1)
}

pub fn move_anim_changed(
    being_ent: Entity,
    move_anim: &mut MoveAnimActive,
    active: bool,
    messages: &mut HashSet<BeingChangedMoveState>,
) {
    move_anim.set(active, being_ent, messages);
}
