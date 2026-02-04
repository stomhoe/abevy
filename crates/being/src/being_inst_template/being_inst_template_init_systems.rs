use being_shared::BeingInstTemplate;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use common::{entity_map_macros::*, define_entity_map_systems};
use game_common::{game_common_components_samplers::EntityWeightedSampler, game_common_string_components::*};
use sprite::{sprite_components::SpriteCfgsToBuild, SpriteCfgEntityMap, sprite_sampler::SpriteWeightedSamplersMap};
use sprite_shared::SampleSprites;

use crate::being_inst_template::{self, being_inst_template_components::*, being_inst_template_resources::*};



pub fn init_being_templates(
    mut cmd: Commands,
    mut seris_handles: ResMut<BitSerisHandles>,
    mut assets: ResMut<Assets<BitSerialization>>,
    sc_emap: Res<SpriteCfgEntityMap>,
    sws_emap: Res<SpriteWeightedSamplersMap>,
) {
    use std::mem::take;
    let mut main_comps = Vec::new();
    let mut sprite_distributions_to_insert = Vec::new();
    let mut scs_to_insert = Vec::new();
    for handle in take(&mut seris_handles.handles) {
        if let Some(template_seri) = assets.remove(handle.id()) {
            let str_id = StrId::trunc(&template_seri.id);

            let bit_entity = cmd.spawn_empty().id();

            let being_inst_template = BeingInstTemplate {
                points: template_seri.points,
            };

            main_comps.push((bit_entity, (being_inst_template, str_id.clone(), )));

            if let Some(sprites_weight_maps) = template_seri.scs_samplers {
                let mut samplers = Vec::new();
                for weight_map_id in sprites_weight_maps {
                    match sws_emap.0.get_cloned(&weight_map_id) {
                        Ok(sampler_entity) => {
                            samplers.push(sampler_entity);
                        },
                        Err(_) => {
                            warn!(target: "being_template_init", "BeingTemplate '{}' sprite weighted sampler '{}' not found in SpriteWeightedSamplersMap", str_id, weight_map_id);
                        }
                    }
                }
                if !samplers.is_empty() {
                    sprite_distributions_to_insert.push((bit_entity, SampleSprites(samplers)));
                }
            }
            if let Some(sprite_ids) = template_seri.scs_ids {
                let mut sprite_entities = SpriteCfgsToBuild::with_capacity(sprite_ids.len());
                for sprite_id in sprite_ids {
                    let Ok(sprite_entity) = sc_emap.0.get_cloned(&sprite_id) else {
                        warn!(target: "being_template_init", "BeingTemplate '{}' sprite '{}' not found in SpriteCfgEntityMap", str_id, sprite_id);
                        continue;
                    };
                    sprite_entities.0.insert(sprite_entity);
                }
                if !sprite_entities.0.is_empty() {
                    scs_to_insert.push((bit_entity, sprite_entities));
                }
            }
            if let Some(health_multiplier) = template_seri.health_multiplier {
                if health_multiplier < 0.0 {
                    warn!(target: "being_template_init", "BeingTemplate '{}' has negative health multiplier {}, setting to 0.0", str_id, health_multiplier);
                }
                cmd.entity(bit_entity).try_insert(BitHealthMultiplier(health_multiplier.max(0.0)));
            }
        }
    }
    cmd.try_insert_batch(main_comps);
    cmd.try_insert_batch(sprite_distributions_to_insert);
    cmd.try_insert_batch(scs_to_insert);
}

