#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::{common_components::{Prefix, StrId}};
use game_common::game_common_samplers::EntityWeightedSampler;
use crate::sprite_sampler::load_sprite_weighted_sampler_seri_defs;
use crate::{sprite_resources::*, sprite_sampler::{SpriteWeightedSamplerEntityMap, sprite_sampler_components::{EguiSpriteSamplerHolder, SpriteWeightedSampler}, sprite_sampler_resources::*}};



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
                comps_to_insert.push((ent, (str_id, EntityWeightedSampler::default(), ChildOf(holder), SpriteWeightedSampler, )));
            }
    }
    cmd.insert_batch(comps_to_insert);
}

#[allow(unused_parens)]
pub fn init_sprite_weighted_samplers_refs(
    mut cmd: Commands,
    sprite_weighted_map: Res<SpriteWeightedSamplerEntityMap>,
    _hashpos_query: Query<(&StrId, ), (With<EntityWeightedSampler>)>,
    sprite_ents_map: Res<SpriteConfigEntityMap>,
) {
    for mut seri in load_sprite_weighted_sampler_seri_defs() {

        let Ok(wmap_ent) = sprite_weighted_map.0.get_cloned(&seri.id) else {
            error!("SpriteWeightedSamplerSeri '{}' not found in SpriteWeightedSamplerEntityMap", seri.id);
            continue;
        };

        let str_id = &seri.id;
        let mut weights: Vec<(Entity, f32)> = Vec::new();

        for (sprite_id, weight) in seri.weights.drain(..) {
            if weight < 0.0 {
                error!("SpriteWeightedSampler {:?} has negative weight {}, skipping this weighted entry", str_id, weight);
                continue;
            }
            if !sprite_id.ends_with("*") {
                if let Ok(sprite_str_id) = StrId::new_with_result(sprite_id.clone(), 3) {
                    if let Ok(ent) = sprite_ents_map.0.get_cloned(&sprite_str_id) {
                        if weights.iter().any(|(e, _)| *e == ent) {
                            error!("SpriteWeightedSampler {:?} already contains sprite entity {:?} for id {:?}, skipping duplicate", str_id, ent, sprite_id);
                            continue;
                        }
                        weights.push((ent.clone(), weight));
                    } else if let Ok(ent) = sprite_weighted_map.0.get_cloned(&sprite_str_id) {
                        if weights.iter().any(|(e, _)| *e == ent) {
                            error!("SpriteWeightedSampler {:?} already contains sampler entity {:?} for id {:?}, skipping duplicate", str_id, ent, sprite_id);
                            continue;
                        }
                        weights.push((ent.clone(), weight));
                    } else {
                        error!("SpriteWeightedSampler {:?} references non-existent sprite/sampler id {:?}, skipping this weighted entry", str_id, sprite_id);
                        continue;
                    }
                } else {
                    error!("SpriteWeightedSampler {:?} failed to create StrId from sprite id {:?}, skipping this weighted entry", str_id, sprite_id);
                    continue;
                }
            } else {
                let sampler_id_trimmed = sprite_id.trim_end_matches('*');
                if let Ok(sampler_str_id) = StrId::new_with_result(sampler_id_trimmed.to_string(), 3) {
                    if let Ok(ent) = sprite_weighted_map.0.get_cloned(&sampler_str_id) {
                        if weights.iter().any(|(e, _)| *e == ent) {
                            error!("SpriteWeightedSampler {:?} already contains sampler entity {:?} for id {:?}, skipping duplicate", str_id, ent, sampler_id_trimmed);
                            continue;
                        }
                        weights.push((ent.clone(), weight));
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

        cmd.entity(wmap_ent).insert(EntityWeightedSampler::new(&weights));
    }
}
