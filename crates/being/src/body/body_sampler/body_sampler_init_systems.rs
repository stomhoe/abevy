#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::StrId;
use game_common::game_common_samplers::EntityWeightedSampler;

use crate::body::{body_tree_resources::*, body_sampler::{body_sampler_components::*, body_sampler_resources::*}};


#[allow(unused_parens)]
pub fn init_body_weighted_samplers(
    mut cmd: Commands,
    map: Res<BodyWeightedSamplerEntityMap>,
) {
    if ! map.0.is_empty() { return; }
    let holder = cmd.spawn((EguiBodyWeightedSamplersHolder, )).id();

    let mut comps_to_insert = Vec::new();

    for seri in load_body_weighted_sampler_seri_defs() {
        if let Ok(str_id) = StrId::new_with_result(seri.id, 4) {

            if let Ok(ent) = map.0.get_cloned(&str_id) {
                error!("BodyWeightedSampler '{}' already in BodyWeightedSamplerEntityMap : {:?}", str_id, ent);
                continue;
            }
            let ent = cmd.spawn_empty().id();
            comps_to_insert.push((ent, (str_id, EntityWeightedSampler::default(), ChildOf(holder), BodyWeightedSampler, )));
        }
    }
    cmd.insert_batch(comps_to_insert);
}

#[allow(unused_parens)]
pub fn init_body_weighted_samplers_strid_refs(
    mut cmd: Commands,
    body_weighted_map: Res<BodyWeightedSamplerEntityMap>,
    body_map: Res<BodyTreeEntityMap>,
) {
    for mut seri in load_body_weighted_sampler_seri_defs() {

        let Ok(wmap_ent) = body_weighted_map.0.get_cloned(&seri.id) else {
            error!("BodyWeightedSamplerSeri '{}' not found in BodyWeightedSamplerEntityMap", seri.id);
            continue;
        };

        let str_id = &seri.id;
        let mut weights: Vec<(Entity, f32)> = Vec::new();

        for (body_id, weight) in seri.weights.drain(..) {
            if weight < 0.0 {
                error!("BodyWeightedSampler {:?} has negative weight {}, skipping this weighted entry", str_id, weight);
                continue;
            }
            if !body_id.ends_with("*") {
                if let Ok(body_str_id) = StrId::new_with_result(body_id.clone(), 3) {
                    if let Ok(ent) = body_map.0.get_cloned(&body_str_id) {
                        if weights.iter().any(|(e, _)| *e == ent) {
                            error!("BodyWeightedSampler {:?} already contains body entity {:?} for id {:?}, skipping duplicate", str_id, ent, body_id);
                            continue;
                        }
                        weights.push((ent.clone(), weight));
                    } else {
                        error!("BodyWeightedSampler {:?} references non-existent body id {:?}, skipping this weighted entry", str_id, body_id);
                        continue;
                    }
                } else {
                    error!("BodyWeightedSampler {:?} failed to create StrId from body id {:?}, skipping this weighted entry", str_id, body_id);
                    continue;
                }
            } else {
                let sampler_id_trimmed = body_id.trim_end_matches('*');
                if let Ok(sampler_str_id) = StrId::new_with_result(sampler_id_trimmed.to_string(), 3) {
                    if let Ok(ent) = body_weighted_map.0.get_cloned(&sampler_str_id) {
                        if weights.iter().any(|(e, _)| *e == ent) {
                            error!("BodyWeightedSampler {:?} already contains sampler entity {:?} for id {:?}, skipping duplicate", str_id, ent, sampler_id_trimmed);
                            continue;
                        }
                        weights.push((ent.clone(), weight));
                    } else {
                        error!("BodyWeightedSampler {:?} references non-existent sampler id {:?}, skipping this weighted entry", str_id, sampler_id_trimmed);
                        continue;
                    }
                } else {
                    error!("BodyWeightedSampler {:?} failed to create StrId from sampler id {:?}, skipping this weighted entry", str_id, sampler_id_trimmed);
                    continue;
                }
            }
        }
        if weights.is_empty() {
            error!("BodyWeightedSampler {:?} has no valid sampling output", str_id);
            continue;
        }

        cmd.entity(wmap_ent).insert(EntityWeightedSampler::new(&weights));
    }
}
