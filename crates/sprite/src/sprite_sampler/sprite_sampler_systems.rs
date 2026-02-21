use being_shared::BeingInstTemplate;
use bevy::ecs::entity::EntityHashSet;
#[allow(unused_imports)] use bevy::prelude::*;
use common::{common_components::StrId, };

use game_common::{game_common_components::EntityZero, game_common_samplers::EntityWeightedSampler};
use sprite_shared::{SampleSpriteEnts, SampleSpritesFromStrIds};

use crate::{sprite_components::ScsToBuild, sprite_resources::*, sprite_sampler::SpriteWeightedSamplerEntityMap};

/// Resolves SampleSpritesFromStrIds into SampleSprites by converting string IDs to entities
#[allow(unused_parens)]
pub fn replace_sampler_string_ids_by_entities(
    mut cmd: Commands,
    query: Query<(Entity, &SampleSpritesFromStrIds, Option<&StrId>), (Changed<SampleSpritesFromStrIds>,)>,
    sampler_map: Option<Res<SpriteWeightedSamplerEntityMap>>,
    sprite_map: Option<Res<SpriteConfigEntityMap>>,
) {
    if query.is_empty() {
        return;
    }
    let Some(sprite_map) = sprite_map else {
        if !query.is_empty() {
            error!(target: "sprite_sampler_systems", "SpriteConfigEntityMap not found, cannot replace sampler string ids");
        }
        return;
    };
    let Some(sampler_map) = sampler_map else {
        if !query.is_empty() {
            error!(target: "sprite_sampler_systems", "SpriteWeightedSamplerEntityMap not found, cannot replace sampler string ids");
        }
        return;
    };

    let mut sample_sprites_to_insert = Vec::new();

    for (ent, sampler_ids, strid) in query.iter() {
        debug!(target: "sprite_sampler_systems", "Resolving sampler string ids for {}", strid.cloned().unwrap_or_default());
        let mut resolved_entities = Vec::new();

        for strid in sampler_ids.ids() {
            resolve_sampler_id_no_sample(
                strid,
                &sampler_map,
                &sprite_map,
                &mut resolved_entities,
            );
        }

        if !resolved_entities.is_empty() {
            sample_sprites_to_insert.push((ent, SampleSpriteEnts::new(resolved_entities)));
        }
    }

    cmd.try_insert_batch(sample_sprites_to_insert);
}

/// Samples from SampleSprites entities and creates SpriteCfgsToBuild
#[allow(unused_parens)]
pub fn sample_from_sprite_entities(
    mut cmd: Commands,
    being_query: Query<(Entity, &SampleSpriteEnts, ), (Changed<SampleSpriteEnts>, Without<BeingInstTemplate>, Without<EntityZero>)>,
    samplers_query: Query<&EntityWeightedSampler>,
) {
    if being_query.is_empty() {
        return;
    }

    let mut configs_to_build = Vec::new();

    for (ent, sample_sprites, ) in being_query.iter() {
        debug!(target: "sprite_sampler_systems", "Sampling from sprite entities for entity {:?}", ent);
        let mut sampled_configs = EntityHashSet::new();

        for entity in sample_sprites.entities().iter() {
            sample_from_entity_recursive(
                *entity,
                &samplers_query,
                &mut sampled_configs,
            );
        }

        if !sampled_configs.is_empty() {
            configs_to_build.push((ent, ScsToBuild(sampled_configs)));
        }
    }

    cmd.try_insert_batch(configs_to_build);
}

fn resolve_sampler_id_no_sample(
    id: &common::common_components::StrId,
    sampler_map: &SpriteWeightedSamplerEntityMap,
    sprite_map: &SpriteConfigEntityMap,
    resolved_entities: &mut Vec<Entity>,
) {
    if let Ok(sprite_ent) = sprite_map.0.get_cloned(id) {
        debug!(target: "sprite_sampler_systems", "Resolved sampler string id '{}' to sprite config entity {:?}", id, sprite_ent);
        resolved_entities.push(sprite_ent);
        return;
    }

    if let Ok(sampler_ent) = sampler_map.0.get_cloned(id) {
        debug!(target: "sprite_sampler_systems", "Resolved sampler string id '{}' to sampler entity {:?}", id, sampler_ent);
        resolved_entities.push(sampler_ent);
    } else {
        error!(target: "sprite_sampler_systems", "Sampler string id '{}' not found in either sprite or sampler maps", id);
    }
}

fn sample_from_entity_recursive(
    ent: Entity,
    sampler_query: &Query<&EntityWeightedSampler>,
    sampled_configs: &mut EntityHashSet,
) {
    if let Ok(weighted_sampler) = sampler_query.get(ent) {
        let mut rng = rand::rng();
        if let Some(sampled_ent) = weighted_sampler.sample_with_rng(&mut rng) {
            debug!(target: "sprite_sampler_systems", "Sampled entity {:?} from sampler {:?}", sampled_ent, ent);
            sample_from_entity_recursive(
                sampled_ent,
                sampler_query,
                sampled_configs,
            );
        } else {
            error!(target: "sprite_sampler_systems", "Failed to sample from sampler {:?}, sampler has no valid entries", ent);
        }
    } else {
        debug!(target: "sprite_sampler_systems", "Resolved entity {:?} to sprite config", ent);
        sampled_configs.insert(ent);
    }
}
