#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use game_common::game_common_components::SimDespawnTimer;

use crate::{modifier_components::*, modifier_move_components::*};



#[derive(Bundle, Debug, )]
pub struct SpeedModifier(
    pub ModifierTarget,
    pub CurrFinalValue,
    pub ApplyMode,
    pub Speed,
);
impl SpeedModifier {
    pub fn new(target: Entity, value: f32, apply_mode: ApplyMode) -> Self {
        SpeedModifier(
            ModifierTarget(target),
            CurrFinalValue(value),
            apply_mode,
            Speed::default(),
        )
    }
}

#[derive(Bundle, Debug, )]
pub struct TemporalSpeedModifier(
    pub SpeedModifier,
    pub SimDespawnTimer,
);
impl TemporalSpeedModifier {
    pub fn new(target: Entity, value: f32, apply_mode: ApplyMode, duration: f32) -> Self {
        TemporalSpeedModifier(
            SpeedModifier::new(target, value, apply_mode),
            SimDespawnTimer::new(duration),
        )
    }
}
