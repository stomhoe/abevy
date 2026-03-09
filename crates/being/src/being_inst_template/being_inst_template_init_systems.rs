use being_shared::{BeingInstTemplate, Predator, PredatorHuntThreshold};
use bevy::prelude::*;
use common::common_components::*;

use ::sprite_shared::prelude::*;
use game_common::game_common_samplers::*;

use crate::being_inst_template::{being_inst_template_resources::*,
};
use crate::race::race_resources::{RaceEntityMap, RaceRef};
use crate::body::{BodyTreeRef, body_tree_resources::BodyTreeEntityMap, body_sampler::body_sampler_resources::{BodyWeightedSamplerEntityMap, BodyWeightedSamplerRef}};
use faction::faction_resources::{FactionEntityMap, FactionStrIdRef};
use tilemap_shared::InteractionZones;
use crate::being_components::{COLLISION_MASK_HASHID, HitboxReceiver};

pub fn init_being_templates(
    mut cmd: Commands,
    race_emap: Option<Res<RaceEntityMap>>,
    faction_emap: Option<Res<FactionEntityMap>>,
    bit_map: Res<BeingInstTemplateEntityMap>,
    body_tree_map: Res<BodyTreeEntityMap>,
    body_sampler_map: Res<BodyWeightedSamplerEntityMap>,

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

    let mut main_comps = Vec::new();
    let mut samples = Vec::new();
    let mut race_refs_to_insert = Vec::new();
    let mut faction_refs_to_insert = Vec::new();

    let Some(faction_emap) = faction_emap else {
        error!("Faction entity map is missing");
        return;
    };
    let Some(race_emap) = race_emap else {
        error!("Race entity map is missing");
        return;
    };

    for template_seri in load_bit_seri_defs() {
        let str_id = StrId::trunc(&template_seri.id);

        let bit_entity = cmd.spawn_empty().id();

        let being_inst_template = BeingInstTemplate {
            points: template_seri.points,
            extra_health_multiplier: template_seri.health_multiplier.max(0.001),
        };

        main_comps.push((bit_entity, (being_inst_template, str_id.clone())));

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
        if let Some(size_variation) = template_seri.size_variation {
            cmd.entity(bit_entity).insert(SpriteGlobalNormalDist::new(size_variation));
        }
        if let Some(hori_variation) = template_seri.hori_variation {
            cmd.entity(bit_entity).insert(SpriteHoriNormalDist::new(hori_variation));
        }
        if let Some(vert_variation) = template_seri.vert_variation {
            cmd.entity(bit_entity).insert(SpriteVertNormalDist::new(vert_variation));
        }
        if PredatorHuntThreshold::is_configured_in_seri(template_seri.predator_hunt_threshold) {
            cmd.entity(bit_entity).insert((
                Predator::default(),
                PredatorHuntThreshold(template_seri.predator_hunt_threshold),
            ));
        }
        let mut interaction_zones = bevy::platform::collections::HashMap::with_capacity(1);
        interaction_zones.insert("melee".to_string(), template_seri.melee_interaction_zone.clone());
        cmd.entity(bit_entity).insert(InteractionZones::new(interaction_zones));
        let hitbox_hashid = if template_seri.hitbox_hashid.trim().is_empty() {
            COLLISION_MASK_HASHID
        } else {
            HashId::from(template_seri.hitbox_hashid.as_str())
        };
        cmd.entity(bit_entity).insert(HitboxReceiver(hitbox_hashid));

        // Resolve race entity from race string
        let race_str_id = StrId::trunc(&template_seri.race);
        match race_emap.0.get_cloned(&race_str_id) {
            Ok(race_entity) => {
                race_refs_to_insert.push((bit_entity, RaceRef(race_entity)));
            }
            Err(_) => {
                error!(target: "being_template_init", "BeingTemplate '{}' race '{}' not found in RaceEntityMap", str_id, race_str_id);
            }
        }

        if template_seri.health_multiplier < 0.0 {
            warn!(target: "being_template_init", "BeingTemplate '{}' has negative health multiplier {}, not applying", str_id, template_seri.health_multiplier);
        }
    }
    cmd.try_insert_batch(main_comps);
    cmd.try_insert_batch(samples);
    cmd.try_insert_batch(faction_refs_to_insert);
    cmd.try_insert_batch(race_refs_to_insert);
}
