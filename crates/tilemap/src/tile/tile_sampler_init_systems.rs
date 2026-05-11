#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use rand::{rngs::StdRng as Pcg64Mcg, SeedableRng};

use crate::tile::{tile_components::Tile, TileEntityMap, TileWeightedSamplerEntityMap, tile_resources::*, tile_sampler_components::TileWeightedSampler, tile_sampler_resources::*};
use game_common::game_common_components::Templ;
use ::common::*;
use ::tilemap_shared::*;

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

#[allow(unused_parens)]
pub fn sample_tile_normal_size_variations(
    mut cmd: Commands,
    tiles_to_sample: Query<(Entity, &InitialPos, &TileRef), (Changed<InitialPos>, With<Tile>, Without<Templ>, common::AnyDisabling,)>,
    dists_query: Query<(Option<&SpriteGlobalNormalDist>, Option<&SpriteHoriNormalDist>, Option<&SpriteVertNormalDist>)>,
    tile_map: Res<TileEntityMap>,
    gen_settings: Query<&GlobalGenSettings>,
) {
    if tiles_to_sample.is_empty() {
        return;
    }

    let Ok(settings) = gen_settings.single() else {
        error_once!("Failed to get global gen settings for tile normal dist sampling");
        return;
    };

    for (ent, initial_pos, tile_ref) in tiles_to_sample.iter() {
        let Ok((global_dist, hori_dist, vert_dist)) = dists_query.get(ent) else {
            continue;
        };
        let Some(templ_ent) = tile_map.0.get_cloned(tile_ref.0).ok() else {
            error!("Failed to resolve TileRef {:?} while sampling tile normal size variations", tile_ref);
            continue;
        };
        let Ok((templ_global_dist, templ_hori_dist, templ_vert_dist)) = dists_query.get(templ_ent) else {
            continue;
        };

        let global_dist = global_dist.or(templ_global_dist);
        let hori_dist = hori_dist.or(templ_hori_dist);
        let vert_dist = vert_dist.or(templ_vert_dist);
        if global_dist.is_none() && hori_dist.is_none() && vert_dist.is_none() {
            continue;
        }

        let seed = initial_pos.pos.hash_value(settings, initial_pos.dim.0.merge(tile_ref.0), 0);
        let mut rng = Pcg64Mcg::seed_from_u64(seed);
        let mut entity_cmd = cmd.entity(ent);

        if let Some(global_dist) = global_dist {
            entity_cmd.insert(global_dist.sample(&mut rng));
        }
        if let Some(hori_dist) = hori_dist {
            entity_cmd.insert(hori_dist.sample(&mut rng));
        }
        if let Some(vert_dist) = vert_dist {
            entity_cmd.insert(vert_dist.sample(&mut rng));
        }
    }
}
