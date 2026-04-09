#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::{common_components::{StrId}};
use common::common_id_components::HashId;
use tilemap_shared::tilemap_shared_samplers::HashIdWeightedSampler;
use crate::sprite_sampler::load_sprite_weighted_sampler_seri_defs;
use crate::{sprite_resources::*, sprite_sampler::{SpriteWeightedSamplerEntityMap, sprite_sampler_components::{EguiSpriteSamplerHolder, SpriteWeightedSampler}}};



#[allow(unused_parens)]
pub fn init_sprite_weighted_samplers(
    mut cmd: Commands,
    map: Res<SpriteWeightedSamplerEntityMap>,
) {
    if ! map.0.is_empty() { return; }
    let holder = cmd.spawn((EguiSpriteSamplerHolder, )).id();

    let mut comps_to_insert = Vec::new();

    for seri in load_sprite_weighted_sampler_seri_defs() {
            if let Ok(str_id) = StrId::new_with_result(seri.id, 4) {

                if let Ok(ent) = map.0.get_cloned(&str_id) {
                    error!("SpriteWeightedSampler '{}' already in SpriteWeightedSamplerEntityMap : {:?}", str_id, ent);
                    continue;
                }
                let ent = cmd.spawn_empty().id();
                comps_to_insert.push((ent, (str_id, HashIdWeightedSampler::default(), ChildOf(holder), SpriteWeightedSampler, )));
            }
    }
    cmd.insert_batch(comps_to_insert);
}

#[allow(unused_parens)]
pub fn init_sprite_weighted_samplers_refs(
    mut cmd: Commands,
    sprite_weighted_map: Res<SpriteWeightedSamplerEntityMap>,
    _hashpos_query: Query<(&StrId, ), (With<HashIdWeightedSampler>)>,
    sprite_ents_map: Res<SpriteConfigEntityMap>,
) {
    for mut seri in load_sprite_weighted_sampler_seri_defs() {

        let Ok(wmap_ent) = sprite_weighted_map.0.get_cloned(&seri.id) else {
            error!("SpriteWeightedSamplerSeri '{}' not found in SpriteWeightedSamplerEntityMap", seri.id);
            continue;
        };

        let str_id = &seri.id;
        let mut weights: Vec<(HashId, f32)> = Vec::new();

        for (sprite_id, weight) in seri.weights.drain(..) {
            if weight < 0.0 {
                error!("SpriteWeightedSampler {:?} has negative weight {}, skipping this weighted entry", str_id, weight);
                continue;
            }
            if !sprite_id.ends_with("*") {
                if let Ok(sprite_str_id) = StrId::new_with_result(sprite_id.clone(), 3) {
                    let sprite_hash_id = HashId::from(sprite_str_id.as_str());
                    if sprite_ents_map.0.get_opt(sprite_hash_id).is_some() {
                        if weights.iter().any(|(e, _)| *e == sprite_hash_id) {
                            error!("SpriteWeightedSampler {:?} already contains sprite hash {:?} for id {:?}, skipping duplicate", str_id, sprite_hash_id, sprite_id);
                            continue;
                        }
                        weights.push((sprite_hash_id, weight));
                    } else {
                        let sampler_hash_id = HashId::from(sprite_str_id.as_str());
                        if sprite_weighted_map.0.get_opt(sampler_hash_id).is_some() {
                            if weights.iter().any(|(e, _)| *e == sampler_hash_id) {
                                error!("SpriteWeightedSampler {:?} already contains sampler hash {:?} for id {:?}, skipping duplicate", str_id, sampler_hash_id, sprite_id);
                                continue;
                            }
                            weights.push((sampler_hash_id, weight));
                        } else {
                            error!("SpriteWeightedSampler {:?} references non-existent sprite/sampler id {:?}, skipping this weighted entry", str_id, sprite_id);
                            continue;
                        }
                    }
                } else {
                    error!("SpriteWeightedSampler {:?} failed to create StrId from sprite id {:?}, skipping this weighted entry", str_id, sprite_id);
                    continue;
                }
            } else {
                let sampler_id_trimmed = sprite_id.trim_end_matches('*');
                if let Ok(sampler_str_id) = StrId::new_with_result(sampler_id_trimmed.to_string(), 3) {
                    let sampler_hash_id = HashId::from(sampler_str_id.as_str());
                    if sprite_weighted_map.0.get_opt(sampler_hash_id).is_some() {
                        if weights.iter().any(|(e, _)| *e == sampler_hash_id) {
                            error!("SpriteWeightedSampler {:?} already contains sampler hash {:?} for id {:?}, skipping duplicate", str_id, sampler_hash_id, sampler_id_trimmed);
                            continue;
                        }
                        weights.push((sampler_hash_id, weight));
                    } else {
                        error!("SpriteWeightedSampler {:?} references non-existent sampler id {:?}, skipping this weighted entry", str_id, sampler_id_trimmed);
                        continue;
                    }
                } else {
                    error!("SpriteWeightedSampler {:?} failed to create StrId from sampler id {:?}, skipping this weighted entry", str_id, sampler_id_trimmed);
                    continue;
                }
            }
        }
        if weights.is_empty() {
            error!("SpriteWeightedSampler {:?} has no valid sampling output", str_id);
            continue;
        }

        let (wmap, negative_indices) = HashIdWeightedSampler::new(&weights);
        if !negative_indices.is_empty() {
            tilemap_shared::log_negative_weighted_sampler_indices!("sprite_sampler_init", &str_id, &weights, negative_indices);
        }
        cmd.entity(wmap_ent).insert(wmap);
    }
}
