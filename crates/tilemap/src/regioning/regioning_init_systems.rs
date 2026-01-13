
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use camera::camera_components::CameraTarget;
use common::common_components::{StrId, StrId20B};
use dimension_shared::{DimensionEntityMap, DimensionRef, MultipleDimensionRefs}
;
use game_common::{game_common_components::{HashedTagsVec, TagHashSet}, game_common_components_samplers::EntityWeightedSampler};
use tilemap_shared::{AcGlobalGenSettings, ChunkPos, GlobalTilePos, HashablePosVec, PoissonDisk, RegionPos};

use crate::{regioning::{regioning_components::{StructuredGenConfig, StructuredGenCfgsWeightedMap, WhitelistedFilterOf}, regioning_resources::*}, terrain_gen::{terrgen_messages::OpFilter, terrgen_resources::*}};





#[allow(unused_parens)]
pub fn init_structured_gen_configs (
    mut cmd: Commands, 
    map: Option<Res<StructuredGenConfigEntityMap>>,
    mut seris_handles: ResMut<StructureSerisHandles>,
    mut assets: ResMut<Assets<StructuredGenConfigSeri>>,
    dimension_entity_map: Res<DimensionEntityMap>,
    
) {
    if map.is_some(){ return;}
    
    let mut ent_w_sampler = EntityWeightedSampler::default();
    
    let mut opfilters_to_spawn = Vec::new();
    let mut dimension_refs_to_insert = Vec::new();
    let mut gen_cfgs_to_insert = Vec::new();
    
    let mut map = StructuredGenConfigEntityMap::default();
    for handle in std::mem::take(&mut seris_handles.handles) {
        let Some(structured_gen_seri) = assets.remove(&handle) else {
            warn!(target: "structure_spawn", "Failed to load StructureSeri from handle: {:?}", handle);
            continue;
        };
        info!(target: "structure_spawn", "Loading StructureSeri from handle: {:?}", handle);
        
        let main_ent = cmd.spawn_empty().id();
        
        let mut gen_cfg = StructuredGenConfig::default();

        gen_cfg.structure_id = StrId::new_truncated(structured_gen_seri.structure_id);
        
        if let Some(max_per_region) = structured_gen_seri.max_per_region {
            gen_cfg.max_per_region = max_per_region;
        }
        if let Some(args) = structured_gen_seri.args.clone() {
            gen_cfg.args = args;
        }
        if let Some(tags) = structured_gen_seri.tags.clone() {
            let tags = TagHashSet::new(tags);
            cmd.entity(main_ent).try_insert(tags);
        }
        
        
        if let Some(whitelisted_filters) = structured_gen_seri.whitelisted_filters{
            if ! whitelisted_filters.is_empty(){
                
                for opfilter_seri in whitelisted_filters {
                    
                    let Ok(tags) = HashedTagsVec::new_error_if_set_empty(opfilter_seri.tags) else {
                        error!(target: "structure_spawn", "OpFilterSerialization within {} has no tags.", structured_gen_seri.id);
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
                    opfilters_to_spawn.push((opfilter, WhitelistedFilterOf::new(main_ent), StrId::new_truncated(structured_gen_seri.id.clone())));
                }
            }
        }
        if let Some(exclusive_for_dimensions) = structured_gen_seri.exclusive_for_dimensions{
            if ! exclusive_for_dimensions.is_empty(){
                let mut dim_refs = MultipleDimensionRefs::default();
                for dim_strid in exclusive_for_dimensions {
                    let Ok(dim_ent) = dimension_entity_map.0.get(&dim_strid) else {
                        error!(target: "structure_spawn", "Failed to find Dimension with StrId: {}", dim_strid);
                        continue;
                    };
                    dim_refs.0.insert(dim_ent);
                }
                if dim_refs.0.is_empty(){
                    error!(target: "structure_spawn", "StructureSeri with id: {} has no valid dimension references.", structured_gen_seri.id);
                    continue;
                }
                dimension_refs_to_insert.push((main_ent, dim_refs));
            }
        }
        if let Some(pdisk_mindist_seed_vec) = structured_gen_seri.pdisk_mindist_and_tag {
            
            let Ok(poisson_disk) = PoissonDisk::multiple_tagged(pdisk_mindist_seed_vec, 3, 16)
            else {
                error!(target: "structure_spawn", "Failed to create PoissonDisk for StructureSeri with id: {}, skipping PoissonDisk creation.", structured_gen_seri.id);
                continue;
            };
            
            cmd.entity(main_ent).insert(poisson_disk);
        }
        
        ent_w_sampler.insert(main_ent, structured_gen_seri.weight);
        
        gen_cfgs_to_insert.push((main_ent, gen_cfg));
        
        map.0.overwrite(structured_gen_seri.id.clone(), main_ent);
        
    }
    cmd.insert_resource(map);
    cmd.spawn_batch(opfilters_to_spawn);
    cmd.insert_batch(dimension_refs_to_insert);
    cmd.insert_batch(gen_cfgs_to_insert);
    
    cmd.spawn((StructuredGenCfgsWeightedMap, ent_w_sampler, ));
}