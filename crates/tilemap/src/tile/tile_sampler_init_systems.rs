#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::tile::{TileWeightedSamplerEntityMap, tile_resources::*, tile_sampler_components::TileWeightedSampler, tile_sampler_resources::*};
use ::common::*;
use tilemap_shared::tilemap_shared_samplers::HashIdWeightedSampler;

#[allow(unused_parens)]
pub fn init_tile_weighted_samplers(
    mut cmd: Commands,
    map: Res<TileWeightedSamplerEntityMap>,
) {
    if ! map.0.is_empty() { return; }

    let mut comps_to_insert = Vec::new();

    for seri in load_tile_weighted_sampler_seri_defs() {
        let Ok(str_id) = StrId::new_with_result(seri.id, 4) else { continue };
        let ent = cmd.spawn_empty().id();
        comps_to_insert.push((ent, (str_id, AddHashIdFromStrId, HashIdWeightedSampler::default(), TileWeightedSampler, )));
    }
    cmd.insert_batch(comps_to_insert);
}

#[allow(unused_parens)]
pub fn init_tile_weighted_samplers_part_two(
    mut cmd: Commands,
    hashpos_weighted_map: Res<TileWeightedSamplerEntityMap>,
    tile_ents_map: Res<TileEntityMap>,
) {
    for mut seri in load_tile_weighted_sampler_seri_defs() {
        let Ok(wmap_ent) = hashpos_weighted_map.0.get_cloned(&seri.id) else {
            error!("TileWeightedSamplerSeri '{}' not found in TileWeightedSamplerEntityMap", seri.id);
            continue;
        };
        let str_id = &seri.id;
        let mut weights: Vec<(HashId, f32)> = Vec::new();

        for (tile_id, weight) in seri.weights.drain(..) {
            if weight < 0.0 {
                error!("TileWeightedSampler {:?} has negative weight {}, skipping this weighted entry", str_id, weight);
                continue;
            }
            if !tile_id.ends_with("*") {
                let tile_hash_id = HashId::from(tile_id.as_str());
                if tile_ents_map.0.get_opt(tile_hash_id).is_some() {
                    if weights.iter().any(|(e, _)| *e == tile_hash_id) {
                        error!("TileWeightedSampler {:?} already contains tile hash {:?} for id {:?}, skipping duplicate", str_id, tile_hash_id, tile_id);
                        continue;
                    }
                    weights.push((tile_hash_id, weight));
                } else {
                    error!("TileWeightedSampler {:?} references non-existent tile id {:?}, skipping this weighted entry", str_id, tile_id);
                    continue;
                }
            } else {
                let sampler_id_trimmed = tile_id.trim_end_matches('*');
                if let Ok(_) = hashpos_weighted_map.0.get_cloned(sampler_id_trimmed) {
                    let sampler_hash_id = HashId::from(sampler_id_trimmed);
                    if weights.iter().any(|(e, _)| *e == sampler_hash_id) {
                        error!("TileWeightedSampler {:?} already contains sampler hash {:?} for id {:?}, skipping duplicate", str_id, sampler_hash_id, sampler_id_trimmed);
                        continue;
                    }
                    weights.push((sampler_hash_id, weight));
                } else {
                    error!("TileWeightedSampler {:?} references non-existent tile id {:?}, skipping this weighted entry", str_id, tile_id);
                    continue;
                }
            }
        }
        if weights.is_empty() {
            error!("TileWeightedSampler {:?} has no valid sampling output", str_id);
            continue;
        }

        let (wmap, negative_items) = HashIdWeightedSampler::new(&weights);
        for negative_item in negative_items {
            error!(target: "tile_sampler_init", "Weighted sampler {} encountered a negative weight for value {:?}; rejected", &str_id, negative_item);
        }
        cmd.entity(wmap_ent).insert(wmap);
    }
}
