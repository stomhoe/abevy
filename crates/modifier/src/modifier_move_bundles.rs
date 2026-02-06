#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use game_common::game_common_components::SimRunningDespawnTimer;

use crate::{modifier_components::*, modifier_move_components::*, modifier_types::WalkSpeed};



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
pub struct TemporalSpeedModifier(
    pub SpeedModifier,
    pub SimRunningDespawnTimer,
);
impl TemporalSpeedModifier {
    pub fn new(target: Entity, parent: Entity, value: f32, apply_mode: ApplyMode, duration: f32) -> Self {
        TemporalSpeedModifier(
            SpeedModifier::new(target, parent, value, apply_mode),
            SimRunningDespawnTimer::new(duration),
        )
    }
}
