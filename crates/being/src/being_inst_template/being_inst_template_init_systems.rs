use being_shared::{BeingInstTemplate, };
use bevy::ecs::error;
use bevy::prelude::*;
use common::common_components::*;
use sprite::{
    sprite_components::ScrsToBuild, sprite_resources::SpriteConfigEntityMap,
};
use ::sprite_shared::*;

use crate::being_inst_template::{
    being_inst_template_components::*, being_inst_template_resources::*,
};
use crate::race::race_resources::{RaceEntityMap, RaceRef};
use crate::body::body_tree_resources::BodyTreeEntityMap;
use faction::faction_resources::{FactionEntityMap, FactionStrIdRef};
use faction::faction_components::BelongsToFaction;

pub fn init_being_templates(
    mut cmd: Commands,
    mut seris_handles: ResMut<BitSerisHandles>,
    mut assets: ResMut<Assets<BitSerialization>>,
    sc_emap: Res<SpriteConfigEntityMap>,
    race_emap: Option<Res<RaceEntityMap>>,
    faction_emap: Option<Res<FactionEntityMap>>,
    bit_map: Res<BeingInstTemplateEntityMap>,

) {
    if !bit_map.0.is_empty(){
        return;
    }

    use std::mem::take;
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

    error!("BeingInstTemplateEntityMap runninnggg");

    for handle in take(&mut seris_handles.handles) {
        if let Some(template_seri) = assets.remove(handle.id()) {
            let str_id = StrId::trunc(&template_seri.id);

            let bit_entity = cmd.spawn_empty().id();

            let being_inst_template = BeingInstTemplate {
                points: template_seri.points,
                extra_health_multiplier: template_seri.health_multiplier.unwrap_or(1.).max(0.01),
            };

            main_comps.push((bit_entity, (being_inst_template, str_id.clone())));

            if let Some(sprites_weight_maps) = template_seri.scs_samplers {
                samples.push((bit_entity, SampleSpritesFromStrIds::new(sprites_weight_maps,)));

            }

            if let Some(faction_str_id) = template_seri.fallback_faction {
                if ! faction_str_id.trim().is_empty(){
                    let faction_str_id = StrId::trunc(&faction_str_id);

                    faction_refs_to_insert.push((bit_entity, FactionStrIdRef(faction_str_id)));
                }
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

            if let Some(health_multiplier) = template_seri.health_multiplier {
                if health_multiplier < 0.0 {
                    warn!(target: "being_template_init", "BeingTemplate '{}' has negative health multiplier {}, setting to 0.0", str_id, health_multiplier);
                }
            }
        }
    }
    cmd.try_insert_batch(main_comps);
    cmd.try_insert_batch(samples);
    cmd.try_insert_batch(faction_refs_to_insert);
    cmd.try_insert_batch(race_refs_to_insert);
}
