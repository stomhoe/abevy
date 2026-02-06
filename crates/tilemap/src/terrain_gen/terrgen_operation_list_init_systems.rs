


use bevy::{ecs::entity::EntityHashMap, prelude::*};

use common::{common_components::{Prefix, StrId}, common_tag_components::TagSet};
use dimension_shared::{Dimension, DimensionEntityMap, DimensionRootOplist, MultipleDimensionRefs, MultipleDimensionStringRefs};

use crate::{terrain_gen::{terrgen_components::FailedSearchOplistFilterHolder, TerrgenEntityMap, terrgen_operaton_list_components::*, terrgen_resources::*, OperationListEntityMap}, tile::{tile_resources::*, tile_sampler_resources::TileWeightedSamplerEntityMap}};
use ::tilemap_shared::*;

use std::mem::take;
use std::collections::HashSet;

#[allow(unused_parens)]
pub fn init_oplists_from_assets(
    mut cmd: Commands, seris_handles: Res<OpListSerisHandles>,
    mut assets: ResMut<Assets<OpListSerialization>>, 
    terr_gen_map: Res<TerrgenEntityMap>,  
    samplers_map: Res<TileWeightedSamplerEntityMap>,
    tiles_map: Res<TileEntityMap>,
    dimension_map: Res<DimensionEntityMap>,
    oplist_map: Res<OperationListEntityMap>,
) {
    if !oplist_map.0.is_empty() { return ; }
    
    let egui_oplist_holder_ent = cmd.spawn(EguiOplistHolder).id();
    cmd.spawn((FailedSearchOplistFilterHolder, ChildOf(egui_oplist_holder_ent)));
    
    let mut oplist_comps = Vec::new();
    let mut oplist_multiple_dimension_refs = Vec::new();
    let mut tags_to_insert = Vec::new();
    
    for handle in seris_handles.handles.iter() {//ESTE VA CON ITER
        let Some(seri) = assets.get_mut(handle) else {
            continue;
        };
        let str_id = match StrId::new_with_result(seri.id.clone(), 1) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(target: "oplist_init", "Failed to create StrId for oplist {}: {:?}", seri.id, err);
                continue;
            }
        };   
        if seri.is_root() && seri.operation_operands.is_empty() {
            error!(target: "oplist_init", "root OpListSeri has no operations");
            continue;
        }
        let size =
        if let Some(size) = seri.size {
            if let Ok(size) = OplistSize::new(size) {
                size
            } else {
                error!(target: "oplist_init", "Invalid oplist_size for {}, must be in [1,4] for each vec component", seri.id);
                continue;
            }
        } else{
            OplistSize::default()
        };
        
        
        let mut oplist = OperationList::default();
        
        //define a mutable array of 16 f64s here
        
        for (operation, str_operands, out) in seri.operation_operands.iter() {
            
            if *out >= VariablesArray::SIZE {
                error!(target: "oplist_init", "Output index {} out of bounds for OperationList", out);
                continue;
            }
            let mut operands = Vec::new();
            for operand in str_operands {
                let operand = operand.trim();    
                if operand.is_empty() { continue; }
                
                let (operand, complement) = if let Some(operand) = operand.strip_prefix("COMP") {
                    (operand.trim(), true)
                } else {
                    (operand, false)
                };
                
                let element = if let Ok(value) = operand.parse::<f32>() {
                    OperandElement::Value(value)
                }
                else if let Some(var_i) = operand.strip_prefix("$") {
                    let Ok(var_i) = var_i.parse::<u8>() else {
                        warn!(target: "oplist_init", "Failed to parse Stack array index from '{}'", operand);
                        continue;
                    };
                    if var_i >= VariablesArray::SIZE {
                        warn!(target: "oplist_init", "Stack array index ${} is greater or equal to {}, which is out of bounds", var_i, VariablesArray::SIZE);
                    }
                    OperandElement::StackArray(var_i)
                } else if let Some(seed_str) = operand.strip_prefix("hp") {
                    let seed = seed_str.parse::<u64>().unwrap_or(1000);
                    OperandElement::HashPos(seed)    
                } else if let Some(pd_str) = operand.strip_prefix("pd") {
                    // Parse PoissonDisk operand: "pd{min_dist}{seed}"
                    // Example: "pd3123" -> min_dist = 3, seed = 123
                    let (min_dist_str, seed_str) = pd_str.split_at(1);
                    let (Ok(min_dist), Ok(seed)) = (min_dist_str.parse::<u8>(), seed_str.parse::<u64>()) else {
                        warn!(target: "oplist_init", "Invalid PoissonDisk min_dist ('{}') or seed ('{}')", min_dist_str, seed_str);
                        continue;
                    };
                    let Ok(op) = OperandElement::new_poisson_disk(min_dist, seed) else {
                        warn!(target: "oplist_init", "Failed to create PoissonDisk operand with min_dist {} and seed {}", min_dist, seed);
                        continue;
                    };
                    op      
                } else if let Some(ent_str) = operand.strip_prefix("fnl.") {
                    // Handle entity operand, possibly with 'COMP' prefix for complement
                    let (noise_sample_range, ent_str) = if let Some(stripped) = ent_str.strip_prefix("1-1.") {
                        (fnl::NoiseSampleRange::NegOneToOne, stripped)
                    } else {
                        (fnl::NoiseSampleRange::ZeroToOne, ent_str)
                    };
                    
                    // If the operand_str ends with ".s" followed by a number, use it as seed
                    let (base_str, extra_seed) = if let Some(idx) = ent_str.rfind(".s") {
                        let (base, seed_str) = ent_str.split_at(idx);
                        let seed = seed_str[2..].parse::<i32>().unwrap_or(0);
                        (base, seed)
                    } else {
                        (ent_str, 0)
                    };
                    let Ok(ent) = terr_gen_map.0.get_cloned(&base_str.to_string()) else {
                        warn!(target: "oplist_init", "Entity not found in TerrgenEntityMap: {}", base_str);
                        continue;
                    };
                    
                    OperandElement::NoiseEntity(ent, noise_sample_range, complement, extra_seed)
                } else {
                    error!(target: "oplist_init", "Unknown operand: {}", operand);
                    continue;
                };
                
                let operand = Operand { complement, element, };
                
                operands.push(operand);
            };
            
            let operation = match operation.as_str().trim() {
                "" => continue,
                "+" => Operation::Add,
                "-" => Operation::Subtract,
                "*" => Operation::Multiply,
                "/" => Operation::Divide,
                "*opo" => Operation::MultiplyOpo,
                "min" => Operation::Min,
                "max" => Operation::Max,
                "avg" => Operation::Average,
                "abs" => Operation::Abs,
                "*nm" => Operation::MultiplyNormalized,
                "*nmabs" => Operation::MultiplyNormalizedAbs,
                "idxmax" => Operation::i_Max,
                "idxnorm" => Operation::i_Norm,
                "lin" => Operation::Linear,
                "clamp" => Operation::Clamp,
                _ => {
                    error!(target: "oplist_init", "Unknown operation: {}", operation);
                    continue;
                },
            };
            
            oplist.trunk.push((operation, operands, *out));    
        }
        oplist.bifurcations = Vec::with_capacity(seri.bifs.len());
        
        for (_oplist, tiles) in seri.bifs.iter() {
            let tiles = tiles
            .iter().filter(|tile_str| !tile_str.is_empty())
            .filter_map(|tile_str| {
                if let Ok(sampler_ent) = samplers_map.0.get_cloned(tile_str) {
                    Some(sampler_ent)
                } else if let Ok(tile_ent) = tiles_map.0.get_cloned(tile_str) {
                    Some(tile_ent)
                } else {
                    warn!(target: "oplist_init", "Tile {} not found in TilingEntityMap or TileWeightedSamplerEntityMap", tile_str);
                    None
                }
            }).collect::<Vec<Entity>>();
            
            let bifurcation = Bifurcation { oplist: None, tiles };
            oplist.bifurcations.push(bifurcation);
        }
        if let Ok(ent) = oplist_map.0.get_cloned(&str_id) {
            error!(target: "oplist_init", "{} already in OperationListEntityMap : {}", str_id, ent);
            continue;
        }
        let spawned_oplist = cmd.spawn_empty().id();
        oplist_comps.push((spawned_oplist, ( str_id, oplist, size, ChildOf(egui_oplist_holder_ent))));
        if seri.is_root() { 
            let mut dim_refs = MultipleDimensionRefs::default();
            for dim_id in seri.root_in_dimensions.iter() {
                if dim_id.trim().is_empty() { continue; }
                let Ok(dim_entity) = dimension_map.0.get_cloned(&dim_id) else
                {
                    error!(target: "oplist_init", "Dimension '{}' not found in DimensionEntityMap for root oplist '{}'", dim_id, seri.id);
                    continue;
                };
                dim_refs.0.insert(dim_entity);

            }
            oplist_multiple_dimension_refs.push((spawned_oplist, dim_refs));
        }
        
        
        if let Some(tags) = &seri.tags {
            tags_to_insert.push((spawned_oplist, TagSet::new(tags)));
        }
        
        
    } 
    cmd.try_insert_batch(oplist_comps);
    cmd.try_insert_batch(oplist_multiple_dimension_refs);
    cmd.try_insert_batch(tags_to_insert);
} 

