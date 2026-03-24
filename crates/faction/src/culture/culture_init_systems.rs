use bevy::{ecs::entity::EntityHashMap, prelude::*};
use common::common_components::{DisplayName, Prefix, StrId, Tag};
use game_common::game_common_string_components::Description;

use crate::culture::culture_components::*;
use crate::culture::culture_resources::*;
use faction_shared::Culture;

pub fn init_cultures(
    mut cmd: Commands,
    culture_map: Res<CultureEntityMap>,
) {
    if !culture_map.0.is_empty() {
        return;
    }

    for culture_seri in load_culture_seri_defs() {
        let str_id = StrId::trunc(&culture_seri.id);
        let mut ecmd = cmd.spawn((Culture, str_id.clone()));

        if !culture_seri.name.trim().is_empty() {
            ecmd.insert(DisplayName(culture_seri.name.clone()));
        }
        if let Some(description) = culture_seri.description.as_ref() {
            ecmd.insert(Description(description.clone()));
        }

        if !culture_seri.tags.is_empty() {
            ecmd.insert(CultureTags(
                culture_seri
                    .tags
                    .iter()
                    .map(Tag::trunc)
                    .collect(),
            ));
        }

        if !culture_seri.bit_weightmap.is_empty() {
            let bit_weightmap = culture_seri
                .bit_weightmap
                .iter()
                .map(|(id, weight)| (StrId::trunc(id), (*weight).max(0.0)))
                .collect();
            ecmd.insert(CultureBitWeightMap(bit_weightmap));
        }

        if !culture_seri.races_opinion.is_empty() {
            let races_opinion = culture_seri
                .races_opinion
                .iter()
                .map(|(id, opinion)| (StrId::trunc(id), *opinion))
                .collect();
            ecmd.insert(CultureRacesOpinionStrIds(races_opinion));
        }
    }
}

pub fn resolve_culture_race_opinions(
    mut cmd: Commands,
    race_prefix: Local<RacePrefix>,
    races_query: Query<(Entity, &StrId, &Prefix)>,
    cultures_query: Query<(Entity, &CultureRacesOpinionStrIds), Or<(Added<CultureRacesOpinionStrIds>, Changed<CultureRacesOpinionStrIds>)>>,
) {
    if cultures_query.is_empty() {
        return;
    }

    let mut races_by_id: std::collections::HashMap<StrId, Entity> = std::collections::HashMap::new();
    for (race_ent, race_id, prefix) in races_query.iter() {
        if prefix == &race_prefix.0 {
            races_by_id.insert(race_id.clone(), race_ent);
        }
    }

    let mut to_insert = Vec::new();
    for (culture_ent, race_opinions) in cultures_query.iter() {
        let mut resolved: EntityHashMap<f32> = EntityHashMap::default();
        for (race_id, opinion) in race_opinions.0.iter() {
            if let Some(race_ent) = races_by_id.get(race_id) {
                resolved.insert(*race_ent, *opinion);
            }
        }
        to_insert.push((culture_ent, CultureRacesOpinion(resolved)));
    }
    cmd.try_insert_batch(to_insert);
}
