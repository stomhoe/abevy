use being_shared::MappedSpritesToSample;
#[allow(unused_imports)] use bevy::prelude::*;
use game_common::game_common_samplers::{EntityWeightedSampler, SpriteGlobalNormalDist, SpriteHoriNormalDist, SpriteVertNormalDist};
use crate::being_inst_template::being_inst_template_resources::BitRef;

use crate::body::body_sampler::body_sampler_components::BodyWeightedSampler;
use crate::body::body_sampler::body_sampler_resources::BodyWeightedSamplerRef;
use crate::body::BodyTreeRef;
use crate::sex::sex_resources::SexRef;
use sprite_shared::SampleSpriteEnts;

use crate::being_components::{Being, };
use crate::race::race_components::{Race, SexesSampler};
use crate::race::race_resources::RaceRef;


#[allow(unused_parens)]
pub fn build_beings_from_race_ref(
    mut cmd: Commands,
    beings_query: Query<(
        Entity,
        &RaceRef,
        Has<SampleSpriteEnts>,
        Has<BodyTreeRef>,
        Has<SexRef>,
    ), (Changed<RaceRef>, With<Being>)>,

    race_query: Query<(// use thread rng to sample these
        Option<&SexesSampler>,
        Option<&MappedSpritesToSample>,
        Option<&BodyWeightedSamplerRef>,
        Option<&BodyTreeRef>,
    ), With<Race>>,
) {
    if beings_query.is_empty() {
        return;
    }
    let mut rng = rand::rng();
    let mut sample_sprites_to_ins: Vec<(Entity, SampleSpriteEnts)> = Vec::new();
    let mut sex_refs_to_ins: Vec<(Entity, SexRef)> = Vec::new();

    for (being_ent, race_ref, has_sample_sprites, has_body_tree_ref, has_sex_ref, ) in beings_query.iter() {
        let Ok((sexes_sampler, mapped_sprites, body_weighted_sampler_ref, body_tree_ref)) = race_query.get(race_ref.0) else {
            warn!(target: "race_build", "RaceRef entity {:?} could not be resolved to a Race entity", race_ref.0);
            continue;
        };

        // Template/body-tree data has priority. If a body is already fixed, don't set sampler ref.
        if !has_body_tree_ref {
            if let Some(&body_tree_ref) = body_tree_ref {
                cmd.entity(being_ent).try_insert_if_new(body_tree_ref);
            }
            else if let Some(&body_sampler_ref) = body_weighted_sampler_ref {
                cmd.entity(being_ent).try_insert(body_sampler_ref);
            }
        }

        let mut selected_sex_ent = None;
        if !has_sex_ref {
            if let Some(sexes_sampler) = sexes_sampler {
                selected_sex_ent = sexes_sampler.0.sample_with_rng(&mut rng);
                if let Some(sex_ent) = selected_sex_ent {
                    sex_refs_to_ins.push((being_ent, SexRef(sex_ent)));
                }
            }
        }

        if !has_sample_sprites {
            if let Some(mapped_sprites) = mapped_sprites {
                if !mapped_sprites.0.is_empty() {
                    let selected_sex_ent = selected_sex_ent
                        .or_else(|| mapped_sprites.0.keys().next().copied());

                    if let Some(sex_ent) = selected_sex_ent {
                        if let Some(sample) = mapped_sprites.0.get(&sex_ent) {
                            sample_sprites_to_ins.push((being_ent, sample.clone()));
                        } else {
                            warn!(target: "race_build", "Race entity {:?} has no sprite mapping for sex entity {:?}", race_ref.0, sex_ent);
                        }
                    } else {
                        warn!(target: "race_build", "Race entity {:?} has no selectable sex for sprite sampling", race_ref.0);
                    }
                }
            }
        }
    }

    cmd.try_insert_batch_if_new(sample_sprites_to_ins);
    cmd.try_insert_batch_if_new(sex_refs_to_ins);
}

#[allow(unused_parens)]
pub fn sample_sprite_normal_variations(
    mut cmd: Commands,
    beings_to_sample: Query<
        (Entity, Option<&BitRef>, Option<&RaceRef>),
        (Or<(Changed<BitRef>, Changed<RaceRef>)>, With<Being>),
    >,
    dists_query: Query<(
        Option<&SpriteGlobalNormalDist>,
        Option<&SpriteHoriNormalDist>,
        Option<&SpriteVertNormalDist>,
    )>,
) {
    let mut rng = rand::rng();
    let mut global_dist_results = Vec::new();
    let mut hori_dist_results = Vec::new();
    let mut vert_dist_results = Vec::new();

    for (being_ent, bit_ref, race_ref, ) in beings_to_sample.iter() {
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