#[allow(unused_parens)]
pub fn init_oplists_bifurcations(
    mut cmd: Commands,
    mut seris_handles: ResMut<OpListSerisHandles>,
    mut assets: ResMut<Assets<OpListSerialization>>, 
    oplist_map: Res<OperationListEntityMap>,
    mut oplist_query: Query<(&mut OperationList, )>,
    is_root: Query<(&MultipleDimensionStringRefs)>,
) -> Result {
    for handle in take(&mut seris_handles.handles) {
        if let Some(seri) = assets.remove(&handle) {
            let Ok(oplist_ent) = oplist_map.0.get_cloned(&seri.id) else {
                error!(
                    target: "oplist_init",
                    "oplist entity with id '{}' not found in OperationListEntityMap",
                    seri.id
                );
                continue;
            };
            let (mut oplist, ) = oplist_query.get_mut(oplist_ent)?;
            
            for (i, seri_bifurcation) in seri.bifs.iter().enumerate() {
                let bifurcation_str = seri_bifurcation.0.trim();
                if bifurcation_str.is_empty() { continue; }
                
                let Ok(bifurcation_ent) = oplist_map.0.get_cloned(&bifurcation_str.to_string()) else {
                    error!(
                        target: "oplist_init",
                        "bifurcation entity with id '{}' not found in OperationListEntityMap",
                        bifurcation_str
                    );
                    continue;
                };
                if oplist_ent == bifurcation_ent {
                    error!(target: "oplist_init", "bifurcation entity with id '{}' would make parent diverge into itself ", bifurcation_str);
                    continue;
                }
                if is_root.get(bifurcation_ent).is_ok() {
                    error!(target: "oplist_init", "bifurcation entity with id '{}' must not be a root oplist", bifurcation_str);
                    continue;
                }
                
                cmd.entity(bifurcation_ent).insert(ChildOf(oplist_ent));
                oplist.bifurcations[i].oplist = Some(bifurcation_ent);
            }   
        }
    }
    Ok(())
}

