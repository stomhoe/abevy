use ::being_shared::*;
use bevy::prelude::*;
use common::common_components::*;
use faction::faction_components::BelongsToFaction;
use sprite_shared::SampleSpriteEnts;

use crate::{being_inst_template::{BeingInstTemplateEntityMap, being_inst_template_components::*, being_inst_template_resources::*}, body::body_sampler::body_sampler_components::SampleTreeEnt, race::race_resources::RaceRef};

#[allow(unused_parens, )]
pub fn build_being_from_being_inst_template_ref(
    mut cmd: Commands,
    bit_query: Query<(&BeingInstTemplate, Option<&SampleSpriteEnts>, Option<&RaceRef>, Option<&SampleTreeEnt>, Option<&BelongsToFaction>), >,
    beings_to_instantiate: Query<(Entity, &BitRef), (Changed<BitRef>)>,
) {
    let mut sample_sprites_to_ins = Vec::new();
    let mut race_refs_to_ins = Vec::new();
    let mut tree_sample_to_ins = Vec::new();
    let mut belongs_to_fac_refs_to_ins = Vec::new();

    for (being_ent, bit_ref) in beings_to_instantiate.iter() {
        let Ok((template, sample_sprites, race_ref, sample_body_body_tree, belongs_to_fac)) = bit_query.get(bit_ref.0) else {
            warn!(target: "bit_build", "BitRef entity {:?} could not be resolved to BeingInstTemplate", bit_ref.0);
            continue;
        };

        if let Some(sample_sprites) = sample_sprites {
            sample_sprites_to_ins.push((being_ent, sample_sprites.clone()));
        }
        if let Some(sample_body_body_tree) = sample_body_body_tree {
            tree_sample_to_ins.push((being_ent, sample_body_body_tree.clone()));
        }

        if let Some(belongs_to_fac) = belongs_to_fac {
            belongs_to_fac_refs_to_ins.push((being_ent, belongs_to_fac.clone()));
        }

        if let Some(race_ref) = race_ref {
            race_refs_to_ins.push((being_ent, race_ref.clone()));
        }

        if template.extra_health_multiplier != 1. {
            //add in a modifier
        }
    }
    cmd.try_insert_batch(sample_sprites_to_ins);
    cmd.try_insert_batch(race_refs_to_ins);
    cmd.try_insert_batch(tree_sample_to_ins);
    cmd.try_insert_batch_if_new(belongs_to_fac_refs_to_ins);
}
