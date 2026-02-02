
use bevy::{ecs::entity::EntityHashSet, platform::collections::HashMap, prelude::*};
use common::{common_components::*, common_tag_components::{HashedTagsVec, TagSet}};
use ::dimension_shared::*;
use game_common::{game_common_components::ArgsMap, game_common_components_samplers::EntityWeightedSampler};
use ::tilemap_shared::*;

use crate::{regioning::{regioning_resources::*, regioning_sgc_components::*}, terrain_gen::{terrgen_messages::OpFilter, terrgen_resources::*}};

#[allow(unused_parens)]
pub fn init_structured_gen_configs (
    mut cmd: Commands, 
    map: Res<SgcEntityMap>,
    mut seris_handles: ResMut<StructureSerisHandles>,
    mut assets: ResMut<Assets<StructuredGenConfigSeri>>,
    dimension_entity_map: Res<DimensionEntityMap>,
    
) {
    if ! map.0.is_empty(){ return;}
    
    let mut ent_w_sampler = EntityWeightedSampler::default();

    let holder = cmd.spawn(EguiSgcHolder).id();
    

    let mut sgcs_comps = Vec::new();

    let mut opfilters_to_spawn = Vec::new();
    let mut exclusive_for_dims = Vec::new();
    
    for handle in std::mem::take(&mut seris_handles.handles) {
        let Some(structured_gen_seri) = assets.remove(&handle) else {
            warn!(target: "sgc_init", "Failed to load StructureSeri from handle: {:?}", handle);
            continue;
        };
        info!(target: "sgc_init", "Loading StructureSeri from handle: {:?}", handle);
        
        let main_ent = cmd.spawn_empty().id();
        

        let mut gen_cfg = StructuredGenConfig::new(structured_gen_seri.structure_id, );

        let sgc_id = StrId::trunc(structured_gen_seri.id.clone());


        
        if let Some(max_per_region) = structured_gen_seri.max_per_region {
            gen_cfg.max_per_region = max_per_region;
        }
        if let Some(args) = structured_gen_seri.args.clone() {
            gen_cfg.args = ArgsMap::with_capacity(args.len());
            for (key, val_vec) in args {
                gen_cfg.args.insert(key, val_vec);
            }
        }
        if let Some(tags) = structured_gen_seri.tags.clone() {
            let tags = TagSet::new(tags);
            cmd.entity(main_ent).try_insert(tags);
        }
        
        
        if let Some(whitelisted_filters) = structured_gen_seri.whitelisted_filters{
            if ! whitelisted_filters.is_empty(){
                
                for opfilter_seri in whitelisted_filters {
                    
                    let Ok(tags) = HashedTagsVec::new_error_if_set_empty(opfilter_seri.tags) else {
                        error!(target: "sgc_init", "OpFilterSerialization within {} has no tags.", structured_gen_seri.id);
                        continue;
                    };
                    
                    let opfilter = OpFilter {
                        start_oplist: Entity::PLACEHOLDER,
                        tags,
                        op_i: opfilter_seri.op_i,
                        min_val: opfilter_seri.min_val,
                        max_val: opfilter_seri.max_val,
                        search_start_pos: GlobalTilePos::default(),
                    };
                    opfilters_to_spawn.push((opfilter, WhitelistedFilterOf::new(main_ent), StrId::trunc(structured_gen_seri.id.clone())));
                }
            }
        }
        if let Some(exclusive_for_dimensions) = structured_gen_seri.exclusive_for_dimensions{
            if ! exclusive_for_dimensions.is_empty(){
                let mut dim_refs = MultipleDimensionRefs::default();
                for dim_strid in exclusive_for_dimensions {
                    let Ok(dim_ent) = dimension_entity_map.0.get_cloned(&dim_strid) else {
                        error!(target: "sgc_init", "Failed to find Dimension with StrId: {}", dim_strid);
                        continue;
                    };
                    dim_refs.0.insert(dim_ent);
                }
                if dim_refs.0.is_empty(){
                    error!(target: "sgc_init", "StructureSeri with id: {} has no valid dimension references.", structured_gen_seri.id);
                    continue;
                }
                exclusive_for_dims.push((main_ent, dim_refs));
            }
        }
        if let Some(pdisk_mindist_seed_vec) = structured_gen_seri.pdisk_mindist_and_tag {
            
            let Ok(poisson_disk) = PoissonDisk::multiple_tagged(pdisk_mindist_seed_vec, 3, 16)
            else {
                error!(target: "sgc_init", "Failed to create PoissonDisk for StructureSeri with id: {}, skipping PoissonDisk creation.", structured_gen_seri.id);
                continue;
            };
            
            cmd.entity(main_ent).insert(poisson_disk);
        }
        
        ent_w_sampler.insert(main_ent, structured_gen_seri.weight);

        
        sgcs_comps.push((main_ent, (sgc_id, gen_cfg, ChildOf(holder),)));
    }
    cmd.spawn_batch(opfilters_to_spawn);
    cmd.insert_batch(exclusive_for_dims);
    cmd.insert_batch(sgcs_comps);
    
    cmd.spawn((SgcsWeightedSampler, ent_w_sampler, ChildOf(holder),));
}

pub fn map_sgc_id_to_entity(
    mut cmd: Commands,
    map: Option<ResMut<SgcEntityMap>>,
    ezeros_query: Query<(Entity, Option<&Prefix>, &StrId), (Changed<StrId>, With<StructuredGenConfig>, )>,
) {
    if let Some(mut map) = map {
        for (ent, prefix, str_id) in ezeros_query.iter() {
            if let Err(prev_ent) = map.0.insert(str_id, ent, ) {
                if prev_ent.0 == ent {
                    continue;
                }
                error!(target: "sgc_init","{} '{}' already in SgcEntityMap with entity {:?}, cannot insert entity {:?}", prefix.cloned().unwrap_or_default(), str_id, prev_ent, ent);
                cmd.entity(ent).try_despawn();
            } else {
                trace!(target: "sgc_init","Inserted {} '{}' into SgcEntityMap with entity {:?}", prefix.cloned().unwrap_or_default(), str_id, ent);
            }
        }
    }
    else {
        error!(target: "sgc_init","SgcEntityMap resource not found when trying to add sgc to it.");
    }
}

#[allow(unused_parens)]
pub fn remove_sgc_from_map_on_despawn(
    trigger: On<Despawn, StructuredGenConfig>,
    query: Query<(&StrId),(AnyDisabling)>,
    mut map: ResMut<SgcEntityMap>,
    mut weighted_map: Query<(&mut EntityWeightedSampler), (With<SgcsWeightedSampler>)>,

) {
    if let Ok(str_id) = query.get(trigger.entity) {
        weighted_map.iter_mut().for_each(|(mut weighted_sampler)| {
            weighted_sampler.remove(&trigger.entity);
        });
        if let Ok(found_entity) = map.0.get_cloned(str_id) {
            if found_entity == trigger.entity {
                map.0.remove(str_id.as_str());
            }
        }
    }
}