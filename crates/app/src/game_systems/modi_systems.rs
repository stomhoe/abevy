use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use crate::game::{modifier::modi_components::*, game_components::TimeBasedMultiplier};


#[allow(unused_parens)]
pub fn apply_antidotes(
    affected_query: Query<&AppliedModifiers>,
    mut antis_query: Query<(&BaseValue, Option<&TimeBasedMultiplier>, &Antidote),()>,
    mut modis_query: Query<(&ModifierCategories, &mut EffectiveValue)>,
) {
    for affected in affected_query.iter() {
        let mut counters_map: HashMap<String, f32> = HashMap::new();
        
        for modif_ent in affected.entities() {
            if let Ok((base_potency, time_multiplier, antidote)) = antis_query.get_mut(*modif_ent) {
                for (poison_id, effectiveness) in antidote.0.iter() {
                    if let Some(sum) = counters_map.get_mut(poison_id) {
                        *sum += base_potency.0 * effectiveness * time_multiplier.map_or(1.0, |tm| tm.sample());
                    } else {
                        counters_map.insert(poison_id.clone(), base_potency.0 * effectiveness * time_multiplier.map_or(1.0, |tm| tm.sample()));
                    }
                }
            }
        }

        for modif_ent in affected.entities() {
            if let Ok((modifier_categories, mut effective_potency)) = modis_query.get_mut(*modif_ent) {

                for cat_str in modifier_categories.0.iter() {
                    if let Some(counter_potency) = counters_map.get(cat_str) {
                        effective_potency.0 -= counter_potency;
                    }
                }


            }
        }
    }
}










