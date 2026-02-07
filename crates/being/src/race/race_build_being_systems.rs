use bevy::ecs::query::{Changed, With};
#[allow(unused_imports)] use bevy::prelude::*;
use common::common_id_components::HashId;
use dimension_shared::DimensionRef;
use tilemap_shared::{GlobalGenSettings, GlobalTilePos};

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
    race_query: Query<(
        Option<&SexesSampler>,
        Option<&MappedSpritesToSample>,
        Option<&BodyTreeStrIdRef>,
    ), With<Race>>,
    query: Query<(
        Entity,
        &RaceRef,
        AnyOf<(&GlobalTilePos, &Transform)>,
        &DimensionRef,
        Option<&SampleSpriteEnts>,
        Option<&SampleTreeEnt>,
        Option<&SampleBodyFromStrId>,
    ), (Changed<RaceRef>, With<Being>)>,
) {
    if query.is_empty() {
        return;
    }
    let Ok(global_gen_settings) = global_gen_settings.single() else {
        error!("Failed to get global gen settings");
        return;
    };
    let mut sample_sprites_to_ins: Vec<(Entity, SampleSpriteEnts)> = Vec::new();
    let mut sample_bodies_to_ins: Vec<(Entity, SampleBodyFromStrId)> = Vec::new();

    for (ent, race_ref, (gpos, transform), dimension_ref, sample_sprites, sample_tree, sample_body_strid) in query.iter() {
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
