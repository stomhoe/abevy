use ::being_shared::*;
use bevy::{
    ecs::{entity::{EntityHashMap, EntityHashSet}, entity_disabling::Disabled, system::SystemParam},
    prelude::*,
};
#[allow(unused_imports, )]
use common::{AnyDisabling, common_components::{SampleSpriteEnts, StrId}, common_tag_components::TagSet, log_targets::{BEING_TEMPLATE_BUILD, BEING_SYSTEM}};
use faction::faction_resources::FactionRef;
use game_common::game_common_components::TemplEntiRef;
use game_common::game_common_timers::Templ;
use ::sprite_shared::*;
use ::tilemap_shared::*;

use crate::{
    body::{BodyTreeRef, body_sampler::body_sampler_resources::BodyWeightedSamplerRef},
    sex::sex_resources::SexRef,
};

#[derive(SystemParam)]
pub struct BuildBeingsFromRefsQueryParams<'w, 's> {
    changed_beings: Query<'w, 's, Entity, (Or<(Changed<BitRef>, Changed<RaceRef>, Changed<TemplEntiRef>)>, With<Being>, Without<Templ>, Without<BeingInstTemplate>, AnyDisabling)>,
    templ_ref_query: Query<'w, 's, &'static TemplEntiRef>,
    bit_query: Query<'w, 's, (&'static BeingInstTemplate,)>,
    race_query: Query<'w, 's, (), With<Race>>,
    scs_to_build_query: Query<'w, 's, (), With<ScsToBuild>>,
    mapped_sprites_to_sample_query: Query<'w, 's, &'static SexMappedSpritesToSample>,
    sexes_sampler_query: Query<'w, 's, &'static SexesSampler>,
    sample_sprite_ents_query: Query<'w, 's, &'static SampleSpriteEnts>,
    body_weighted_sampler_query: Query<'w, 's, &'static BodyWeightedSamplerRef>,
    body_tree_ref_query: Query<'w, 's, &'static BodyTreeRef>,
    race_ref_query: Query<'w, 's, &'static RaceRef>,
    bit_ref_query: Query<'w, 's, &'static BitRef>,
    faction_ref_query: Query<'w, 's, &'static FactionRef>,
    predator_cfg_query: Query<'w, 's, (), With<PredatorCfg>>,
    wander_cfg_query: Query<'w, 's, &'static WanderConfig>,
    avoid_blacklisted_spawn_tiles_query: Query<'w, 's, (), With<AvoidBlacklistedSpawnTilesForWander>>,
}

#[derive(SystemParam)]
pub struct BuildBeingsFromRefsRemovedParams<'w, 's> {
    removed_disabled: RemovedComponents<'w, 's, Disabled>,
    removed_bit_ref: RemovedComponents<'w, 's, BitRef>,
    removed_race_ref: RemovedComponents<'w, 's, RaceRef>,
}

#[derive(SystemParam)]
pub struct BuildBeingsFromRefsLocalParams<'s> {
    prev_refs_by_ent: Local<'s, EntityHashMap<(Option<BitRef>, Option<RaceRef>)>>,
    beings_to_process: Local<'s, EntityHashSet>,
}

