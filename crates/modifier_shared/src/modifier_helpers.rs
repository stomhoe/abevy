use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use game_common::game_common_components::TemplEntiRef;

use crate::modifier_components::{ApplyMode, BaseValue, CurrEffectiveValue, ModifierTarget};
use crate::modifier_components::AppliedModifiers;

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

pub fn resolve_modifier_component<T: Component + Clone>(
    modi_ent: Entity,
    templ_ref: Option<&TemplEntiRef>,
    query: &Query<&T>,
) -> Option<T> {
    if let Ok(value) = query.get(modi_ent) {
        return Some(value.clone());
    }

    let Some(templ_ref) = templ_ref else {
        return None;
    };
    query.get(templ_ref.0).ok().cloned()
}

pub fn modifier_has_marker<T: Component>(
    instance: Entity,
    templ_ref: Option<&TemplEntiRef>,
    marker_query: &Query<(), With<T>>,
) -> bool {
    marker_query.get(instance).is_ok()
        || templ_ref
            .map(|templ_ref| marker_query.get(templ_ref.0).is_ok())
            .unwrap_or(false)
}

pub fn collect_applied_modifier_entities<'w, 's>(
    effects: &mut EntityHashSet,
    instance: Entity,
    templ_ref: Option<&TemplEntiRef>,
    applied_mods_query: &Query<'w, 's, &AppliedModifiers>,
) {
    if let Ok(applied_mods) = applied_mods_query.get(instance) {
        effects.extend(applied_mods.iter());
    }
    let Some(templ_ref) = templ_ref else {
        return;
    };
    let Ok(applied_mods) = applied_mods_query.get(templ_ref.0) else {
        return;
    };
    effects.extend(applied_mods.iter());
}
