use bevy::prelude::*;

use crate::movement_components::*;

#[derive(Bundle, Debug, Clone)]
pub struct MovementRemoveOnFreezeBundle(
    pub GridLockedMovement,
    pub InputMoveDir,
    pub FinalNormMoveDir,
    pub PendingTileCorrection,
    pub sprite_animation_shared::MoveAnimActive,
    pub tilemap_shared::SnapTransformToGpos,
);
