use being::body::{BodySums, BodyOf};
use being::body::body_part::body_part_components::{BodyPartMissing, BodyPartOf};
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use modifier_shared::modifier_components::{CurrEffectiveValue, ModifierTarget};
use modifier_shared::modifier_types::{ManipulationDexterity, ManipulationStrength};

pub fn update_body_manipulation_totals(
    bodies: Query<Entity, With<BodyOf>>,
    part_of_query: Query<(&BodyPartOf, Has<BodyPartMissing>)>,
    dex_mods: Query<(&ModifierTarget, &CurrEffectiveValue), With<ManipulationDexterity>>,
    strength_mods: Query<(&ModifierTarget, &CurrEffectiveValue), With<ManipulationStrength>>,
    mut body_health_query: Query<&mut BodySums>,
) {
    let mut dexterity_by_body: EntityHashMap<f32> = EntityHashMap::default();
    let mut strength_by_body: EntityHashMap<f32> = EntityHashMap::default();

    for (target, value) in dex_mods.iter() {
        let Some(body) = target_body(target.0, &part_of_query, &bodies) else {
            continue;
        };
        *dexterity_by_body.entry(body).or_insert(0.0) += value.0.max(0.0);
    }

    for (target, value) in strength_mods.iter() {
        let Some(body) = target_body(target.0, &part_of_query, &bodies) else {
            continue;
        };
        *strength_by_body.entry(body).or_insert(0.0) += value.0.max(0.0);
    }

    for body in bodies.iter() {
        let Ok(mut health) = body_health_query.get_mut(body) else {
            continue;
        };
        health.manip_dex = dexterity_by_body.get(&body).copied().unwrap_or(0.0);
        health.manip_str = strength_by_body.get(&body).copied().unwrap_or(0.0);
    }
}

fn target_body(
    target: Entity,
    part_of_query: &Query<(&BodyPartOf, Has<BodyPartMissing>)>,
    bodies: &Query<Entity, With<BodyOf>>,
) -> Option<Entity> {
    if bodies.get(target).is_ok() {
        return Some(target);
    }
    let Ok((part_of, is_missing)) = part_of_query.get(target) else {
        return None;
    };
    if is_missing {
        return None;
    }
    Some(part_of.body)
}
