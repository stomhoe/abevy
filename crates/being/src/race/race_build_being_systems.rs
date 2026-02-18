#[allow(unused_imports)] use bevy::prelude::*;
use common::common_id_components::HashId;
use ::tilemap_shared::*;
use game_common::game_common_samplers::{ScaleHpAndStrengthWithSize, SpriteGlobalNormalDist, SpriteHoriNormalDist, SpriteVertNormalDist};
use crate::being_inst_template::being_inst_template_resources::BitRef;

use crate::body::BodyTreeStrIdRef;
use sprite_shared::SampleSpriteEnts;

use crate::being_components::{Being, MappedSpritesToSample};
use crate::body::body_sampler::body_sampler_components::{SampleBodyFromStrId, SampleTreeEnt};
use crate::race::race_components::{Race, SexesSampler};
use crate::race::race_resources::RaceRef;


#[allow(unused_parens)]
pub fn build_beings_from_race_ref(
    mut cmd: Commands,
    global_gen_settings: Query<&GlobalGenSettings>,
    dimension_hash_query: Query<&HashId, common::AnyDisabling>,
    beings_query: Query<(
        Entity,
        &RaceRef,
        AnyOf<(&GlobalTilePos, &Transform)>,
        &DimensionRef,
        Option<&SampleSpriteEnts>,
        Option<&SampleTreeEnt>,
        Option<&SampleBodyFromStrId>,
    ), (Changed<RaceRef>, With<Being>)>,
    race_query: Query<(
        Option<&SexesSampler>,
        Option<&MappedSpritesToSample>,
        Option<&BodyTreeStrIdRef>,
    ), With<Race>>,
) {
    if beings_query.is_empty() {
        return;
    }
    let Ok(global_gen_settings) = global_gen_settings.single() else {
        error!("Failed to get global gen settings");
        return;
    };
    let mut sample_sprites_to_ins: Vec<(Entity, SampleSpriteEnts)> = Vec::new();
    let mut sample_bodies_to_ins: Vec<(Entity, SampleBodyFromStrId)> = Vec::new();

    for (ent, race_ref, (gpos, transform), dimension_ref, sample_sprites, sample_tree, sample_body_strid) in beings_query.iter() {
        let Ok((sexes_sampler, mapped_sprites, body_tree_str_id_ref)) = race_query.get(race_ref.0) else {
            warn!(target: "race_build", "RaceRef entity {:?} could not be resolved to a Race entity", race_ref.0);
            continue;
        };

        if sample_tree.is_none() && sample_body_strid.is_none() {
            if let Some(body_tree_str_id_ref) = body_tree_str_id_ref {
                sample_bodies_to_ins.push((
                    ent,
                    SampleBodyFromStrId::new(body_tree_str_id_ref.0.as_ref()),
                ));
            }
        }

        if sample_sprites.is_none() {
            if let Some(mapped_sprites) = mapped_sprites {
                if !mapped_sprites.0.is_empty() {
                    let pos = if let Some(gpos) = gpos {
                        Some(*gpos)
                    } else if let Some(transform) = transform {
                        Some(GlobalTilePos::from(transform.translation.xy()))
                    } else {
                        None
                    };

                    let dim_hash = dimension_hash_query.get(dimension_ref.0).copied().ok();

                    let sampled_sex_ent = match (sexes_sampler, pos, dim_hash) {
                        (Some(sexes_sampler), Some(pos), Some(dim_hash)) => {
                            sexes_sampler.0.sample_with_pos(pos, &global_gen_settings, dim_hash)
                        }
                        _ => None,
                    };

                    let selected_sex_ent = sampled_sex_ent
                        .or_else(|| mapped_sprites.0.keys().next().copied());

                    if let Some(sex_ent) = selected_sex_ent {
                        if let Some(sample) = mapped_sprites.0.get(&sex_ent) {
                            sample_sprites_to_ins.push((ent, sample.clone()));
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
    cmd.try_insert_batch_if_new(sample_bodies_to_ins);
}

#[allow(unused_parens)]
pub fn sample_sprite_variations(
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
