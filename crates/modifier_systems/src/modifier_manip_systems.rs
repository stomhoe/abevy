use being::body::{BodyOf, BodySums};
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use game_common::game_common_components::{TemplEnti, TemplEntiRef};
use modifier_shared::modifier_components::{AppliedModifiers, CurrEffectiveValue, ModifierTarget};
use modifier_shared::modifier_types::{ManipulationDexterity, ManipulationStrength};
use modifier_shared::{collect_applied_modifier_entities, modifier_has_marker, resolve_modifier_component};
use ::being_shared::*;


#[allow(unused_parens, )]
pub fn update_body_manipulation_totals(
    bodies: Query<(Entity, &BodyOf),>,
    parts_query: Query<(Entity, &ChildOf, Option<&TemplEntiRef>, Has<Missing>, ), (With<BodypartChildOfBodypart>, Without<TemplEnti>, )>,
    part_applied_mods_query: Query<&AppliedModifiers, >,
    mods: Query<(Entity, &ModifierTarget, Option<&TemplEntiRef>, ), (Without<TemplEnti>, )>,
    curr_values: Query<&CurrEffectiveValue, >,
    dex_markers: Query<(), With<ManipulationDexterity>>,
    strength_markers: Query<(), With<ManipulationStrength>>,
    mut body_health_query: Query<&mut BodySums, >,
    mut dexterity_by_body: Local<EntityHashMap<f32>>,
    mut strength_by_body: Local<EntityHashMap<f32>>,
) {
    dexterity_by_body.clear();
    strength_by_body.clear();

    for (part_ent, part_of, part_templ_ref, is_missing, ) in parts_query.iter() {
        if is_missing {
            continue;
        }
        let body_ent = part_of.parent();
        let mut part_dex = 0.0;
        let mut part_str = 0.0;
        let mut effects = EntityHashSet::default();
        collect_applied_modifier_entities(&mut effects, part_ent, part_templ_ref, &part_applied_mods_query);
        for mod_ent in effects.iter() {
            let Ok((entity, target, templ_ref, )) = mods.get(*mod_ent) else { continue };
            if target.0 != part_ent
                && part_templ_ref.map(|part_templ_ref| target.0 != part_templ_ref.0).unwrap_or(true)
            {
                continue;
            }
            let Some(value) = resolve_modifier_component(entity, templ_ref, &curr_values) else { continue };
            if modifier_has_marker::<ManipulationDexterity>(entity, templ_ref, &dex_markers) {
                part_dex += value.0.max(0.0);
            }
            if modifier_has_marker::<ManipulationStrength>(entity, templ_ref, &strength_markers) {
                part_str += value.0.max(0.0);
            }
        }
        if part_dex > 0.0 {
            *dexterity_by_body.entry(body_ent).or_insert(0.0) += part_dex;
        }
        if part_str > 0.0 {
            *strength_by_body.entry(body_ent).or_insert(0.0) += part_str;
        }
    }

    for (body_ent, body_of, ) in bodies.iter() {
        let Ok(mut health) = body_health_query.get_mut(body_ent) else {
            continue;
        };
        let _ = body_of;
        health.manip_dex = dexterity_by_body.get(&body_ent).copied().unwrap_or(0.0);
        health.manip_str = strength_by_body.get(&body_ent).copied().unwrap_or(0.0);
    }
}
