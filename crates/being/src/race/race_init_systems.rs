#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_components::Templ;
use game_common::game_common_string_components::*;
use common::common_components::SampleSpriteEnts;
use sprite_systems::{sprite_resources::SpriteConfigEntityMap, sprite_sampler::SpriteWeightedSamplerEntityMap};
use tilemap::terrain::biome::{biome_components::CreatureSampler, biome_resources::BiomeEntityMap};

use sex::sex_resources::SexEntityMap;
use crate::body::BodyTreeEntityMap;
use crate::body::BodyTreeRef;
use crate::body::body_tree_components::*;
use crate::body::body_sampler::body_sampler_resources::*;
use crate::being_interaction_zone_helper::build_being_interaction_zones_with_fallback;
use crate::pack::pack_components::PackInitialSizeSampler;
use crate::pack::pack_components::PackSpawnRadius;
use crate::{sex };
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use ::being_shared::*;
use ::tilemap_shared::*;

pub fn init_races(
    mut cmd: Commands,
    sprite_map: Res<SpriteConfigEntityMap>,
    sampler_map: Option<Res<SpriteWeightedSamplerEntityMap>>,
    sexes_map: Res<SexEntityMap>,
    body_tree_map: Res<BodyTreeEntityMap>,
    body_tree_source_query: Query<(&BodyTreeSexes, Option<&InteractionZones>), With<BodyTree>>,
    body_sampler_map:Res<BodyWeightedSamplerEntityMap>,
    biome_emap: Res<BiomeEntityMap>,
    mut biome_pack_samplers: Query<&mut CreatureSampler>,
) {
    for race_seri in load_race_seri_defs() {
        let str_id = StrId::trunc(&race_seri.id);

        let ingame_name = DisplayName(race_seri.name.clone());
        let description = race_seri.description.trim();
        let demonym = race_seri.demonym.trim();

        let singular_str = if race_seri.singular.trim().is_empty() {
            race_seri.name.clone()
        } else {
            race_seri.singular.clone()
        };
        let plural_str = if race_seri.plural.trim().is_empty() {
            format!("{}s", singular_str)
        } else {
            race_seri.plural.clone()
        };
        let singular = SingularDenomination(singular_str.into());
        let plural = PluralDenomination(plural_str.into());

        let resolve_sprite_entities = |ids: &Vec<String>| -> Vec<Entity> {
            let mut resolved = Vec::new();
            for raw_id in ids {
                let trimmed = raw_id.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let sprite_str_id = StrId::trunc(trimmed);
                if let Ok(entity) = sprite_map.0.get_cloned(&sprite_str_id) {
                    resolved.push(entity);
                    continue;
                }

                if let Some(sampler_map) = sampler_map.as_ref() {
                    if let Ok(entity) = sampler_map.0.get_cloned(&sprite_str_id) {
                        resolved.push(entity);
                        continue;
                    }
                }

                warn!(
                    target: "race_init",
                    "Race '{}' sprite sampler '{}' not found in SpriteConfigEntityMap or SpriteWeightedSamplerEntityMap",
                    str_id,
                    trimmed
                );
            }
            resolved
        };

        let fallback_sprite_entities = resolve_sprite_entities(&race_seri.fallback_sprites_to_sample);
        if fallback_sprite_entities.is_empty() && !race_seri.fallback_sprites_to_sample.is_empty() {
            warn!(
                target: "race_init",
                "Race '{}' fallback sprites resolved to empty set",
                str_id
            );
        }

        let mut mapped_sprites_to_sample: EntityHashMap<SampleSpriteEnts> = EntityHashMap::default();

        let sets_of_monochoosable_sprites = {
            let mut labeled_monochoosable_sets = Vec::new();
            for (group_label, sprite_ids) in &race_seri.sets_of_choosable_sprites {
                let mut sprite_set = EntityHashSet::new();
                for sprite_id in sprite_ids {
                    let trimmed = sprite_id.trim();
                    if !trimmed.is_empty() {
                        if trimmed == "none"{
                            sprite_set.insert(Entity::PLACEHOLDER);
                        }
                        else{
                            match sprite_map.0.get_cloned(trimmed) {
                                Ok(entity) => {
                                    sprite_set.insert(entity);
                                }
                                Err(_) => {
                                    error!(target: "race_init", "Race '{}' selectable sprite '{}' not found in SpriteConfigEntityMap", str_id, trimmed);
                                }
                            }
                        }
                    }
                }
                if !sprite_set.is_empty() {
                    labeled_monochoosable_sets.push((StrId::trunc(group_label), sprite_set));
                }
            }

            if !labeled_monochoosable_sets.is_empty() {
                Some(SetsOfPlayerMonoChoosableSprites(labeled_monochoosable_sets))
            } else {
                None
            }
        };
        let mut entity_cmds = cmd.spawn((Race, Templ, str_id.clone(), ingame_name, singular, plural));
        let body_tree_str_id = StrId::trunc(&race_seri.body_or_sampler);
        let mut body_tree_ent = None;

        if let Ok(body_sampler_ent) = body_sampler_map.0.get_cloned(&body_tree_str_id) {
            entity_cmds.insert(BodyWeightedSamplerRef(body_sampler_ent));
        } else if let Ok(tree_ent) = body_tree_map.0.get_cloned(&body_tree_str_id) {
            entity_cmds.insert(BodyTreeRef(tree_ent));
            body_tree_ent = Some(tree_ent);
        }
        let body_tree_zones = body_tree_ent
            .and_then(|body_tree_ent| body_tree_source_query.get(body_tree_ent).ok())
            .and_then(|(_, zones)| zones);
        entity_cmds.insert(build_being_interaction_zones_with_fallback(
            body_tree_zones,
            race_seri.melee_interaction_zone.clone(),
            race_seri.collision_zone.clone(),
        ));

        if !description.is_empty() {
            entity_cmds.insert(Description(description.to_string()));
        }
        if !demonym.is_empty() {
            entity_cmds.insert(Demonym(demonym.to_string().into()));
        }
        if let Some(selectable) = sets_of_monochoosable_sprites {
            entity_cmds.insert(selectable);
        }
        if !race_seri.size_variation.is_sentinel() {
            entity_cmds.insert(SpriteGlobalNormalDist::new(race_seri.size_variation.clone()));
        }
        if !race_seri.hori_variation.is_sentinel() {
            entity_cmds.insert(SpriteHoriNormalDist::new(race_seri.hori_variation.clone()));
        }
        if !race_seri.vert_variation.is_sentinel() {
            entity_cmds.insert(SpriteVertNormalDist::new(race_seri.vert_variation.clone()));
        }
        if race_seri.produces_step_sfx {
            entity_cmds.insert(ProducesStepSfx);
        }
        if !race_seri.footstep_sfx.paths.is_empty() || race_seri.footstep_sfx.disable_tile_step_sfx {
            entity_cmds.insert(RaceFootstepSfxConfig {
                paths: race_seri.footstep_sfx.paths.clone(),
                disable_tile_step_sfx: race_seri.footstep_sfx.disable_tile_step_sfx,
            });
        }

        entity_cmds.insert(race_seri.tags_with_my_id());
        let entity = entity_cmds.id();
        if !race_seri.spawn_pack_size_normal_dist.is_sentinel() {
            cmd.entity(entity).insert(PackInitialSizeSampler(CappedNormalDist::from_seri(
                race_seri.spawn_pack_size_normal_dist.clone(),
            )));
        }
        cmd.entity(entity).insert(PackSpawnRadius(race_seri.pack_spawn_radius));
        if !race_seri.spawn_pack_entity {
            cmd.entity(entity).insert(NoSpawnSquadEntity);
        }
        if let Some(predator_cfg) = PredatorCfg::from_seri(&race_seri.predator) {
            cmd.entity(entity).insert(predator_cfg);
        }
        if !race_seri.wander.is_disabled() {
            cmd.entity(entity).insert(race_seri.wander.clone().sanitized());
        }
        if !race_seri.whitelisted_spawn_tile_tags.is_empty() {
            cmd.entity(entity).insert(WhitelistedSpawnTileTags::new(&race_seri.whitelisted_spawn_tile_tags));
        }
        if !race_seri.blacklisted_spawn_tile_tags.is_empty() || !race_seri.blacklisted_tiles_for_spawning.is_empty() {
            cmd.entity(entity).insert(BlacklistedSpawnTileTags::new(
                race_seri
                    .blacklisted_spawn_tile_tags
                    .iter()
                    .chain(race_seri.blacklisted_tiles_for_spawning.iter()),
            ));
        }

        let body_tree_sexes = body_tree_ent
            .and_then(|body_tree_ent| body_tree_source_query.get(body_tree_ent).ok())
            .map(|(sexes, _)| sexes);

        if let Some(body_tree_sexes) = body_tree_sexes {
            let mut sex_entities_weights: Vec<(Entity, f32)> = Vec::new();
            let mut sex_size_variations = bevy::ecs::entity::EntityHashMap::default();
            for (sex_id, sex_cfg) in &body_tree_sexes.0 {
                match sexes_map.0.get_cloned(sex_id) {
                    Ok(sex_entity) => {
                        sex_entities_weights.push((sex_entity, sex_cfg.weight as f32));
                        if let Some(size_var) = sex_cfg.size_variation.clone().filter(|v| !v.is_sentinel()) {
                            sex_size_variations.insert(sex_entity, SpriteGlobalNormalDist::new(size_var));
                        }

                        let mut resolved_entities = if sex_cfg.sprites.is_empty() {
                            fallback_sprite_entities.clone()
                        } else {
                            resolve_sprite_entities(&sex_cfg.sprites)
                        };

                        if resolved_entities.is_empty() && !fallback_sprite_entities.is_empty() {
                            resolved_entities = fallback_sprite_entities.clone();
                        }

                        if resolved_entities.is_empty() {
                            warn!(
                                target: "race_init",
                                "Race '{}' sex '{}' has no resolved sprites to sample",
                                str_id,
                                sex_id
                            );
                        } else {
                            mapped_sprites_to_sample.insert(sex_entity, SampleSpriteEnts::new(resolved_entities));
                        }
                    }
                    Err(_) => {
                        warn!(target: "race_init", "Race '{}' sex '{}' not found in SexEntityMap", str_id, sex_id);
                    }
                }
            }
            if !sex_entities_weights.is_empty() {
                let sex_sampler = SexesSampler::new(&sex_entities_weights);
                cmd.entity(entity).insert(sex_sampler);
            }
            if !sex_size_variations.is_empty() {
                cmd.entity(entity).insert(SexSizeVariationsBySex(sex_size_variations));
            }
        } else {
            warn!(target: "race_init", "BodyTree '{}' has no BodyTreeSexes; race '{}' will not get sex sampler data", body_tree_str_id, str_id);
        }
        if race_seri.scale_hp_and_strength_with_size {
            cmd.entity(entity).insert(ScaleHpAndStrengthWithSampledSize);
        }

        cmd.entity(entity)
            .insert(SexMappedSpritesToSample(mapped_sprites_to_sample));

        for (biome_tag, weight) in &race_seri.biome_affinity {
            if *weight == 0.0 {
                continue;
            }
            let Ok(biome_ent) = biome_emap.0.get_cloned(biome_tag) else {
                continue;
            };
            let Ok(mut biome_pack_sampler) = biome_pack_samplers.get_mut(biome_ent) else {
                continue;
            };
            biome_pack_sampler.add_affinity(entity, *weight);
        }

        trace!(target: "race_init", "Initialized race '{}' with entity {:?}", str_id, entity);
    }
}
