use being_shared::{BeingInstTemplate, Predator, PredatorHuntThreshold};
use bevy::prelude::*;
use common::common_components::*;

use ::sprite_shared::*;
use game_common::game_common_samplers::*;

use crate::being_inst_template::{being_inst_template_resources::*,
};
use crate::race::race_resources::{RaceEntityMap, RaceRef};
use crate::body::body_tree_resources::BodyTreeEntityMap;
use faction::faction_resources::{FactionEntityMap, FactionStrIdRef};

pub fn init_being_templates(
    mut cmd: Commands,
    race_emap: Option<Res<RaceEntityMap>>,
    faction_emap: Option<Res<FactionEntityMap>>,
    bit_map: Res<BeingInstTemplateEntityMap>,

) {
    if !bit_map.0.is_empty(){
        return;
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
                Predator,
                PredatorHuntThreshold(template_seri.predator_hunt_threshold),
            ));
        }

        // Resolve race entity from race string
        let race_str_id = StrId::trunc(&template_seri.race);
        match race_emap.0.get_cloned(&race_str_id) {
            Ok(race_entity) => {
                race_refs_to_insert.push((bit_entity, RaceRef(race_entity)));
            }
            Err(_) => {
                warn!(target: "being_template_init", "BeingTemplate '{}' race '{}' not found in RaceEntityMap", str_id, race_str_id);
            }
        }

        if template_seri.health_multiplier < 0.0 {
            warn!(target: "being_template_init", "BeingTemplate '{}' has negative health multiplier {}, setting to 0.0", str_id, template_seri.health_multiplier);
        }
    }
    cmd.try_insert_batch(main_comps);
    cmd.try_insert_batch(samples);
    cmd.try_insert_batch(faction_refs_to_insert);
    cmd.try_insert_batch(race_refs_to_insert);
}
