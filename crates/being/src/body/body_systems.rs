use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::platform::collections::HashSet;
use common::common_components::HashId;
use game_common::game_common_components::{Templ, TemplEntiRef};
use modifier_shared::{collect_applied_modifier_entities, modifier_has_marker, resolve_modifier_component};
use modifier_shared::modifier_item_types::MassKg;
use modifier_shared::modifier_components::*;
use tilemap_shared::{Dimension, DimensionRef, Gravity};
use tilemap_shared::DimensionEntityMap;

use crate::body::{body_components::*,};
use ::being_shared::*;

#[derive(SystemParam)]
pub struct BodyWeightSumQueries<'w, 's> {
    body_changed_query: Query<'w, 's, Entity, (With<BodyOf>, Or<(Added<BodyOf>, Changed<BodypartChildrenBodyparts>)>)>,
    being_dim_changed_query: Query<'w, 's, Entity, (With<Being>, Changed<DimensionRef>)>,
    part_body_changed_query: Query<'w, 's, &'static ChildOf, (With<BodypartChildOfBodypart>, Or<(Changed<BodypartChildOfBodypart>, Changed<Missing>, Changed<BodypartChildrenBodyparts>)>)>,
    parts_query: Query<'w, 's, (Entity, &'static ChildOf, Option<&'static TemplEntiRef>, Has<Missing>), With<BodypartChildOfBodypart>>,
    part_applied_mods_query: Query<'w, 's, &'static AppliedModifiers>,
    mass_modifiers_query: Query<'w, 's, (Entity, &'static ModifierTarget, Option<&'static TemplEntiRef>), Without<Templ>>,
    curr_values_query: Query<'w, 's, &'static CurrEffectiveValue>,
    mass_markers_query: Query<'w, 's, (), With<MassKg>>,
    body_of_query: Query<'w, 's, (Entity, &'static BodyOf), With<BodyOf>>,
    being_dim_query: Query<'w, 's, &'static DimensionRef, With<Being>>,
    dimension_map: Res<'w, DimensionEntityMap>,
    dimension_hash_query: Query<'w, 's, &'static HashId, With<Dimension>>,
    being_weight_query: Query<'w, 's, &'static mut BodyWeightSum, With<Being>>,
    gravity_changed_query: Query<'w, 's, Entity, (With<Dimension>, Changed<Gravity>)>,
    gravity_query: Query<'w, 's, &'static Gravity, With<Dimension>>,
    removed_missing: RemovedComponents<'w, 's, Missing>,
}

#[derive(SystemParam)]
pub struct BodyWeightSumLocals<'s> {
    body_and_dim_cache: Local<'s, (EntityHashSet, HashSet<HashId>)>,
}

#[allow(unused_parens, )]
pub fn update_body_weight_sum(
    mut cmd: Commands,
    queries: BodyWeightSumQueries,
    mut locals: BodyWeightSumLocals,
) {
    let BodyWeightSumQueries {
        body_changed_query,
        being_dim_changed_query,
        part_body_changed_query,
        parts_query,
        part_applied_mods_query,
        mass_modifiers_query,
        curr_values_query,
        mass_markers_query,
        body_of_query,
        being_dim_query,
        dimension_map,
        dimension_hash_query,
        mut being_weight_query,
        gravity_changed_query,
        gravity_query,
        mut removed_missing,
    } = queries;
    let BodyWeightSumLocals {
        body_and_dim_cache,
    } = &mut locals;
    let (affected_bodies, changed_dims) = &mut **body_and_dim_cache;
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
        let Ok(&dim_hash) = dimension_hash_query.get(dim_ent) else {
            continue;
        };
        changed_dims.insert(dim_hash);
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
        let Some(dim_ent) = dimension_map.0.get_opt(dim_ref.0).copied() else {
            continue;
        };
        let gravity = gravity_query.get(dim_ent).copied().unwrap_or_default();
        let total_mass = mass_per_body.get(body_ent).copied().unwrap_or_default().max(0.0);
        let total_weight = gravity.mass_to_newtons(total_mass).max(0.0);
        let Ok(mut prev_weight) = being_weight_query.get_mut(body_of.being) else {
            cmd.entity(body_of.being).try_insert(BodyWeightSum(total_weight));
            continue;
        };
        if (prev_weight.0 - total_weight).abs() > f32::EPSILON {
            prev_weight.0 = total_weight;
        }
    }
}
