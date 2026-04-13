use bevy::prelude::*;

use modifier_shared::AppliedModifiers;
use sprite_animation_shared::MoveAnimActive;
use tilemap_shared::SnapTransformToGpos;

use crate::movement_shared_components::*;

#[derive(Bundle, Debug, Default, Clone)]
pub struct GridLockedMovementRequirementsBundle(
    pub InputMoveDir,
    pub InputInvMul,
    pub FinalNormMoveDir,
    pub SpeedPotential,
    pub SpeedMagnitude,
    pub AppliedModifiers,
    pub SnapTransformToGpos,
);

#[derive(Bundle, Debug, Default, Clone)]
pub struct MoveVisualsBundle(
    pub GridLockedMovementVisual,
    pub MoveAnimActive
);
