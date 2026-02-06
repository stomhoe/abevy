use being_shared::BeingInstTemplate;
#[allow(unused_imports)] use bevy::prelude::*;
use common::common_id_components::HashId;
use dimension_shared::DimensionRef;
use game_common::game_common_components_samplers::EntityWeightedSampler;
use tilemap_shared::{GlobalGenSettings, GlobalTilePos};

use crate::{body::{body_components::*, body_resources::*, body_sampler::{BodyWeightedSamplersMap, body_sampler_components::*}, }, race::race_components::Race};

/// Resolves SampleBodiesFromStrIds into SampleBodies by converting string IDs to entities
#[allow(unused_parens)]
pub fn replace_body_sampler_string_id_by_entity(
    mut cmd: Commands,
    query: Query<(Entity, &SampleBodyFromStrId), (Changed<SampleBodyFromStrId>,)>,
    sampler_map: Option<Res<BodyWeightedSamplersMap>>,
    body_map: Option<Res<BodyTreeEntityMap>>,
) {
    let Some(sampler_map) = sampler_map else {
        if !query.is_empty() {
            error!(target: "body_sampler_systems", "BodyWeightedSamplersMap not found, cannot replace sampler string ids");
        }
        return;
    };

    let Some(body_map) = body_map else {
        if !query.is_empty() {
            error!(target: "body_sampler_systems", "BodyTreeEntityMap not found, cannot replace sampler string ids");
        }
        return;
    };

    let mut sample_bodies_to_insert = Vec::new();

    for (ent, sampler_strid) in query.iter() {
        debug!(target: "body_sampler_systems", "Resolving sampler string ids for entity {:?}", ent);
        
        if let Some(resolved_ent) = resolve_sampler_id_no_sample(
            &sampler_strid.id(),
            &sampler_map,
            &body_map,
        ) {
            sample_bodies_to_insert.push((ent, SampleBody::new(resolved_ent)));
        }
    }

    cmd.try_insert_batch(sample_bodies_to_insert);
}

fn resolve_sampler_id_no_sample(
    id: &common::common_components::StrId,
    sampler_map: &BodyWeightedSamplersMap,
    body_map: &BodyTreeEntityMap,
) -> Option<Entity>{
    if let Ok(body_ent) = body_map.0.get_cloned(id) {
        debug!(target: "body_sampler_systems", "Resolved sampler string id '{}' to body config entity {:?}", id, body_ent);
        return Some(body_ent);
    }

    if let Ok(sampler_ent) = sampler_map.0.get_cloned(id) {
        debug!(target: "body_sampler_systems", "Resolved sampler string id '{}' to sampler entity {:?}", id, sampler_ent);
        return Some(sampler_ent);
    } else {
        error!(target: "body_sampler_systems", "Sampler string id '{}' not found in either body or sampler maps", id);
        return None;
    }
}

/// Samples from SampleBodies entities and creates BodyCfgsToBuild
#[allow(unused_parens)]
pub fn sample_from_body_entities(
    mut cmd: Commands,
    global_gen_settings: Single<&GlobalGenSettings>,
    query: Query<(Entity, &SampleBody, AnyOf<(&GlobalTilePos, &Transform)>, &DimensionRef), (Changed<SampleBody>,
        Without<BeingInstTemplate>, Without<Race>
    )>,
    sampler_query: Query<&EntityWeightedSampler>,
    dimension_hash_query: Query<&HashId, common::AnyDisabling>,
) {
    let mut configs_to_build = Vec::new();

    for (ent, sample_body, (gpos, transform), dimension_ref) in query.iter() {
        debug!(target: "body_sampler_systems", "Sampling from body entity for entity {:?}", ent);

        let pos = if let Some(gpos) = gpos {
            *gpos
        } else if let Some(transform) = transform {
            GlobalTilePos::from(transform.translation.xy())
        } else {
            continue;
        };

        let Ok(dim_hash) = dimension_hash_query.get(dimension_ref.0).copied() else {
            continue;
        };

        if let Some(resolved_config) = sample_from_entity_recursive(
            *sample_body.entity(),
            &sampler_query,
            pos,
            &global_gen_settings,
            dim_hash,
        ) {
            configs_to_build.push((ent, BodyTreeToBuild(resolved_config)));
        }
    }

    cmd.try_insert_batch(configs_to_build);
}



fn sample_from_entity_recursive(
    ent: Entity,
    sampler_query: &Query<&EntityWeightedSampler>,
    pos: GlobalTilePos,
    gen_settings: &GlobalGenSettings,
    dim_hash: HashId,
) -> Option<Entity> {
    if let Ok(weighted_sampler) = sampler_query.get(ent) {
        if let Some(sampled_ent) = weighted_sampler.sample_with_pos(pos, gen_settings, dim_hash) {
            debug!(target: "body_sampler_systems", "Sampled entity {:?} from sampler {:?}", sampled_ent, ent);
            return sample_from_entity_recursive(
                sampled_ent,
                sampler_query,
                pos,
                gen_settings,
                dim_hash,
            );
        } else {
            error!(target: "body_sampler_systems", "Failed to sample from sampler {:?}, sampler has no valid entries", ent);
            return None;
        }
    } else {
        debug!(target: "body_sampler_systems", "Resolved entity {:?} to body config", ent);
        return Some(ent);
    }
}
