use bevy::prelude::*;
use sprite_animation_shared::MoveAnimActive;

use crate::movement_components::*;

#[derive(Bundle, Debug, Clone)]
pub struct MovementRemoveOnFreezeBundle(
    pub InputMoveDir,
    pub PendingTileCorrection,
    pub FinalNormMoveDir,
    pub GridLockedMovement,
    pub MoveAnimActive,
);
