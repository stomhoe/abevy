use bevy::prelude::*;
use common::common_components::Prefix;
use game_common::{game_common_components::{EntityZero, ExcludedFromAutoRenamer}, prelude::EntityZeroRef};
use bevy_replicon::prelude::*;

use crate::modifier_components::{ApplyMode, ModifierTarget};

#[derive(Bundle)]
pub struct EntityZeroModifierTargetBundle {
    pub modifier_target: ModifierTarget,
    pub entity_zero: EntityZero,
    pub apply_mode: ApplyMode,
    pub prefix: Prefix,
    pub excluded_from_auto_renamer: ExcludedFromAutoRenamer,
}

impl EntityZeroModifierTargetBundle {
    pub fn new(target: Entity) -> Self {
        Self {
            modifier_target: ModifierTarget(target),
            entity_zero: EntityZero,
            apply_mode: ApplyMode::Add,
            prefix: Prefix::trunc("Modif"),
            excluded_from_auto_renamer: ExcludedFromAutoRenamer,
        }
    }
}

#[derive(Bundle)]
pub struct RefBaseModifierTargetBundle(pub ModifierTarget, pub EntityZeroRef, pub ChildOf);

impl RefBaseModifierTargetBundle {
    pub fn new(target: Entity, ezero_ref: EntityZeroRef, parent: Entity) -> Self {
        Self(ModifierTarget(target), ezero_ref, ChildOf(parent))
    }
}
