use ::being_shared::*;
use bevy::{
    ecs::{entity::EntityHashSet, entity_disabling::Disabled},
    prelude::*,
};
#[allow(unused_imports, )]
use common::{AnyDisabling, common_components::{SampleSpriteEnts, StrId}, common_tag_components::TagSet, log_targets::{BEING_TEMPLATE_BUILD, BEING_SYSTEM}};
use faction::faction_resources::FactionRef;
use game_common::game_common_timers::TemplEnti;
use ::tilemap_shared::*;

use crate::{
    being_inst_template::being_inst_template_resources::{BitRef},
    body::{BodyTreeRef, body_sampler::body_sampler_resources::BodyWeightedSamplerRef},
    race::race_components::{Race, SexesSampler, SexSizeVariationsBySex},
    race::race_resources::RaceRef,
    sex::sex_resources::SexRef,
};

pub fn build_beings_from_refs(
    mut cmd: Commands,
    changed_beings: Query<
        Entity,
        (
            Without<TemplEnti>,
            Without<BeingInstTemplate>,
            Or<(Changed<BitRef>, Changed<RaceRef>)>,
        ),
    >,
    mut customer_query: Query<
        (
            Option<&BitRef>,
            Option<&RaceRef>,
            Option<&mut JoinedGroups>,
            Has<SampleSpriteEnts>,
            Has<BodyTreeRef>,
            Has<BodyWeightedSamplerRef>,
            Has<SexRef>,
            Has<InteractionZones>,
        ),
        (Without<TemplEnti>, Without<BeingInstTemplate>, AnyDisabling),
    >,
    bit_query: Query<(
        &BeingInstTemplate,
        Option<&SampleSpriteEnts>,
        Option<&RaceRef>,
        Option<&BodyWeightedSamplerRef>,
        Option<&FactionRef>,
        Option<&CappedNormalDist>,
    )>,
    race_query: Query<(
        Option<&SexesSampler>,
        Option<&MappedSpritesToSample>,
        Option<&BodyWeightedSamplerRef>,
        Option<&BodyTreeRef>,
    ), With<Race>>,
    zone_sources: Query<&InteractionZones>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut beings_to_build: Local<EntityHashSet>,
) {
    let mut sample_sprites_to_ins = Vec::new();
    let mut race_refs_to_ins = Vec::new();
    let mut body_sampler_to_ins = Vec::new();
    let mut member_of_to_ins = Vec::new();
    let mut sex_refs_to_ins = Vec::new();

    beings_to_build.clear();
    beings_to_build.extend(removed_disabled.read());
    beings_to_build.extend(changed_beings.iter());

    let mut rng = rand::rng();

    for being_ent in beings_to_build.drain() {
        let Ok((bit_ref, race_ref, member_of, has_sample_sprites, has_body_tree_ref, has_body_sampler_ref, has_sex_ref, has_interaction_zones, )) = customer_query.get_mut(being_ent) else {
            continue;
        };
        let is_reenabled_only = changed_beings.get(being_ent).is_err();
        if is_reenabled_only
            && has_sample_sprites
            && (has_body_tree_ref || has_body_sampler_ref)
            && has_sex_ref
            && has_interaction_zones
        {
            continue;
        }
        cmd.entity(being_ent).try_insert_if_new(Being);

        let mut effective_race_ref = race_ref.copied();
        let mut has_sample_sprites_now = has_sample_sprites;
        let mut has_body_tree_ref_now = has_body_tree_ref || has_body_sampler_ref;

        if let Some(bit_ref) = bit_ref {
            let Ok((template, sample_sprites, bit_race_ref, sample_body_body_tree, faction_ref, _norm_dist)) = bit_query.get(bit_ref.0) else {
                warn!(target: BEING_TEMPLATE_BUILD, "BitRef entity {:?} could not be resolved to BeingInstTemplate", bit_ref.0);
                continue;
            };
            if !has_sample_sprites_now && let Some(sample_sprites) = sample_sprites {
                let sample_sprites = sample_sprites.clone();
                sample_sprites_to_ins.push((being_ent, sample_sprites));
                has_sample_sprites_now = true;
            }
            if let Some(sample_body_body_tree) = sample_body_body_tree {
                body_sampler_to_ins.push((being_ent, *sample_body_body_tree));
                has_body_tree_ref_now = true;
            }
            if let Some(faction_ref) = faction_ref {
                if let Some(mut member_of) = member_of {
                    member_of.insert(faction_ref.0);
                } else {
                    member_of_to_ins.push((being_ent, JoinedGroups::single(faction_ref.0)));
                }
            }
            if let Some(bit_race_ref) = bit_race_ref {
                if race_ref.map(|r| r.0) != Some(bit_race_ref.0) {
                    race_refs_to_ins.push((being_ent, *bit_race_ref));
                }
                effective_race_ref = Some(*bit_race_ref);
            }
            if template.extra_health_multiplier != 1.0 {
                // add in a modifier
            }
        }
        let mut inserted_interaction_zones = false;
        if let Some(bit_ref) = bit_ref {
            if let Ok(zones) = zone_sources.get(bit_ref.0) {
                cmd.entity(being_ent).insert(zones.clone());
                inserted_interaction_zones = true;
            }
        }
        if !inserted_interaction_zones && !has_interaction_zones {
            if let Some(race_ref) = effective_race_ref {
                if let Ok(zones) = zone_sources.get(race_ref.0) {
                    cmd.entity(being_ent).insert(zones.clone());
                }
            }
        }

        let mut selected_sex_ent = None;
        if let Some(race_ref) = effective_race_ref {
            let Ok((sexes_sampler, mapped_sprites, body_weighted_sampler_ref, body_tree_ref)) = race_query.get(race_ref.0) else {
                warn!(target: BEING_SYSTEM, "RaceRef entity {:?} could not be resolved to a Race entity", race_ref.0);
                continue;
            };

            if !has_body_tree_ref_now {
                if let Some(&body_tree_ref) = body_tree_ref {
                    cmd.entity(being_ent).try_insert_if_new(body_tree_ref);
                } else if let Some(&body_sampler_ref) = body_weighted_sampler_ref {
                    cmd.entity(being_ent).try_insert(body_sampler_ref);
                }
            }

            if !has_sex_ref {
                if let Some(sexes_sampler) = sexes_sampler {
                    selected_sex_ent = sexes_sampler.0.sample_with_rng(&mut rng);
                    if let Some(sex_ent) = selected_sex_ent {
                        sex_refs_to_ins.push((being_ent, SexRef(sex_ent)));
                    }
                }
            }

            if !has_sample_sprites_now {
                if let Some(mapped_sprites) = mapped_sprites {
                    if !mapped_sprites.0.is_empty() {
                        let selected_sex_ent = selected_sex_ent.or_else(|| mapped_sprites.0.keys().next().copied());
                        let Some(sex_ent) = selected_sex_ent else {
                            warn!(target: BEING_SYSTEM, "Race entity {:?} has no selectable sex for sprite sampling", race_ref.0);
                            continue;
                        };
                        let Some(sample) = mapped_sprites.0.get(&sex_ent) else {
                            warn!(target: BEING_SYSTEM, "Race entity {:?} has no sprite mapping for sex entity {:?}", race_ref.0, sex_ent);
                            continue;
                        };
                        sample_sprites_to_ins.push((being_ent, sample.clone()));
                    }
                }
            }
        }
    }
    cmd.try_insert_batch(sample_sprites_to_ins);
    cmd.try_insert_batch(race_refs_to_ins);
    cmd.try_insert_batch(body_sampler_to_ins);
    cmd.try_insert_batch_if_new(member_of_to_ins);
    cmd.try_insert_batch_if_new(sex_refs_to_ins);
}

