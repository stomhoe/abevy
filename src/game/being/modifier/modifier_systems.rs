use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::{being::modifier::modifier_components::*, game_components::TimeBasedMultiplier};


#[allow(unused_parens)]
pub fn update_currtime_potency(
    mut query: Query<(&BasePotency, &mut CurrTimeBasedPotency, Option<&TimeBasedMultiplier>),()>
) {
    for (&BasePotency(base_pot), mut currtime_potency, time_multiplier) in query.iter_mut() {

        currtime_potency.0 = base_pot;

        if let Some(time_multiplier) = time_multiplier {
            currtime_potency.0 *= time_multiplier.sample();
        }
    }
}

#[allow(unused_parens)]
pub fn apply_antidotes(
    affected_query: Query<&AppliedModifiers>,
    mut antis_query: Query<(&CurrTimeBasedPotency, &Antidote),()>,
    mut modis_query: Query<(&ModifierCategories, &mut EffectivePotency)>,
) {
    for affected in affected_query.iter() {
        let mut counters_map: HashMap<String, f32> = HashMap::new();
        
        for modif_ent in affected.entities() {
            if let Ok((currtime_potency, antidote)) = antis_query.get_mut(*modif_ent) {
                for (poison_id, effectiveness) in antidote.0.iter() {
                    if let Some(sum) = counters_map.get_mut(poison_id) {
                        *sum += currtime_potency.0 * effectiveness;
                    } else {
                        counters_map.insert(poison_id.clone(), currtime_potency.0 * effectiveness);
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





//HACER Q SI EL PARENT FILTRA MAS RAPIDO TICKEE POR EL FACTOR DADO
#[allow(unused_parens, )]
pub fn tick_time_based_multipliers(
    time: Res<Time>, mut query: Query<(&mut TimeBasedMultiplier), ( With<ModifierCategories>)>
) {
    //for (mut multiplier, ) in query.iter_mut() { multiplier.timer.tick(time.delta()); }
}






