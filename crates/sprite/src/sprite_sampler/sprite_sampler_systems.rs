use being_shared::BeingInstTemplate;
use bevy::ecs::{entity::EntityHashSet, entity_disabling::Disabled};
use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;
use common::{AnyDisabling, common_components::{HashId, StrId}, };

use game_common::game_common_components::Templ;
use tilemap_shared::tilemap_shared_samplers::HashIdWeightedSampler;
use common::common_components::SampleSpritesamplers;
use ::sprite_shared::*;

use crate::{sprite_resources::*, sprite_sampler::SpriteWeightedSamplerEntityMap};

/// Resolves SampleSpritesFromStrIds into SampleSprites by converting string IDs to entities
#[allow(unused_parens)]
pub fn replace_sampler_string_ids_by_entities(
    mut cmd: Commands,
    changed_query: Query<Entity, Changed<SampleSpritesFromStrIds>>,
    query: Query<(&SampleSpritesFromStrIds, Option<&StrId>, Has<SampleSpritesamplers>), AnyDisabling>,
    sampler_map: Option<Res<SpriteWeightedSamplerEntityMap>>,
    sprite_map: Option<Res<SpriteConfigEntityMap>>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut resolved_entities: Local<Vec<Entity>>,
    mut entities_to_process: Local<Vec<Entity>>,
) {
    let mut sample_sprites_to_insert = Vec::new();
    resolved_entities.clear();
    entities_to_process.clear();

    entities_to_process.extend(removed_disabled.read());
    entities_to_process.extend(changed_query.iter());
    if entities_to_process.is_empty() {
        return;
    }
    let Some(sprite_map) = sprite_map else {
        error!(target: "sprite_sampler_systems", "SpriteConfigEntityMap not found, cannot replace sampler string ids");
        return;
    };
    let Some(sampler_map) = sampler_map else {
        error!(target: "sprite_sampler_systems", "SpriteWeightedSamplerEntityMap not found, cannot replace sampler string ids");
        return;
    };
    if sprite_map.0.is_empty(){
        error!(target: "sprite_sampler_systems", "SpriteConfigEntityMap has no entries, cannot replace sampler string ids");
    }

    for ent in entities_to_process.drain(..) {
        let Ok((sampler_ids, strid, has_sample_sprites)) = query.get(ent) else {
            continue;
        };
        let is_reenabled_only = changed_query.get(ent).is_err();
        if is_reenabled_only && has_sample_sprites {
            continue;
        }
        debug!(target: "sprite_sampler_systems", "Resolving sampler string ids for {}", strid.cloned().unwrap_or_default());
        resolved_entities.clear();

        for strid in sampler_ids.ids() {
            resolve_sampler_id_no_sample(
                strid,
                &sampler_map,
                &sprite_map,
                &mut resolved_entities,
            );
        }

        if !resolved_entities.is_empty() {
            sample_sprites_to_insert.push((ent, SampleSpritesamplers(resolved_entities.drain(..).collect())));
        }
    }

    cmd.try_insert_batch(std::mem::take(&mut sample_sprites_to_insert));
}

/// Samples from SampleSprites entities and creates SpriteCfgsToBuild
#[allow(unused_parens)]
pub fn sample_from_sprite_entities(
    mut cmd: Commands,
    changed_beings: Query<Entity, (Changed<SampleSpritesamplers>, Without<BeingInstTemplate>, Without<Templ>)>,
    being_query: Query<(&SampleSpritesamplers, Has<ScsToBuild>), (Without<BeingInstTemplate>, Without<Templ>, AnyDisabling)>,
    samplers_query: Query<&HashIdWeightedSampler>,
    sprite_hash_query: Query<&HashId, (With<SpriteConfig>, AnyDisabling)>,
    sampler_map: Res<SpriteWeightedSamplerEntityMap>,
    sprite_map: Res<SpriteConfigEntityMap>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut beings_to_process: Local<Vec<Entity>>,
    mut visited: Local<EntityHashSet>,
) {
    let mut configs_to_build = Vec::new();
    let mut sampled_configs = HashSet::new();
    beings_to_process.clear();
    visited.clear();

    beings_to_process.extend(removed_disabled.read());
    beings_to_process.extend(changed_beings.iter());
    if beings_to_process.is_empty() {
        return;
    }

    for ent in beings_to_process.iter() {
        let Ok((sample_sprites, has_scs_to_build)) = being_query.get(*ent) else {
            continue;
        };
        let is_reenabled_only = changed_beings.get(*ent).is_err();
        if is_reenabled_only && has_scs_to_build {
            continue;
        }
        debug!(target: "sprite_sampler_systems", "Sampling from sprite entities for entity {:?}", ent);
        sampled_configs.clear();
        visited.clear();

        for entity in sample_sprites.0.iter() {
            sample_from_entity_recursive(
                *entity,
                &samplers_query,
                &sprite_hash_query,
                &sampler_map,
                &sprite_map,
                &mut sampled_configs,
                &mut visited,
            );
        }

        if !sampled_configs.is_empty() {
            configs_to_build.push((*ent, ScsToBuild(std::mem::take(&mut sampled_configs))));
        }
        cmd.entity(*ent).try_remove::<SampleSpritesamplers>();
    }

    cmd.try_insert_batch(std::mem::take(&mut configs_to_build));
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
    sampler_query: &Query<&HashIdWeightedSampler>,
    sprite_hash_query: &Query<&HashId, (With<SpriteConfig>, AnyDisabling)>,
    sampler_map: &SpriteWeightedSamplerEntityMap,
    sprite_map: &SpriteConfigEntityMap,
    sampled_configs: &mut HashSet<HashId>,
    visited: &mut EntityHashSet,
) {
    if let Ok(weighted_sampler) = sampler_query.get(ent) {
        if !visited.insert(ent) {
            warn!(target: "sprite_sampler_systems", "Detected cycle while sampling sprite sampler graph at entity {:?}", ent);
            return;
        }
            let mut rng = rand::rng();
            if let Some(sampled_hash_id) = weighted_sampler.sample_with_rng(&mut rng) {
                debug!(target: "sprite_sampler_systems", "Sampled hash {:?} from sampler {:?}", sampled_hash_id, ent);
                let Some(sampled_ent) = sampler_map
                    .0
                    .get_opt(sampled_hash_id)
                    .copied()
                    .or_else(|| sprite_map.0.get_opt(sampled_hash_id).copied()) else {
                    error!(target: "sprite_sampler_systems", "Sampled hash {:?} from sampler {:?} was not found in sprite/sampler maps", sampled_hash_id, ent);
                    visited.remove(&ent);
                    return;
                };
                sample_from_entity_recursive(
                    sampled_ent,
                    sampler_query,
                    sprite_hash_query,
                    sampler_map,
                    sprite_map,
                    sampled_configs,
                    visited,
                );
            } else {
                error!(target: "sprite_sampler_systems", "Failed to sample from sampler {:?}, sampler has no valid entries", ent);
        }
        visited.remove(&ent);
    } else {
        let Ok(hash_id) = sprite_hash_query.get(ent) else {
            error!(target: "sprite_sampler_systems", "Resolved entity {:?} to sprite config but it has no HashId", ent);
            return;
        };
        debug!(target: "sprite_sampler_systems", "Resolved entity {:?} to sprite config hash {}", ent, hash_id);
        sampled_configs.insert(*hash_id);
    }
}
