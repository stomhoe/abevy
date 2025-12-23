#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{EntityPrefix, StrId};
use game_common::game_common_components_samplers::EntityWeightedSampler;

use crate::{tile::{tile_components::*, tile_resources::*, tile_sampler_resources::*}};

#[allow(unused_parens)]
pub fn init_tile_weighted_samplers(
    mut cmd: Commands, 
    seris_handles: ResMut<TileWeightedSamplerHandles>,
    assets: Res<Assets<TileWeightedSamplerSeri>>,
    map: Option<Res<TileWeightedSamplersMap>>,
) {
    if map.is_some() { return; }
    let holder = cmd.spawn((TileSamplerHolder, )).id();
    let mut map = TileWeightedSamplersMap::default();

    let mut comps_to_insert = Vec::new();
    
    for handle in seris_handles.handles.iter() {
        if let Some(seri) = assets.get(handle) {
            //info!("Loading TileWeightedSamplerSeri from handle: {:?}", handle);
            
            if let Ok(str_id) = StrId::new_with_result(seri.id.clone(), 4) {

                if let Ok(ent) = map.0.get(&str_id) {
                    error!("TileWeightedSampler '{}' already in TileWeightedSamplersMap : {:?}", str_id, ent);
                    continue;
                }
                let ent = cmd.spawn_empty().id();
                map.0.force_insert(&str_id, ent);
                comps_to_insert.push((ent, (str_id, EntityWeightedSampler::default(), ChildOf(holder), )));
            }
        }
    }
    cmd.insert_resource(map);
    cmd.insert_batch(comps_to_insert);
} 

#[allow(unused_parens)]
pub fn init_tile_weighted_samplers_refs(
    mut cmd: Commands, 
    mut seris_handles: ResMut<TileWeightedSamplerHandles>,
    mut assets: ResMut<Assets<TileWeightedSamplerSeri>>,
    hashpos_weighted_map: Res<TileWeightedSamplersMap>,
    hashpos_query: Query<(&StrId, ), (With<EntityWeightedSampler>)>,
    tile_ents_map: Res<TileEntitiesMap>,
) {
    for handle in seris_handles.handles.drain(..) {
        let Some(mut seri) = assets.remove(&handle) else { continue };

        let wmap_ent = match hashpos_weighted_map.0.get(&seri.id) {
            Ok(ent) => ent,
            Err(err) => {
                error!("TileWeightedSamplerSeri '{}' not found in HashPosWeightedSamplersMap: {}", seri.id, err);
                continue;
            }
        };

        let str_id = &seri.id;
        let mut weights: Vec<(Entity, f32)> = Vec::new();

        for (tile_id, weight) in seri.weights.drain(..) {
            if weight < 0.0 {
                error!("TileWeightedSampler {:?} has negative weight {}, skipping this weighted entry", str_id, weight);
                continue;
            }
            if !tile_id.ends_with("*") {
                if let Ok(ent) = tile_ents_map.0.get(&tile_id) {
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
                if let Ok(ent) = hashpos_weighted_map.0.get(&sampler_id_trimmed.to_string()) {
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




