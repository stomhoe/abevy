use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use being_shared::movement_shared_components::{
    GridLockedMovement,
    GridLockedMovementVisual,
    TryStartStepOutcome,
};
use param_sets::BlockingTileParamSet;
use tilemap_shared::{DimensionRef, GlobalTilePos};

pub fn find_nearest_overlap_resolution_gpos(
    blocking_tiles: &mut BlockingTileParamSet,
    reserved_positions: &HashSet<(DimensionRef, GlobalTilePos)>,
    dim_ref: DimensionRef,
    anchor: GlobalTilePos,
    being_ent: Entity,
) -> Option<GlobalTilePos> {
    if !reserved_positions.contains(&(dim_ref, anchor))
        && !blocking_tiles.is_blocked_at_tiles_only(dim_ref, anchor, being_ent)
    {
        return Some(anchor);
    }

    const MAX_SEARCH_RADIUS: i32 = 256;
    for radius in 1..=MAX_SEARCH_RADIUS {
        let min_x = anchor.0.x - radius;
        let max_x = anchor.0.x + radius;
        let min_y = anchor.0.y - radius;
        let max_y = anchor.0.y + radius;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x != min_x && x != max_x && y != min_y && y != max_y {
                    continue;
                }
                let candidate = GlobalTilePos(IVec2::new(x, y));
                if reserved_positions.contains(&(dim_ref, candidate)) {
                    continue;
                }
                if blocking_tiles.is_blocked_at_tiles_only(dim_ref, candidate, being_ent) {
                    continue;
                }
                return Some(candidate);
            }
        }
    }

    None
}

pub fn try_start_step(
    movement: &mut GridLockedMovement,
    visual: &mut GridLockedMovementVisual,
    blocking_tiles: &mut BlockingTileParamSet,
    dim_ref: DimensionRef,
    being_ent: Entity,
    tile_pos: &mut GlobalTilePos,
    dir: IVec2,
    step_ticks_total: u16,
) -> TryStartStepOutcome {
    if dir == IVec2::ZERO || movement.is_stepping() || step_ticks_total == 0 {
        return if dir == IVec2::ZERO {
            TryStartStepOutcome::IVec2ZeroDir
        } else if movement.is_stepping() {
            TryStartStepOutcome::AlreadyStepping
        } else {
            TryStartStepOutcome::ZeroStepTicks
        };
    }
    let next_tile = GlobalTilePos(tile_pos.0 + dir);
    if !blocking_tiles.can_wall_phase(being_ent) && blocking_tiles.is_blocked_at(dim_ref, next_tile, being_ent) {
        return TryStartStepOutcome::Blocked;
    }
    movement.start_step(visual, tile_pos, dir, step_ticks_total);
    TryStartStepOutcome::Successful
}

pub fn advance_steps_immediate(
    movement: &mut GridLockedMovement,
    visual: &mut GridLockedMovementVisual,
    blocking_tiles: &mut BlockingTileParamSet,
    dim_ref: DimensionRef,
    being_ent: Entity,
    tile_pos: &mut GlobalTilePos,
    dir: IVec2,
    steps: u16,
) -> u16 {
    if dir == IVec2::ZERO || steps == 0 || movement.is_stepping() {
        return 0;
    }
    if blocking_tiles.can_wall_phase(being_ent) {
        for _ in 0..steps {
            tile_pos.0 += dir;
        }
        movement.clear_step(visual, *tile_pos);
        visual.mark_moved();
        return steps;
    }
    let mut steps_taken = 0;
    for _ in 0..steps {
        let next_tile = GlobalTilePos(tile_pos.0 + dir);
        if blocking_tiles.is_blocked_at(dim_ref, next_tile, being_ent) {
            break;
        }
        tile_pos.0 += dir;
        steps_taken += 1;
    }
    if steps_taken > 0 {
        movement.clear_step(visual, *tile_pos);
        visual.mark_moved();
    }
    steps_taken
}
