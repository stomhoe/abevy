
use bevy::prelude::*;
use common::{common_components::*, common_tag_components::{HashedTagsVec, TagSet}};
use game_common::{game_common_components::ArgsDict, game_common_samplers::EntityWeightedSampler};
use ::tilemap_shared::*;

use crate::{regioning::{StructuredGenConfigEntityMap, regioning_resources::*, regioning_sgc_components::*}, terrain::opfilter::{opfilter_components::OpFilter, opfilter_resources::OpFilterEntityMap}, };

#[allow(unused_parens)]
pub fn init_structured_gen_configs (
    mut cmd: Commands,
    map: Res<StructuredGenConfigEntityMap>,
    dimension_entity_map: Res<DimensionEntityMap>,
    egui_holder_query: Query<Entity, With<EguiSgcsHolder>>,
    opfilter_entity_map: Res<OpFilterEntityMap>,
) {
    if ! map.0.is_empty(){ return;}

    let mut ent_w_sampler = EntityWeightedSampler::default();

    let mut sgcs_comps = Vec::new();

    let mut exclusive_for_dims = Vec::new();

    let egui_ent = if let Ok(egui_ent) = egui_holder_query.single() {
        egui_ent
    } else {
        cmd.spawn(EguiSgcsHolder).id()
    };

    for structured_gen_seri in load_sgc_seri_defs() {
        let seri_id = structured_gen_seri.id.clone();

        let main_ent = cmd.spawn_empty().id();


        let mut gen_cfg = StructuredGenConfig::new(structured_gen_seri.structure_id, );

        let sgc_id = StrId::trunc(structured_gen_seri.id.clone());


        gen_cfg.max_per_region = structured_gen_seri.max_per_region;
        if !structured_gen_seri.args.is_empty() {
            gen_cfg.args = ArgsDict::with_capacity(structured_gen_seri.args.len());
            for (key, val_vec) in structured_gen_seri.args.clone() {
                gen_cfg.args.insert(key, val_vec);
            }
        }
        if !structured_gen_seri.tags.is_empty() {
            let tags = TagSet::new(structured_gen_seri.tags.clone());
            cmd.entity(main_ent).try_insert(tags);
        }


        if !structured_gen_seri.whitelisted_filters.is_empty() {
            let opfilter_ents: Vec<Entity> = Vec::with_capacity(structured_gen_seri.whitelisted_filters.len());
            for opfilter_id in structured_gen_seri.whitelisted_filters {


            }
        }
        if !structured_gen_seri.exclusive_for_dimensions.is_empty() {
            let mut dim_refs = MultipleDimensionRefs::default();
            for dim_strid in structured_gen_seri.exclusive_for_dimensions.iter() {
                let Ok(dim_ent) = dimension_entity_map.0.get_cloned(&dim_strid) else {
                    error!(target: "sgc_init", "Failed to find Dimension with StrId: {}", dim_strid);
                    continue;
                };
                dim_refs.0.insert(dim_ent);
            }
            if dim_refs.0.is_empty(){
                error!(target: "sgc_init", "StructureSeri with id: {} has no valid dimension references.", seri_id);
                continue;
            }
            exclusive_for_dims.push((main_ent, dim_refs));
        }
        if !structured_gen_seri.pdisk_mindist_and_tag.is_empty() {

            let Ok(poisson_disk) = PoissonDisk::multiple_tagged(structured_gen_seri.pdisk_mindist_and_tag.clone(), 3, 16)
            else {
                error!(target: "sgc_init", "Failed to create PoissonDisk for StructureSeri with id: {}, skipping PoissonDisk creation.", seri_id);
                continue;
            };

            cmd.entity(main_ent).insert(poisson_disk);
        }

        ent_w_sampler.insert(main_ent, structured_gen_seri.weight);


        sgcs_comps.push((main_ent, (sgc_id, gen_cfg, )));
    }
    cmd.insert_batch(exclusive_for_dims);
    cmd.insert_batch(sgcs_comps);

    cmd.spawn((SgcsWeightedSampler, ent_w_sampler, ChildOf(egui_ent),));
}