#[allow(unused_parens)]
pub fn cycle_detection(
    query: Query<(Entity, &OperationList, Has<MultipleDimensionStringRefs>)>,
) {
    let roots: Vec<Entity> = query
    .iter()
    .filter_map(|(ent, _, is_root)| if is_root { Some(ent) } else { None })
    .collect();
    fn dfs(
        query: &Query<(Entity, &OperationList, Has<MultipleDimensionStringRefs>)>,
        current: Entity,
        visited: &mut HashSet<Entity>,
        stack: &mut Vec<Entity>,
    ) -> bool {
        if stack.contains(&current) {
            error!(target: "oplist_init", "Cycle detected, caused by oplist entity {:?}'s bifurcations", current);
            return true;
        }
        if !visited.insert(current) {
            return false; 
        }
        stack.push(current);
        
        let Ok((_, oplist, _)) = query.get(current) else {
            stack.pop();
            return false;
        };
        for bifur in &oplist.bifurcations {
            if let Some(child) = bifur.oplist {
                if dfs(query, child, visited, stack) {
                    return true;
                }
            }
        }
        stack.pop();
        false
    }
    for root in roots {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        if dfs(&query, root, &mut visited, &mut stack) {
            error!(target: "oplist_init", "Cycle detected starting from root oplist {:?}", root);
        }
    }
}
#[allow(unused_parens)]
pub fn assign_rootoplist_to_dimensions(mut cmd: Commands, 
    oplist_query: Query<(Entity, &StrId, &MultipleDimensionRefs),(With<OperationList>, )>,
    dimension_query: Query<(&StrId, Option<&DimensionRootOplist>), With<Dimension>>,
) {
    let mut assignments: EntityHashMap<DimensionRootOplist> = EntityHashMap::new();
    

    for (oplist_ent, my_oplist_id, dim_refs) in oplist_query.iter() {
        for &dim_ent in dim_refs.0.iter() {
            let Ok((dim_str_id, root_op_list)) = dimension_query.get(dim_ent) else {
                error!(target: "oplist_init", "Dimension entity '{}' referenced by DimensionEntityMap is not spawned in world", dim_ent);
                continue;
            };
            
            match (assignments.get(&dim_ent), root_op_list) {
                (Some(&other_ent), _) => {
                    if other_ent.0 == oplist_ent { warn!(target: "oplist_init", "self is already dimoplist"); continue; }
                    let Ok((_, other_id, _, )) = oplist_query.get(other_ent.0) else {
                        continue;
                    };
                    error!(target: "oplist_init", "Dimension {} already has root operation list {}; couldn't assign {} as its root oplist", dim_str_id, other_id, my_oplist_id);
                    continue;
                },
                (_, Some(&DimensionRootOplist(other_ent))) => {
                    if other_ent == oplist_ent { warn!(target: "oplist_init", "self is already dimoplist"); continue; }
                    
                    let Ok((_, other_id, _, )) = oplist_query.get(other_ent) else {
                        continue;
                    };
                    error!(target: "oplist_init", "Dimension {} already has root operation list {}; couldn't assign {} as its root oplist", dim_str_id, other_id, my_oplist_id);
                    continue;
                },
                (None, None) => {
                    assignments.insert(dim_ent, DimensionRootOplist(oplist_ent));
                },
            }
        }
    }
    cmd.try_insert_batch(assignments);
}
