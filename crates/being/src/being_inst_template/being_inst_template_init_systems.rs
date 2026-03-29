
use bevy::prelude::*;
use common::common_components::*;
use ::being_shared::*;

use game_common::Templ;
use ::sprite_shared::*;
use tilemap_shared::tilemap_shared_samplers::*;


use crate::pack::pack_components::PackInitialSizeSampler;
use crate::being_interaction_zone_helper::build_being_interaction_zones_with_fallback;
use crate::body::{BodyTreeRef, body_tree_resources::BodyTreeEntityMap, body_sampler::body_sampler_resources::{BodyWeightedSamplerEntityMap, BodyWeightedSamplerRef}};
use faction::faction_resources::{FactionStrIdRef};
use tilemap::terrain::biome::{biome_components::CreatureSampler, biome_resources::BiomeEntityMap};
use tilemap_shared::{BlacklistedSpawnTileTags, WhitelistedSpawnTileTags};
use common::common_tag_components::TagSet;

pub fn init_being_templates(
    mut cmd: Commands,
    race_emap: Option<Res<RaceEntityMap>>,
    bit_map: Res<BeingInstTemplateEntityMap>,
    body_tree_map: Res<BodyTreeEntityMap>,
    body_sampler_map: Res<BodyWeightedSamplerEntityMap>,
    biome_emap: Res<BiomeEntityMap>,
    mut biome_pack_samplers: Query<&mut CreatureSampler>,
) {
    if !bit_map.0.is_empty(){
        return;
    }
    if body_tree_map.0.is_empty() {
        error!(target: "being_template_init", "BodyTreeEntityMap is empty");
    }
    if body_sampler_map.0.is_empty() {
        warn!(target: "being_template_init", "BodyWeightedSamplerEntityMap is empty (may be ok if none are used)");
    }

    let mut samples = Vec::new();
    let mut race_refs_to_insert = Vec::new();
    let mut faction_refs_to_insert = Vec::new();

    let Some(race_emap) = race_emap else {
        error!("Race entity map is missing");
        return;
    };

    for template_seri in load_bit_seri_defs() {
        let str_id = StrId::trunc(&template_seri.id);


        let being_inst_template = BeingInstTemplate {
            points: template_seri.points,
            extra_health_multiplier: template_seri.health_multiplier.max(0.001),
        };
        let bit_entity = cmd.spawn((
            being_inst_template,
            str_id.clone(),
            Templ
        )).id();

        cmd.entity(bit_entity).insert(template_seri.tags_and_own_id());

        if !template_seri.scs_samplers.is_empty() {
            samples.push((bit_entity, SampleSpritesFromStrIds::new(template_seri.scs_samplers,)));
        }

        if !template_seri.fallback_faction.trim().is_empty(){
            let faction_str_id = StrId::trunc(&template_seri.fallback_faction);
            faction_refs_to_insert.push((bit_entity, FactionStrIdRef(faction_str_id)));
        }
        if !template_seri.body_tree.trim().is_empty() {
            let body_tree_str_id = StrId::trunc(&template_seri.body_tree);
            if let Ok(body_sampler_ent) = body_sampler_map.0.get_cloned(&body_tree_str_id) {
                cmd.entity(bit_entity).insert(BodyWeightedSamplerRef(body_sampler_ent));
            } else if let Ok(body_tree_ent) = body_tree_map.0.get_cloned(&body_tree_str_id) {
                cmd.entity(bit_entity).insert(BodyTreeRef(body_tree_ent));
            } else{
                error!(target: "being_template_init", "Body tree/sampler '{}' not found for BeingInstTemplate '{}'", body_tree_str_id, str_id);
            }
        }
        if let Some(size_variation) = template_seri.size_variation.filter(|v| !v.is_sentinel()) {
            cmd.entity(bit_entity).insert(SpriteGlobalNormalDist::new(size_variation));
        }
        if let Some(hori_variation) = template_seri.hori_variation.filter(|v| !v.is_sentinel()) {
            cmd.entity(bit_entity).insert(SpriteHoriNormalDist::new(hori_variation));
        }
        if let Some(vert_variation) = template_seri.vert_variation.filter(|v| !v.is_sentinel()) {
            cmd.entity(bit_entity).insert(SpriteVertNormalDist::new(vert_variation));
        }
        if let Some(spawn_pack_size_normal_dist) = template_seri.spawn_pack_size_normal_dist.filter(|v| !v.is_sentinel()) {
            cmd.entity(bit_entity).insert(PackInitialSizeSampler(CappedNormalDist::from_seri(
                spawn_pack_size_normal_dist,
            )));
        }
        if !template_seri.spawn_pack_entity {
            cmd.entity(bit_entity).insert(NoSpawnSquadEntity);
        }
        if !template_seri.wander.is_disabled() {
            cmd.entity(bit_entity).insert(WanderConfig::from_seri(&template_seri.wander));
        }
        if !template_seri.whitelisted_spawn_tile_tags.is_empty() {
            cmd.entity(bit_entity).insert(WhitelistedSpawnTileTags(tilemap_shared::being_components::WhitelistedTags(TagSet::new(&template_seri.whitelisted_spawn_tile_tags))));
        }
        if !template_seri.blacklisted_spawn_tile_tags.is_empty() || !template_seri.blacklisted_tiles_for_spawning.is_empty() {
            cmd.entity(bit_entity).insert(BlacklistedSpawnTileTags(tilemap_shared::being_components::BlacklistedTags(TagSet::new(
                template_seri
                    .blacklisted_spawn_tile_tags
                    .iter()
                    .chain(template_seri.blacklisted_tiles_for_spawning.iter()),
            ))));
        }
        if template_seri.dont_extend_from_bit_spawn_whitelist {
            cmd.entity(bit_entity).insert(DontExtendBitSpawnWhitelist);
        }
        if template_seri.dont_extend_from_bit_spawn_blacklist {
            cmd.entity(bit_entity).insert(DontExtendBitSpawnBlacklist);
        }
        if template_seri.dont_extend_from_race_spawn_whitelist {
            cmd.entity(bit_entity).insert(DontExtendRaceSpawnWhitelist);
        }
        if template_seri.dont_extend_from_race_spawn_blacklist {
            cmd.entity(bit_entity).insert(DontExtendRaceSpawnBlacklist);
        }
        if let Some(predator_cfg) = PredatorCfg::from_seri(&template_seri.predator) {
            cmd.entity(bit_entity).insert(predator_cfg);
        }
        if DetectionVisionCone::is_configured_in_seri(
            template_seri.detection_vision_cone_range_tiles,
            template_seri.detection_vision_cone_half_angle_deg,
        ) {
            cmd.entity(bit_entity).insert(DetectionVisionCone {
                range_tiles: template_seri.detection_vision_cone_range_tiles.max(0.0),
                half_angle_deg: template_seri.detection_vision_cone_half_angle_deg.clamp(1.0, 179.0),
            });
        }
        // Resolve race entity from race string
        let race_str_id = StrId::trunc(&template_seri.race);
        let Ok(race_entity) = race_emap.0.get_cloned(&template_seri.race) else {
            error!(target: "being_template_init", "BeingTemplate '{}' race '{}' not found in RaceEntityMap", str_id, race_str_id);
            continue;
        };
        race_refs_to_insert.push((bit_entity, RaceRef(race_entity)));
        cmd.entity(bit_entity).insert(build_being_interaction_zones_with_fallback(
            None,
            template_seri.melee_attack_zone.clone(),
            template_seri.collision_zone.clone(),
        ));

        if template_seri.health_multiplier < 0.0 {
            warn!(target: "being_template_init", "BeingTemplate '{}' has negative health multiplier {}, not applying", str_id, template_seri.health_multiplier);
        }

        for (biome_tag, weight) in &template_seri.biome_affinity {
            if *weight == 0.0 {
                continue;
            }
            let Ok(biome_ent) = biome_emap.0.get_cloned(biome_tag) else {
                continue;
            };
            let Ok(mut biome_pack_sampler) = biome_pack_samplers.get_mut(biome_ent) else {
                continue;
            };
            biome_pack_sampler.add_affinity(bit_entity, *weight);
        }
    }
    cmd.try_insert_batch(samples);
    cmd.try_insert_batch(faction_refs_to_insert);
    cmd.try_insert_batch(race_refs_to_insert);
}
