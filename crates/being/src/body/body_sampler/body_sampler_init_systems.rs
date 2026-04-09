#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{AddHashIdFromStrId, StrId};
use common::common_id_components::HashId;
use game_common::game_common_components::TemplEntiHashIdRef;
use tilemap_shared::tilemap_shared_samplers::HashIdWeightedSampler;

use crate::body::{body_resources::*, body_sampler::{body_sampler_components::*, body_sampler_resources::*}};


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
            let hash_id = HashId::from(str_id.as_str());
            comps_to_insert.push((ent, (str_id, AddHashIdFromStrId, TemplEntiHashIdRef(hash_id), HashIdWeightedSampler::default(), ChildOf(holder), BodyWeightedSampler, )));
        }
    }
    cmd.insert_batch(comps_to_insert);
}

#[allow(unused_parens)]
pub fn init_body_weighted_samplers_strid_refs(
    mut cmd: Commands,
    body_weighted_map: Res<BodyWeightedSamplerEntityMap>,
    body_map: Res<BodyEntityMap>,
) {
    for mut seri in load_body_weighted_sampler_seri_defs() {

        let Ok(wmap_ent) = body_weighted_map.0.get_cloned(&seri.id) else {
            error!("BodyWeightedSamplerSeri '{}' not found in BodyWeightedSamplerEntityMap", seri.id);
            continue;
        };

        let str_id = &seri.id;
        let mut weights: Vec<(HashId, f32)> = Vec::new();

        for (body_id, weight) in seri.weights.drain(..) {
            if weight < 0.0 {
                error!("BodyWeightedSampler {:?} has negative weight {}, skipping this weighted entry", str_id, weight);
                continue;
            }
            if !body_id.ends_with("*") {
                if let Ok(body_str_id) = StrId::new_with_result(body_id.clone(), 3) {
                    let body_hash_id = HashId::from(body_str_id.as_str());
                    if body_map.0.get_opt(body_hash_id).is_some() {
                        if weights.iter().any(|(e, _)| *e == body_hash_id) {
                            error!("BodyWeightedSampler {:?} already contains body hash {:?} for id {:?}, skipping duplicate", str_id, body_hash_id, body_id);
                            continue;
                        }
                        weights.push((body_hash_id, weight));
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
                    let sampler_hash_id = HashId::from(sampler_str_id.as_str());
                    if body_weighted_map.0.get_opt(sampler_hash_id).is_some() {
                        if weights.iter().any(|(e, _)| *e == sampler_hash_id) {
                            error!("BodyWeightedSampler {:?} already contains sampler hash {:?} for id {:?}, skipping duplicate", str_id, sampler_hash_id, sampler_id_trimmed);
                            continue;
                        }
                        weights.push((sampler_hash_id, weight));
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

        let (wmap, negative_indices) = HashIdWeightedSampler::new(&weights);
        if !negative_indices.is_empty() {
            tilemap_shared::log_negative_weighted_sampler_indices!("being_body_sampler_init", &str_id, &weights, negative_indices);
        }
        cmd.entity(wmap_ent).insert(wmap);
    }
}
