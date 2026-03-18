#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use game_common::game_common_timers::SimDespawnTimer;

use crate::{modifier_components::*, modifier_types::WalkSpeed};



#[derive(Bundle, Debug, )]
pub struct SpeedModifier(
    pub ModifierTarget,
    pub BaseValue,
    pub ApplyMode,
    pub WalkSpeed,
    pub ChildOf,
);
impl SpeedModifier {
    pub fn new(target: Entity, parent: Entity,  value: f32, apply_mode: ApplyMode) -> Self {
        SpeedModifier(
            ModifierTarget(target),
            BaseValue(value),
            apply_mode,
            WalkSpeed::default(),
            ChildOf(parent),
        )
    }
}

#[derive(Bundle, Debug, )]
pub struct TempSpeedModifier(
    pub SpeedModifier,
    pub SimDespawnTimer,
);
impl TempSpeedModifier {
    pub fn new(target: Entity, parent: Entity, value: f32, apply_mode: ApplyMode, duration: f32) -> Self {
        TempSpeedModifier(
            SpeedModifier::new(target, parent, value, apply_mode),
            SimDespawnTimer::new(duration),
        )
    }
}