#[allow(unused_parens, )]
pub fn build_beings_from_refs(
    mut cmd: Commands,
    queries: BuildBeingsFromRefsQueryParams,
    mut removed: BuildBeingsFromRefsRemovedParams,
    mut locals: BuildBeingsFromRefsLocalParams,
) {
    let mut sample_sprites_to_ins = Vec::new();
    let mut body_sampler_to_ins = Vec::new();
    let mut body_tree_refs_to_ins = Vec::new();
    let mut faction_refs_to_ins = Vec::new();
    let mut sex_refs_to_ins = Vec::new();

    locals.beings_to_process.extend(removed.removed_bit_ref.read());
    locals.beings_to_process.extend(removed.removed_race_ref.read());
    locals.beings_to_process.extend(removed.removed_disabled.read());
    locals.beings_to_process.extend(queries.changed_beings.iter());

    let mut rng = rand::rng();

    for being_ent in locals.beings_to_process.drain() {
        cmd.entity(being_ent).try_insert_if_new(Being);

        let mut bit_ref = queries.bit_ref_query.get(being_ent).ok().copied();
        let mut race_ref = queries.race_ref_query.get(being_ent).ok().copied();
        if let Ok(&TemplEntiRef(templ_ent)) = queries.templ_ref_query.get(being_ent) {
            if queries.bit_query.get(templ_ent).is_ok() {
                let templ_bit_ref = BitRef(templ_ent);
                if bit_ref != Some(templ_bit_ref) {
                    cmd.entity(being_ent).insert(templ_bit_ref);
                    debug!(target: BEING_TEMPLATE_BUILD, "Resolved TemplEntiRef {:?} for being {:?} as BitRef", templ_ent, being_ent);
                }
                bit_ref = Some(templ_bit_ref);
            } else if queries.race_query.get(templ_ent).is_ok() {
                let templ_race_ref = RaceRef(templ_ent);
                if race_ref != Some(templ_race_ref) {
                    cmd.entity(being_ent).insert(templ_race_ref);
                    debug!(target: BEING_TEMPLATE_BUILD, "Resolved TemplEntiRef {:?} for being {:?} as RaceRef", templ_ent, being_ent);
                }
                race_ref = Some(templ_race_ref);
            }
        }

        let mut race_ref = bit_ref
            .and_then(|bit_ref| queries.race_ref_query.get(bit_ref.0).ok().copied())
            .or(race_ref);
        let current_refs = (bit_ref, race_ref);

        let is_predator_now = bit_ref.is_some_and(|bit_ref| queries.predator_cfg_query.get(bit_ref.0).is_ok())
            || race_ref.is_some_and(|race_ref| queries.predator_cfg_query.get(race_ref.0).is_ok());

        if is_predator_now {
            cmd.entity(being_ent).try_insert_if_new(Predator);
        } else {
            cmd.entity(being_ent).try_remove::<Predator>();
        }
        let has_avoid_blacklisted_spawn_tiles = if let Some(bit_ref) = bit_ref {
            if let Ok(bit_wander_cfg) = queries.wander_cfg_query.get(bit_ref.0) {
                bit_wander_cfg.avoid_blacklisted_spawn_tiles
            } else {
                race_ref
                    .and_then(|race_ref| queries.wander_cfg_query.get(race_ref.0).ok())
                    .map(|cfg| cfg.avoid_blacklisted_spawn_tiles)
                    .unwrap_or(false)
            }
        } else {
            race_ref
                .and_then(|race_ref| queries.wander_cfg_query.get(race_ref.0).ok())
                .map(|cfg| cfg.avoid_blacklisted_spawn_tiles)
                .unwrap_or(false)
        };
        let had_avoid_blacklisted_spawn_tiles = queries
            .avoid_blacklisted_spawn_tiles_query
            .get(being_ent)
            .is_ok();
        if has_avoid_blacklisted_spawn_tiles {
            cmd.entity(being_ent).try_insert_if_new(AvoidBlacklistedSpawnTilesForWander);
        } else {
            cmd.entity(being_ent).try_remove::<AvoidBlacklistedSpawnTilesForWander>();
        }
        if had_avoid_blacklisted_spawn_tiles != has_avoid_blacklisted_spawn_tiles {
            debug!(
                target: BEING_TEMPLATE_BUILD,
                "Being {:?} avoid_blacklisted_spawn_tiles={}",
                being_ent,
                has_avoid_blacklisted_spawn_tiles,
            );
        }
        match locals.prev_refs_by_ent.get(&being_ent).copied() {
            Some(prev_refs) if prev_refs == current_refs => continue,
            Some(prev_refs) => {
                if current_refs == (None, None) {
                    locals.prev_refs_by_ent.remove(&being_ent);
                } else {
                    locals.prev_refs_by_ent.insert(being_ent, current_refs);
                }
                debug!(
                    target: BEING_TEMPLATE_BUILD,
                    "Rebuilding being {:?}: bit_ref {:?}->{:?} race_ref {:?}->{:?}",
                    being_ent,
                    prev_refs.0,
                    bit_ref,
                    prev_refs.1,
                    race_ref,
                );
            }
            None => {
                if current_refs == (None, None) {
                    continue;
                }
                locals.prev_refs_by_ent.insert(being_ent, current_refs);

            }
        }

        let mut has_sample_sprites_now = queries.sample_sprite_ents_query.get(being_ent).is_ok() || queries.scs_to_build_query.get(being_ent).is_ok();
        let mut has_body_tree_ref_now = queries.body_weighted_sampler_query.get(being_ent).is_ok() || queries.body_tree_ref_query.get(being_ent).is_ok();

        if let Some(bit_ref) = bit_ref {
            let Ok((template, )) = queries.bit_query.get(bit_ref.0) else {
                warn!(target: BEING_TEMPLATE_BUILD, "BitRef entity {:?} could not be resolved to BeingInstTemplate", bit_ref.0);
                continue;
            };
            if !has_sample_sprites_now && let Ok(sample_sprites) = queries.sample_sprite_ents_query.get(bit_ref.0) {
                sample_sprites_to_ins.push((being_ent, sample_sprites.clone()));
                has_sample_sprites_now = true;
            }
            if !has_body_tree_ref_now {
                if let Ok(&sample_body_body_tree) = queries.body_weighted_sampler_query.get(bit_ref.0) {
                    body_sampler_to_ins.push((being_ent, sample_body_body_tree));
                    has_body_tree_ref_now = true;
                } else if let Ok(&body_tree_ref) = queries.body_tree_ref_query.get(bit_ref.0) {
                    body_tree_refs_to_ins.push((being_ent, body_tree_ref));
                    has_body_tree_ref_now = true;
                }
            }
            if let Ok(&faction_ref) = queries.faction_ref_query.get(bit_ref.0) {
                faction_refs_to_ins.push((being_ent, faction_ref));
            }

            if let Ok(&race_ref_from_bit) = queries.race_ref_query.get(bit_ref.0) {
                if race_ref != Some(race_ref_from_bit) {
                    cmd.entity(being_ent).insert(race_ref_from_bit);
                    debug!(target: BEING_TEMPLATE_BUILD, "Resolved bit {:?} for being {:?} to RaceRef {:?}", bit_ref.0, being_ent, race_ref_from_bit.0);
                }
                race_ref = Some(race_ref_from_bit);
            }
            if template.extra_health_multiplier != 1.0 {
                // add in a modifier
            }
        }

        if let Some(race_ref) = race_ref {
            if !has_body_tree_ref_now {
                if let Ok(&sample_body_body_tree) = queries.body_weighted_sampler_query.get(race_ref.0) {
                    body_sampler_to_ins.push((being_ent, sample_body_body_tree));
                    has_body_tree_ref_now = true;
                } else if let Ok(&body_tree_ref) = queries.body_tree_ref_query.get(race_ref.0) {
                    body_tree_refs_to_ins.push((being_ent, body_tree_ref));
                    has_body_tree_ref_now = true;
                }
            }

            let mut selected_sex_ent = None;
            if let Ok(sexes_sampler) = queries.sexes_sampler_query.get(race_ref.0) {
                selected_sex_ent = sexes_sampler.0.sample_with_rng(&mut rng);
                if let Some(sex_ent) = selected_sex_ent {
                    sex_refs_to_ins.push((being_ent, SexRef(sex_ent)));
                }
            }
            if !has_sample_sprites_now && let Ok(mapped_sprites) = queries.mapped_sprites_to_sample_query.get(race_ref.0) {
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
    cmd.try_insert_batch(sample_sprites_to_ins);
    cmd.try_insert_batch(body_sampler_to_ins);
    cmd.try_insert_batch(body_tree_refs_to_ins);
    cmd.try_insert_batch_if_new(faction_refs_to_ins);
    cmd.try_insert_batch_if_new(sex_refs_to_ins);
}

#[derive(SystemParam)]
pub struct SampleSpriteNormalSizeVariationsQueryParams<'w, 's> {
    changed_beings: Query<'w, 's, Entity, (Or<(Changed<BitRef>, Changed<RaceRef>)>, With<Being>)>,
    beings_to_sample: Query<'w, 's, (Option<&'static BitRef>, Option<&'static RaceRef>, Option<&'static SexRef>, Has<SpriteGlobalNormalDistResult>, Has<SpriteHoriNormalDistResult>, Has<SpriteVertNormalDistResult>), (With<Being>, AnyDisabling)>,
    race_sex_size_dists: Query<'w, 's, &'static SexSizeVariationsBySex, With<Race>>,
    dists_query: Query<'w, 's, (Option<&'static SpriteGlobalNormalDist>, Option<&'static SpriteHoriNormalDist>, Option<&'static SpriteVertNormalDist>)>,
}

#[derive(SystemParam)]
pub struct SampleSpriteNormalSizeVariationsRemovedParams<'w, 's> {
    removed_disabled: RemovedComponents<'w, 's, Disabled>,
}

#[derive(SystemParam)]
pub struct SampleSpriteNormalSizeVariationsLocalParams<'s> {
    global_dist_results: Local<'s, Vec<(Entity, SpriteGlobalNormalDistResult)>>,
    hori_dist_results: Local<'s, Vec<(Entity, SpriteHoriNormalDistResult)>>,
    vert_dist_results: Local<'s, Vec<(Entity, SpriteVertNormalDistResult)>>,
    beings_to_process: Local<'s, Vec<Entity>>,
}

#[allow(unused_parens)]
pub fn sample_sprite_normal_size_variations(
    mut cmd: Commands,
    queries: SampleSpriteNormalSizeVariationsQueryParams,
    mut removed: SampleSpriteNormalSizeVariationsRemovedParams,
    mut locals: SampleSpriteNormalSizeVariationsLocalParams,
) {
    locals.global_dist_results.clear();
    locals.hori_dist_results.clear();
    locals.vert_dist_results.clear();

    locals.beings_to_process.extend(removed.removed_disabled.read());
    locals.beings_to_process.extend(queries.changed_beings.iter());
    if locals.beings_to_process.is_empty() {
        return;
    }

    let mut rng = rand::rng();
    let mut global_dist_results = Vec::new();
    let mut hori_dist_results = Vec::new();
    let mut vert_dist_results = Vec::new();

    for being_ent in locals.beings_to_process.drain(..) {
        let Ok((bit_ref, race_ref, sex_ref, has_global_result, has_hori_result, has_vert_result)) = queries.beings_to_sample.get(being_ent) else {
            continue;
        };
        let is_reenabled_only = queries.changed_beings.get(being_ent).is_err();
        if is_reenabled_only && has_global_result && has_hori_result && has_vert_result {
            continue;
        }
        let mut global_dist: Option<&SpriteGlobalNormalDist> = None;
        let mut hori_dist: Option<&SpriteHoriNormalDist> = None;
        let mut vert_dist: Option<&SpriteVertNormalDist> = None;

        if let Some(bit_ref) = bit_ref {
            if let Ok((bit_global, bit_hori, bit_vert, )) = queries.dists_query.get(bit_ref.0) {
                if bit_global.is_some() { global_dist = bit_global; }
                if bit_hori.is_some() { hori_dist = bit_hori; }
                if bit_vert.is_some() { vert_dist = bit_vert; }
            }
        }

        if let Some(race_ref) = race_ref {
            if let Some(sex_ref) = sex_ref {
                if let Ok(sex_dists) = queries.race_sex_size_dists.get(race_ref.0) {
                    if let Some(dist) = sex_dists.0.get(&sex_ref.0) {
                        global_dist = Some(dist);
                    }
                }
            }
            if let Ok((race_global, race_hori, race_vert, )) = queries.dists_query.get(race_ref.0) {
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
