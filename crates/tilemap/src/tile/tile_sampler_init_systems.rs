#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{StrId};
use game_common::game_common_components_samplers::EntityWeightedSampler;

use crate::tile::{TileWeightedSamplerEntityMap, tile_resources::*, tile_sampler_components::TileWeightedSampler, tile_sampler_resources::*};

#[allow(unused_parens)]
pub fn init_tile_weighted_samplers(
    mut cmd: Commands,
    seris_handles: ResMut<TileWeightedSamplerSerisHandles>,
    assets: Res<Assets<TileWeightedSamplerSeri>>,
    map: Res<TileWeightedSamplerEntityMap>,
) {
    if ! map.0.is_empty() { return; }

    let mut comps_to_insert = Vec::new();

    for handle in seris_handles.handles.iter() {
        let Some(seri) = assets.get(handle) else { continue };
        let Ok(str_id) = StrId::new_with_result(seri.id.clone(), 4) else { continue };
        let ent = cmd.spawn_empty().id();
        comps_to_insert.push((ent, (str_id, EntityWeightedSampler::default(), TileWeightedSampler, )));
    }
    cmd.insert_batch(comps_to_insert);
}

#[allow(unused_parens)]
pub fn init_tile_weighted_samplers_part_two(
    mut cmd: Commands,
    mut seris_handles: ResMut<TileWeightedSamplerSerisHandles>,
    mut assets: ResMut<Assets<TileWeightedSamplerSeri>>,
    hashpos_weighted_map: Res<TileWeightedSamplerEntityMap>,
    tile_ents_map: Res<TileEntityMap>,
) {
    for handle in seris_handles.handles.drain(..) {
        let Some(mut seri) = assets.remove(&handle) else { continue };
        let Ok(wmap_ent) = hashpos_weighted_map.0.get_cloned(&seri.id) else {
            error!("TileWeightedSamplerSeri '{}' not found in TileWeightedSamplerEntityMap", seri.id);
            continue;
        };
        let str_id = &seri.id;
        let mut weights: Vec<(Entity, f32)> = Vec::new();

        for (tile_id, weight) in seri.weights.drain(..) {
            if weight < 0.0 {
                error!("TileWeightedSampler {:?} has negative weight {}, skipping this weighted entry", str_id, weight);
                continue;
            }
            if !tile_id.ends_with("*") {
                if let Ok(ent) = tile_ents_map.0.get_cloned(&tile_id) {
                    if weights.iter().any(|(e, _)| *e == ent) {
                        error!("TileWeightedSampler {:?} already contains tile entity {:?} for id {:?}, skipping duplicate", str_id, ent, tile_id);
                        continue;
                    }
                    weights.push((ent.clone(), weight));
                } else {
                    error!("TileWeightedSampler {:?} references non-existent tile id {:?}, skipping this weighted entry", str_id, tile_id);
                    continue;
                }
            } else {
                let sampler_id_trimmed = tile_id.trim_end_matches('*');
                if let Ok(ent) = hashpos_weighted_map.0.get_cloned(&sampler_id_trimmed.to_string()) {
                    if weights.iter().any(|(e, _)| *e == ent) {
                        error!("TileWeightedSampler {:?} already contains sampler entity {:?} for id {:?}, skipping duplicate", str_id, ent, sampler_id_trimmed);
                        continue;
                    }
                    weights.push((ent.clone(), weight));
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

        cmd.entity(wmap_ent).insert(EntityWeightedSampler::new(&weights));
    }
}
