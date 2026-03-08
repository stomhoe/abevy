use bevy::prelude::*;

use crate::modifier_components::{ApplyMode, BaseValue, CurrEffectiveValue, ModifierTarget};

pub fn spawn_modifier<T: Component + Default>(
    cmd: &mut Commands,
    target_and_parent: Entity,
    value: f32,
) {
    spawn_modifier_with_target_and_mode::<T>(cmd, target_and_parent, target_and_parent, value, ApplyMode::Add);
}

pub fn spawn_modifier_with_mode<T: Component + Default>(
    cmd: &mut Commands,
    target_and_parent: Entity,
    value: f32,
    apply_mode: ApplyMode,
) {
    spawn_modifier_with_target_and_mode::<T>(cmd, target_and_parent, target_and_parent, value, apply_mode);
}

pub fn spawn_modifier_with_target<T: Component + Default>(
    cmd: &mut Commands,
    target: Entity,
    parent: Entity,
    value: f32,
) {
    spawn_modifier_with_target_and_mode::<T>(cmd, target, parent, value, ApplyMode::Add);
}

pub fn spawn_modifier_with_target_and_mode<T: Component + Default>(
    cmd: &mut Commands,
    target: Entity,
    parent: Entity,
    value: f32,
    apply_mode: ApplyMode,
) {
    if value == 0.0 {
        return;
    }
    cmd.spawn((
        ModifierTarget(target),
        BaseValue(value),
        CurrEffectiveValue(value),
        apply_mode,
        T::default(),
        ChildOf(parent),
    ));
}
