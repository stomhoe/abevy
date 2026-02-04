use ::being_shared::*;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::{game_common_components_samplers::EntityWeightedSampler, game_common_string_components::*};
use sprite::{sprite_components::SpriteCfgsToBuild, SpriteCfgEntityMap, sprite_sampler::SpriteWeightedSamplersMap};
use sprite_shared::SampleSprites;

use crate::being_inst_template::{BitEntityMap, being_inst_template_components::*, being_inst_template_resources::*};


#[allow(unused_parens)]
pub fn build_being_from_being_inst_template_ref(
    mut cmd: Commands,
    bit_query: Query<(&BeingInstTemplate, Option<&SampleSprites>, Option<&SpriteCfgsToBuild>, Option<&RaceRef>, Option<&BitHealthMultiplier>), >,
    beings_to_instantiate: Query<(Entity, &BitRef), (Changed<BitRef>)>,

) {
    let mut sample_sprites_to_ins = Vec::new();
    let mut sprite_cfg_build_ins = Vec::new();
    let mut race_refs_to_ins = Vec::new();
    let mut health_multiplier_to_ins = Vec::new();
    for (being_ent, bit_ref) in beings_to_instantiate.iter() {
        let Ok((_, sample_sprites, scs_to_build_opt, race_ref, health_multiplier)) = bit_query.get(bit_ref.0) else {
            warn!(target: "being_template_build", "BitRef entity {:?} could not be resolved to BeingInstTemplate", bit_ref.0);
            continue;
        };


        if let Some(sample_sprites) = sample_sprites {
            sample_sprites_to_ins.push((being_ent, sample_sprites.clone()));
        }

        if let Some(scs_to_build) = scs_to_build_opt {
            sprite_cfg_build_ins.push((being_ent, scs_to_build.clone()));
        }
        if let Some(race_ref) = race_ref {
            race_refs_to_ins.push((being_ent, race_ref.clone()));  
        }
        if let Some(health_multiplier) = health_multiplier {
            health_multiplier_to_ins.push((being_ent, health_multiplier.clone()));
        }
    }
    cmd.try_insert_batch(sample_sprites_to_ins);
    cmd.try_insert_batch(sprite_cfg_build_ins);
    cmd.try_insert_batch(race_refs_to_ins);
    cmd.try_insert_batch(health_multiplier_to_ins);
}

#[allow(unused_parens)]
pub fn convert_strid_to_ent(
    mut cmd: Commands,
    query: Query<(Entity, &BitStrIdRef), (Changed<BitStrIdRef>, AnyDisabling)>,
    bit_emap: Res<BitEntityMap>,
) {
    let mut bit_refs = Vec::new();
    for (ent, bit_str_id_ref) in query.iter() {
        let Ok(bit_entity) = bit_emap.0.get_cloned(&bit_str_id_ref.0) else {
            warn!(target: "being_template_build", "BitStrIdRef '{}' could not be resolved to entity in BitEntityMap", bit_str_id_ref.0);
            continue;
        };
        bit_refs.push((ent, BitRef(bit_entity)));
    }
    cmd.try_insert_batch(bit_refs);
}
