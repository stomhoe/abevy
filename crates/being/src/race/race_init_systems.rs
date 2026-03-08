use being_shared::MappedSpritesToSample;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_id_components::{HashId, HashIdMap};
use game_common::game_common_components::EntityZero;
use game_common::game_common_samplers::*;
use game_common::game_common_string_components::*;
use sprite_shared::SampleSpriteEnts;
use sprite::{sprite_resources::SpriteConfigEntityMap, sprite_sampler::SpriteWeightedSamplerEntityMap};

use sex::sex_resources::SexEntityMap;
use crate::body::BodyTreeEntityMap;
use crate::body::BodyTreeRef;
use crate::body::body_part::body_part_components::*;
use crate::body::body_tree_components::BodyTreeDistributedTotals;
use crate::body::BodyTreeStrIdRef;
use crate::body::body_sampler::body_sampler_resources::BodyWeightedSamplerEntityMap;
use crate::body::body_sampler::body_sampler_resources::BodyWeightedSamplerRef;
use crate::{race::{race_components::*, race_resources::*}, sex };
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use ::being_shared::{Predator, PredatorHuntThreshold};
use tilemap_shared::InteractionZones;

pub fn init_races(
    mut cmd: Commands,
    sprite_map: Res<SpriteConfigEntityMap>,
    sampler_map: Option<Res<SpriteWeightedSamplerEntityMap>>,
    sexes_map: Res<SexEntityMap>,
    body_tree_map: Res<BodyTreeEntityMap>,
    body_sampler_map:Res<BodyWeightedSamplerEntityMap>,
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

            let mut entity_cmds = cmd.spawn((Race, EntityZero, str_id.clone(), ingame_name, singular, plural));
            let mut interaction_zones = bevy::platform::collections::HashMap::with_capacity(1);
            interaction_zones.insert("melee".to_string(), race_seri.melee_interaction_zone.clone());
            entity_cmds.insert(InteractionZones::new(interaction_zones));
            let mut totals = HashIdMap::default();
            for (key, val) in &race_seri.distributed_totals {
                totals.overwrite(HashId::from(key), val.max(0.0));
            }
            if !totals.contains_key(BodyPartStat::STAT_HP_CAPACITY) {
                totals.overwrite(BodyPartStat::STAT_HP_CAPACITY, 1.0);
            }
            if !totals.contains_key(BodyPartStat::STAT_HP_REGEN_RATE) {
                totals.overwrite(BodyPartStat::STAT_HP_REGEN_RATE, 1.0);
            }
            if !totals.contains_key(BodyPartStat::STAT_BLOOD_CAPACITY) {
                totals.overwrite(BodyPartStat::STAT_BLOOD_CAPACITY, 1.0);
            }
            if !totals.contains_key(BodyPartStat::STAT_VISION) {
                totals.overwrite(BodyPartStat::STAT_VISION, 1.0);
            }
            if !totals.contains_key(BodyPartStat::STAT_CALORIC_BURN_RATE) {
                totals.overwrite(BodyPartStat::STAT_CALORIC_BURN_RATE, 1.0);
            }
            if !totals.contains_key(BodyPartStat::STAT_WALK_SPEED) {
                totals.overwrite(BodyPartStat::STAT_WALK_SPEED, 300.);
            }
            if !totals.contains_key(BodyPartStat::STAT_MASS_KG) {
                totals.overwrite(BodyPartStat::STAT_MASS_KG, race_seri.mass_kg.max(0.0));
            }
            entity_cmds.insert((
                BodyTreeDistributedTotals(totals),
            ));

            let body_tree_str_id = StrId::trunc(&race_seri.body_tree_or_sampler);

            if let Ok(body_sampler_ent) = body_sampler_map.0.get_cloned(&body_tree_str_id) {
                entity_cmds.insert(BodyWeightedSamplerRef(body_sampler_ent));
            } else if let Ok(body_tree_ent) = body_tree_map.0.get_cloned(&body_tree_str_id) {
                entity_cmds.insert(BodyTreeRef(body_tree_ent));
            }

            if !description.is_empty() {
                entity_cmds.insert(Description(description.to_string()));
            }
            if !demonym.is_empty() {
                entity_cmds.insert(Demonym(demonym.to_string().into()));
            }
            if let Some(selectable) = sets_of_monochoosable_sprites {
                entity_cmds.insert(selectable);
            }
            if !normal_dist_is_disabled(&race_seri.size_variation) {
                entity_cmds.insert(SpriteGlobalNormalDist::new(race_seri.size_variation.clone()));
            }
            if !normal_dist_is_disabled(&race_seri.hori_variation) {
                entity_cmds.insert(SpriteHoriNormalDist::new(race_seri.hori_variation.clone()));
            }
            if !normal_dist_is_disabled(&race_seri.vert_variation) {
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

            let entity = entity_cmds.id();
            if PredatorHuntThreshold::is_configured_in_seri(race_seri.predator_hunt_threshold) {
                let mut own_races = bevy::platform::collections::HashSet::default();
                for race_id in &race_seri.friend_races {
                    let trimmed = race_id.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    own_races.insert(StrId::trunc(trimmed));
                }
                own_races.insert(str_id.clone());
                let (mut pack_min, mut pack_max) = race_seri.predator_pack_size_range;
                if pack_min == 0 {
                    pack_min = 1;
                }
                if pack_max < pack_min {
                    pack_max = pack_min;
                }
                cmd.entity(entity).insert((
                    Predator {
                        own_races,
                        territorialism: race_seri.predator_territorialism.max(0.0),
                        pack_size_min: pack_min,
                        pack_size_max: pack_max,
                        do_not_hunt_tags: common::common_tag_components::TagSet::new(&race_seri.predator_dont_hunt),
                        prey_body_size_ratio_tolerance: race_seri.predator_prey_body_size_ratio_tolerance,
                    },
                    PredatorHuntThreshold(race_seri.predator_hunt_threshold),
                ));
            }
            if !race_seri.wander.is_disabled() {
                let wander_cfg = &race_seri.wander;
                cmd.entity(entity).insert(WanderConfig {
                    dir_secs_min: wander_cfg.dir_secs_min.max(0.01),
                    dir_secs_max: wander_cfg.dir_secs_max.max(wander_cfg.dir_secs_min.max(0.01)),
                    move_secs_min: wander_cfg.move_secs_min.max(0.01),
                    move_secs_max: wander_cfg.move_secs_max.max(wander_cfg.move_secs_min.max(0.01)),
                    halt_secs_min: wander_cfg.halt_secs_min.max(0.01),
                    halt_secs_max: wander_cfg.halt_secs_max.max(wander_cfg.halt_secs_min.max(0.01)),
                    speed_min: wander_cfg.speed_min.max(0.0),
                    speed_max: wander_cfg.speed_max.max(wander_cfg.speed_min.max(0.0)),
                    avoid_tile_tags: common::common_tag_components::TagSet::new(&wander_cfg.avoid),
                });
            }

            if !race_seri.sexes.is_empty() {
                let mut sex_entities_weights: Vec<(Entity, f32)> = Vec::new();
                let mut sex_size_variations = bevy::ecs::entity::EntityHashMap::default();
                for (sex_id, sex_cfg) in &race_seri.sexes {
                    match sexes_map.0.get_cloned(sex_id) {
                        Ok(sex_entity) => {
                            sex_entities_weights.push((sex_entity, sex_cfg.weight() as f32));
                            if let Some(size_var) = sex_cfg.size_variation() {
                                sex_size_variations.insert(sex_entity, SpriteGlobalNormalDist::new(size_var));
                            }

                            let mut resolved_entities = if sex_cfg.sprites().is_empty() {
                                fallback_sprite_entities.clone()
                            } else {
                                resolve_sprite_entities(sex_cfg.sprites())
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
            }
            if race_seri.scale_hp_and_strength_with_size {
                cmd.entity(entity).insert(ScaleHpAndStrengthWithSize);
            }

            cmd.entity(entity)
                .insert(MappedSpritesToSample(mapped_sprites_to_sample));

            trace!(target: "race_init", "Initialized race '{}' with entity {:?}", str_id, entity);
    }
}
