use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use modifier_shared::modifier_item_types::MassKg;
use modifier_shared::modifier_components::*;
use tilemap_shared::{Dimension, DimensionRef, Gravity};

use crate::being_components::Being;
use crate::body::{body_tree_components::*, body_part::body_part_components::*};

pub fn update_body_tree_weight_sum(
    mut cmd: Commands,
    body_exists_query: Query<(), With<BodyOf>>,
    body_changed_query: Query<Entity, (With<BodyOf>, Or<(Added<BodyOf>, Changed<BodyParts>)>)>,
    being_dim_changed_query: Query<Entity, (With<Being>, Changed<DimensionRef>)>,
    part_body_changed_query: Query<&BodyPartOf, (With<BodyPart>, Or<(Added<BodyPartOf>, Changed<BodyPartOf>, Changed<BodyPartMissing>, Changed<Children>)>)>,
    parts_query: Query<(Entity, &BodyPartOf, Has<BodyPartMissing>), With<BodyPart>>,
    mass_modifiers_changed_query: Query<&ModifierTarget, (With<MassKg>, Changed<CurrEffectiveValue>)>,
    mass_modifiers_query: Query<(&ModifierTarget, &CurrEffectiveValue), With<MassKg>>,
    body_of_query: Query<(Entity, &BodyOf), With<BodyOf>>,
    being_dim_query: Query<&DimensionRef, With<Being>>,
    being_weight_query: Query<&BodyTreeWeightSum, With<Being>>,
    gravity_changed_query: Query<Entity, (With<Dimension>, Changed<Gravity>)>,
    gravity_query: Query<&Gravity, With<Dimension>>,
    mut removed_missing: RemovedComponents<BodyPartMissing>,
    mut affected_bodies: Local<EntityHashSet>,
    mut changed_dims: Local<EntityHashSet>,
) {
    affected_bodies.clear();
    changed_dims.clear();
    for body_ent in body_changed_query.iter() {
        affected_bodies.insert(body_ent);
    }
    for being_ent in being_dim_changed_query.iter() {
        for (body_ent, body_of) in body_of_query.iter() {
            if body_of.being != being_ent {
                continue;
            }
            affected_bodies.insert(body_ent);
        }
    }
    for dim_ent in gravity_changed_query.iter() {
        changed_dims.insert(dim_ent);
    }
    for body_of in part_body_changed_query.iter() {
        affected_bodies.insert(body_of.body);
    }
    for part_ent in removed_missing.read() {
        let Ok((_, body_of, _)) = parts_query.get(part_ent) else { continue };
        affected_bodies.insert(body_of.body);
    }
    for target in mass_modifiers_changed_query.iter() {
        let Ok((_, body_of, _)) = parts_query.get(target.0) else { continue };
        affected_bodies.insert(body_of.body);
    }
    if !changed_dims.is_empty() {
        for (body_ent, body_of) in body_of_query.iter() {
            let Ok(dim_ref) = being_dim_query.get(body_of.being) else { continue };
            if changed_dims.contains(&dim_ref.0) {
                affected_bodies.insert(body_ent);
            }
        }
    }
    if affected_bodies.is_empty() {
        return;
    }

    let mut mass_per_part: EntityHashMap<f32> = EntityHashMap::default();
    for (target, value) in mass_modifiers_query.iter() {
        *mass_per_part.entry(target.0).or_insert(0.0) += value.0.max(0.0);
    }

    let mut mass_per_body: EntityHashMap<f32> = EntityHashMap::default();
    for (part_ent, body_of, missing) in parts_query.iter() {
        if missing || !affected_bodies.contains(&body_of.body) {
            continue;
        }
        let part_mass = mass_per_part.get(&part_ent).copied().unwrap_or_default();
        *mass_per_body.entry(body_of.body).or_insert(0.0) += part_mass;
    }

    for body_ent in affected_bodies.iter() {
        let Ok(()) = body_exists_query.get(*body_ent) else { continue };
        let Ok((_, body_of)) = body_of_query.get(*body_ent) else { continue };
        let Ok(dim_ref) = being_dim_query.get(body_of.being) else { continue };
        let gravity = gravity_query.get(dim_ref.0).copied().unwrap_or_default();
        let total_mass = mass_per_body.get(body_ent).copied().unwrap_or_default().max(0.0);
        let total_weight = gravity.mass_to_newtons(total_mass).max(0.0);
        let Ok(prev_weight) = being_weight_query.get(body_of.being) else {
            cmd.entity(body_of.being).try_insert(BodyTreeWeightSum(total_weight));
            continue;
        };
        if (prev_weight.0 - total_weight).abs() > f32::EPSILON {
            cmd.entity(body_of.being).try_insert(BodyTreeWeightSum(total_weight));
        }
    }
}
