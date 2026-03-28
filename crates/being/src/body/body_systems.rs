use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use game_common::game_common_components::{Templ, TemplEntiRef};
use modifier_shared::{collect_applied_modifier_entities, modifier_has_marker, resolve_modifier_component};
use modifier_shared::modifier_item_types::MassKg;
use modifier_shared::modifier_components::*;
use tilemap_shared::{Dimension, DimensionRef, Gravity};

use crate::body::{body_tree_components::*,};
use ::being_shared::*;

pub fn update_body_tree_weight_sum(
    mut cmd: Commands,
    body_changed_query: Query<Entity, (With<BodyOf>, Or<(Added<BodyOf>, Changed<BodypartChildrenBodyparts>)>)>,
    being_dim_changed_query: Query<Entity, (With<Being>, Changed<DimensionRef>)>,
    part_body_changed_query: Query<&ChildOf, (With<BodypartChildOfBodypart>, Or<( Changed<BodypartChildOfBodypart>, Changed<Missing>, Changed<BodypartChildrenBodyparts>)>)>,
    parts_query: Query<(Entity, &ChildOf, Option<&TemplEntiRef>, Has<Missing>), With<BodypartChildOfBodypart>>,
    part_applied_mods_query: Query<&AppliedModifiers>,
    mass_modifiers_query: Query<(Entity, &ModifierTarget, Option<&TemplEntiRef>), Without<Templ>>,
    curr_values_query: Query<&CurrEffectiveValue>,
    mass_markers_query: Query<(), With<MassKg>>,
    body_of_query: Query<(Entity, &BodyOf), With<BodyOf>>,
    being_dim_query: Query<&DimensionRef, With<Being>>,
    being_weight_query: Query<&BodyTreeWeightSum, With<Being>>,
    gravity_changed_query: Query<Entity, (With<Dimension>, Changed<Gravity>)>,
    gravity_query: Query<&Gravity, With<Dimension>>,
    mut removed_missing: RemovedComponents<Missing>,
    mut body_and_dim_cache: Local<(EntityHashSet, EntityHashSet)>,
) {
    let (affected_bodies, changed_dims) = &mut *body_and_dim_cache;
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
        affected_bodies.insert(body_of.parent());
    }
    for part_ent in removed_missing.read() {
        let Ok((_, body_of, _, _)) = parts_query.get(part_ent) else { continue };
        affected_bodies.insert(body_of.parent());
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

    let mut mass_per_body: EntityHashMap<f32> = EntityHashMap::default();
    for (part_ent, body_of, part_templ_ref, missing) in parts_query.iter() {
        let body_ent = body_of.parent();
        if missing || !affected_bodies.contains(&body_ent) {
            continue;
        }
        let mut part_mass = 0.0;
        let mut effects = EntityHashSet::default();
        collect_applied_modifier_entities(&mut effects, part_ent, part_templ_ref, &part_applied_mods_query);
        for mod_ent in effects.iter() {
            let Ok((entity, target, templ_ref)) = mass_modifiers_query.get(*mod_ent) else {
                continue;
            };
            if target.0 != part_ent
                && part_templ_ref.map(|part_templ_ref| target.0 != part_templ_ref.0).unwrap_or(true)
            {
                continue;
            }
            if !modifier_has_marker::<MassKg>(entity, templ_ref, &mass_markers_query) {
                continue;
            }
            let Some(value) = resolve_modifier_component(entity, templ_ref, &curr_values_query) else {
                continue;
            };
            part_mass += value.0.max(0.0);
        }
        *mass_per_body.entry(body_ent).or_insert(0.0) += part_mass;
    }

    for body_ent in affected_bodies.iter() {
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