#[allow(unused_parens)]
pub fn sample_sprite_normal_size_variations(
    mut cmd: Commands,
    changed_beings: Query<Entity, (Or<(Changed<BitRef>, Changed<RaceRef>)>, With<Being>)>,
    beings_to_sample: Query<
        (
            Option<&BitRef>,
            Option<&RaceRef>,
            Option<&SexRef>,
            Has<SpriteGlobalNormalDistResult>,
            Has<SpriteHoriNormalDistResult>,
            Has<SpriteVertNormalDistResult>,
        ),
        (With<Being>, AnyDisabling),
    >,
    race_sex_size_dists: Query<&SexSizeVariationsBySex, With<Race>>,
    dists_query: Query<(
        Option<&SpriteGlobalNormalDist>,
        Option<&SpriteHoriNormalDist>,
        Option<&SpriteVertNormalDist>,
    )>,
    mut removed_disabled: RemovedComponents<Disabled>,
    mut global_dist_results: Local<Vec<(Entity, SpriteGlobalNormalDistResult)>>,
    mut hori_dist_results: Local<Vec<(Entity, SpriteHoriNormalDistResult)>>,
    mut vert_dist_results: Local<Vec<(Entity, SpriteVertNormalDistResult)>>,
    mut beings_to_process: Local<Vec<Entity>>,
) {
    global_dist_results.clear();
    hori_dist_results.clear();
    vert_dist_results.clear();

    beings_to_process.extend(removed_disabled.read());
    beings_to_process.extend(changed_beings.iter());
    if beings_to_process.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    let mut global_dist_results = Vec::new();
    let mut hori_dist_results = Vec::new();
    let mut vert_dist_results = Vec::new();

    for being_ent in beings_to_process.drain(..) {
        let Ok((bit_ref, race_ref, sex_ref, has_global_result, has_hori_result, has_vert_result)) = beings_to_sample.get(being_ent) else {
            continue;
        };
        let is_reenabled_only = changed_beings.get(being_ent).is_err();
        if is_reenabled_only && has_global_result && has_hori_result && has_vert_result {
            continue;
        }
        let mut global_dist: Option<&SpriteGlobalNormalDist> = None;
        let mut hori_dist: Option<&SpriteHoriNormalDist> = None;
        let mut vert_dist: Option<&SpriteVertNormalDist> = None;

        if let Some(bit_ref) = bit_ref {
            if let Ok((bit_global, bit_hori, bit_vert, )) = dists_query.get(bit_ref.0) {
                if bit_global.is_some() { global_dist = bit_global; }
                if bit_hori.is_some() { hori_dist = bit_hori; }
                if bit_vert.is_some() { vert_dist = bit_vert; }
            }
        }

        if let Some(race_ref) = race_ref {
            if let Some(sex_ref) = sex_ref {
                if let Ok(sex_dists) = race_sex_size_dists.get(race_ref.0) {
                    if let Some(dist) = sex_dists.0.get(&sex_ref.0) {
                        global_dist = Some(dist);
                    }
                }
            }
            if let Ok((race_global, race_hori, race_vert, )) = dists_query.get(race_ref.0) {
                if global_dist.is_none() { global_dist = race_global; }
                if hori_dist.is_none() { hori_dist = race_hori; }
                if vert_dist.is_none() { vert_dist = race_vert; }
            }
        }

        if let Some(global_dist) = global_dist {
            let result = global_dist.sample(&mut rng);
            global_dist_results.push((being_ent, result));
        }

        if let Some(hori_dist) = hori_dist {
            let result = hori_dist.sample(&mut rng);
            hori_dist_results.push((being_ent, result));
        }

        if let Some(vert_dist) = vert_dist {
            let result = vert_dist.sample(&mut rng);
            vert_dist_results.push((being_ent, result));
        }
    }
    cmd.try_insert_batch(global_dist_results);
    cmd.try_insert_batch(hori_dist_results);
    cmd.try_insert_batch(vert_dist_results);
}
