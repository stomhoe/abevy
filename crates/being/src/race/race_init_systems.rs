use being_shared::MappedSpritesToSample;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_components::EntityZero;
use game_common::game_common_samplers::*;
use game_common::game_common_string_components::*;
use sprite_shared::SampleSpriteEnts;
use sprite::{sprite_resources::SpriteConfigEntityMap, sprite_sampler::SpriteWeightedSamplerEntityMap};

use sex::sex_resources::SexEntityMap;
use crate::body::BodyTreeStrIdRef;
use crate::{race::{race_components::*, race_resources::*}, sex };
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use ::being_shared::{Predator, PredatorHuntThreshold};

pub fn init_races(
    mut cmd: Commands,
    sprite_map: Res<SpriteConfigEntityMap>,
    sampler_map: Option<Res<SpriteWeightedSamplerEntityMap>>,
    sexes_map: Res<SexEntityMap>,
) {
    for race_seri in load_race_seri_defs() {
            let str_id = StrId::trunc(&race_seri.id);

            let ingame_name = DisplayName(race_seri.name.clone());
            let description = race_seri.description.as_ref().map(|d| Description(d.clone()));
            let demonym = race_seri.demonym.as_ref().map(|d| Demonym(d.clone().into()));

            let singular_str = race_seri.singular.unwrap_or_else(|| race_seri.name.clone());
            let plural_str = race_seri.plural
                .unwrap_or_else(|| format!("{}s", singular_str));
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
                if let Some(sets) = &race_seri.sets_of_choosable_sprites {
                    let mut labeled_monochoosable_sets = Vec::new();
                    for (group_label, sprite_ids) in sets {
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
                } else {
                    None
                }
            };

            let mut entity_cmds = cmd.spawn((Race, EntityZero, str_id.clone(), ingame_name, singular, plural));

            let body_tree_str_id = StrId::trunc(&race_seri.body_tree);
            entity_cmds.insert(BodyTreeStrIdRef(body_tree_str_id));

            if let Some(desc) = description {
                entity_cmds.insert(desc);
            }
            if let Some(dem) = demonym {
                entity_cmds.insert(dem);
            }

            if let Some(selectable) = sets_of_monochoosable_sprites {
                entity_cmds.insert(selectable);
            }

            if let Some(size_variation) = race_seri.size_variation {
                entity_cmds.insert(SpriteGlobalNormalDist::new(size_variation));
            }
            if let Some(hori_variation) = race_seri.hori_variation {
                entity_cmds.insert(SpriteHoriNormalDist::new(hori_variation));
            }
            if let Some(vert_variation) = race_seri.vert_variation {
                entity_cmds.insert(SpriteVertNormalDist::new(vert_variation));
            }

            let entity = entity_cmds.id();
            if PredatorHuntThreshold::is_configured_in_seri(race_seri.predator_hunt_threshold) {
                cmd.entity(entity).insert((
                    Predator,
                    PredatorHuntThreshold(race_seri.predator_hunt_threshold),
                ));
            }

            if !race_seri.sexes.is_empty() {
                let mut sex_entities_weights: Vec<(Entity, f32)> = Vec::new();
                for (sex_id, (weight, sprite_ids)) in &race_seri.sexes {
                    match sexes_map.0.get_cloned(sex_id) {
                        Ok(sex_entity) => {
                            sex_entities_weights.push((sex_entity, *weight as f32));

                            let mut resolved_entities = if sprite_ids.is_empty() {
                                fallback_sprite_entities.clone()
                            } else {
                                resolve_sprite_entities(sprite_ids)
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
            }
            if race_seri.scale_hp_and_strength_with_size {
                cmd.entity(entity).insert(ScaleHpAndStrengthWithSize);
            }

            cmd.entity(entity)
                .insert(MappedSpritesToSample(mapped_sprites_to_sample));

            trace!(target: "race_init", "Initialized race '{}' with entity {:?}", str_id, entity);
    }
}
