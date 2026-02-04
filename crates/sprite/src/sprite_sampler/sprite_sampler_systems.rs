use being_shared::BeingInstTemplate;
use bevy::ecs::entity::EntityHashSet;
#[allow(unused_imports)] use bevy::prelude::*;
use common::common_components::AnyDisabling;
use common::common_id_components::HashId;
use dimension_shared::DimensionRef;
use game_common::game_common_components_samplers::EntityWeightedSampler;
use sprite_shared::{SampleSprites, SampleSpritesFromStrIds};
use tilemap_shared::{GlobalGenSettings, GlobalTilePos};

use crate::{SpriteCfgEntityMap, sprite_components::SpriteCfgsToBuild, sprite_sampler::{SpriteWeightedSampler, SpriteWeightedSamplersMap}};

/// Resolves SampleSpritesFromStrIds into SampleSprites by converting string IDs to entities
#[allow(unused_parens)]
pub fn replace_sampler_string_ids_by_entities(
    mut cmd: Commands,
    query: Query<(Entity, &SampleSpritesFromStrIds), (Changed<SampleSpritesFromStrIds>,)>,
    sampler_map: Option<Res<SpriteWeightedSamplersMap>>,
    sprite_map: Option<Res<SpriteCfgEntityMap>>,
) {
    let Some(sampler_map) = sampler_map else {
        if !query.is_empty() {
            error!(target: "sprite_sampler_systems", "SpriteWeightedSamplersMap not found, cannot replace sampler string ids");
        }
        return;
    };

    let Some(sprite_map) = sprite_map else {
        if !query.is_empty() {
            error!(target: "sprite_sampler_systems", "SpriteCfgEntityMap not found, cannot replace sampler string ids");
        }
        return;
    };

    let mut sample_sprites_to_insert = Vec::new();

    for (ent, sampler_ids) in query.iter() {
        debug!(target: "sprite_sampler_systems", "Resolving sampler string ids for entity {:?}", ent);
        let mut resolved_entities = Vec::new();

        for strid in sampler_ids.ids() {
            // Try to resolve each strid to an entity
            resolve_sampler_id_no_sample(
                strid,
                &sampler_map,
                &sprite_map,
                &mut resolved_entities,
            );
        }

        if !resolved_entities.is_empty() {
            sample_sprites_to_insert.push((ent, SampleSprites::new(resolved_entities)));
        }
    }

    cmd.try_insert_batch(sample_sprites_to_insert);
}

/// Samples from SampleSprites entities and creates SpriteCfgsToBuild
#[allow(unused_parens)]
pub fn sample_from_sprite_entities(
    mut cmd: Commands,
    global_gen_settings: Single<&GlobalGenSettings>,
    query: Query<(Entity, &SampleSprites, AnyOf<(&GlobalTilePos, &Transform)>, &DimensionRef), (Changed<SampleSprites>,
        Without<BeingInstTemplate>
    )>,
    sampler_query: Query<&EntityWeightedSampler>,
    dimension_hash_query: Query<&HashId, AnyDisabling>,
) {
    let mut configs_to_build = Vec::new();

    for (ent, sample_sprites, (gpos, transform), dimension_ref) in query.iter() {
        debug!(target: "sprite_sampler_systems", "Sampling from sprite entities for entity {:?}", ent);
        let mut sampled_configs = EntityHashSet::new();

        // Use provided GlobalTilePos or compute from Transform
        let pos = if let Some(gpos) = gpos {
            *gpos
        } else if let Some(transform) = transform {
            GlobalTilePos::from(transform.translation.xy())
        } else {
            continue;
        };
        
        // Get dimension hash from the dimension entity
        let Ok(dim_hash) = dimension_hash_query.get(dimension_ref.0).copied() else {
            continue;
        };

        // Sample from each entity in SampleSprites
        for entity in sample_sprites.entities().iter() {
            sample_from_entity_recursive(
                *entity,
                &sampler_query,
                &mut sampled_configs,
                pos,
                &global_gen_settings,
                dim_hash,
            );
        }

        if !sampled_configs.is_empty() {
            configs_to_build.push((ent, SpriteCfgsToBuild(sampled_configs)));
        }
    }

    cmd.try_insert_batch(configs_to_build);
}

/// Resolves a sampler ID to an entity without sampling (for initial resolution)
fn resolve_sampler_id_no_sample(
    id: &common::common_components::StrId,
    sampler_map: &SpriteWeightedSamplersMap,
    sprite_map: &SpriteCfgEntityMap,
    resolved_entities: &mut Vec<Entity>,
) {
    // First, try to resolve as a sprite config
    if let Ok(sprite_ent) = sprite_map.0.get_cloned(id) {
        debug!(target: "sprite_sampler_systems", "Resolved sampler string id '{}' to sprite config entity {:?}", id, sprite_ent);
        resolved_entities.push(sprite_ent);
        return;
    }

    // If not a sprite config, try to resolve as a sampler
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
    pos: GlobalTilePos,
    gen_settings: &GlobalGenSettings,
    dim_hash: HashId,
) {
    // Check if this entity is a sampler
    if let Ok(weighted_sampler) = sampler_query.get(ent) {
        // It's a sampler, sample from it using position-based hashing
        if let Some(sampled_ent) = weighted_sampler.sample_with_pos(pos, gen_settings, dim_hash) {
            debug!(target: "sprite_sampler_systems", "Sampled entity {:?} from sampler {:?}", sampled_ent, ent);
            // Recursively resolve the sampled entity
            sample_from_entity_recursive(
                sampled_ent,
                sampler_query,
                sampled_configs,
                pos,
                gen_settings,
                dim_hash,
            );
        } else {
            error!(target: "sprite_sampler_systems", "Failed to sample from sampler {:?}, sampler has no valid entries", ent);
        }
    } else {
        // Not a sampler, treat it as a sprite config and add it
        debug!(target: "sprite_sampler_systems", "Resolved entity {:?} to sprite config", ent);
        sampled_configs.insert(ent);
    }
}
