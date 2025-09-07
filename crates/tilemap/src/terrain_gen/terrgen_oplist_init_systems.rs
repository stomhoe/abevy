


use bevy::{ecs::entity::EntityHashMap, prelude::*};

use bevy_replicon::shared::server_entity_map::ServerEntityMap;

use common::common_components::{DisplayName, EntityPrefix, StrId};
use dimension_shared::{Dimension, DimensionRootOplist, MultipleDimensionRefs, MultipleDimensionStringRefs};

use crate::{chunking_components::*, terrain_gen::{terrgen_components::FnlNoise, terrgen_oplist_components::*, terrgen_resources::*}, tile::{tile_resources::*, tile_sampler_resources::TileWeightedSamplersMap}};
use ::tilemap_shared::*;

use std::mem::take;
use std::collections::HashSet;

#[allow(unused_parens)]
pub fn init_oplists_from_assets(
    mut cmd: Commands, seris_handles: Res<OpListSerisHandles>,
    mut assets: ResMut<Assets<OpListSerialization>>, 
    terr_gen_map: Res<TerrGenEntityMap>,  
    samplers_map: Res<TileWeightedSamplersMap>,
    tiles_map: Res<TileEntitiesMap>,
    oplist_map: Option<Res<OpListEntityMap>>,
) {
    if oplist_map.is_some() { return ; }
    cmd.init_resource::<OpListEntityMap>();

    for handle in seris_handles.handles.iter() {//ESTE VA CON ITER
        if let Some(seri) = assets.get_mut(handle) {
            let str_id = match StrId::new_with_result(seri.id.clone(), 3) {
                Ok(id) => id,
                Err(_) => {
                    error!("Failed to create StrId for oplist {}", seri.id);
                    continue;
                }
            };
            if seri.is_root() && seri.operation_operands.is_empty() {
                error!("root OpListSeri has no operations");
                continue;
            }
            let size =
                if let Some(size) = seri.size {
                    if let Ok(size) = OplistSize::new(size) {
                        size
                    } else {
                        error!("Invalid oplist_size for {}, must be in [1,4] for each vec component", seri.id);
                        continue;
                    }
                } else{
                    OplistSize::default()
                };

            let mut oplist = OperationList::default();

            //define a mutable array of 16 f64s here

            for (operation, str_operands, out) in seri.operation_operands.iter() {

                if *out >= VariablesArray::SIZE {
                    error!("Output index {} out of bounds for OperationList", out);
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
                            warn!("Failed to parse Stack array index from '{}'", operand);
                            continue;
                        };
                        if var_i >= VariablesArray::SIZE {
                            warn!("Stack array index ${} is greater or equal to {}, which is out of bounds", var_i, VariablesArray::SIZE);
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
                            warn!("Invalid PoissonDisk min_dist ('{}') or seed ('{}')", min_dist_str, seed_str);
                            continue;
                        };
                        let Ok(op) = OperandElement::new_poisson_disk(min_dist, seed) else {
                            warn!("Failed to create PoissonDisk operand with min_dist {} and seed {}", min_dist, seed);
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
                        let Ok(ent) = terr_gen_map.0.get(&base_str.to_string()) else {
                            warn!("Entity not found in TerrGenEntityMap: {}", base_str);
                            continue;
                        };

                        OperandElement::NoiseEntity(ent, noise_sample_range, complement, extra_seed)
                    } else {
                        error!("Unknown operand: {}", operand);
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
                    "*opo" => Operation::MultiplyOpo,
                    "/" => Operation::Divide,
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
                        error!("Unknown operation: {}", operation);
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
                    if let Ok(sampler_ent) = samplers_map.0.get(tile_str) {
                        Some(sampler_ent)
                    } else if let Ok(tile_ent) = tiles_map.0.get(tile_str) {
                        Some(tile_ent)
                    } else {
                        warn!("Tile {} not found in TilingEntityMap or TileWeightedSamplersMap", tile_str);
                        None
                    }
                }).collect::<Vec<Entity>>();

                let bifurcation = Bifurcation { oplist: None, tiles };
                oplist.bifurcations.push(bifurcation);
            }
            let spawned_oplist = cmd.spawn(( str_id, oplist, size)).id();
            if seri.is_root() { cmd.entity(spawned_oplist).insert(MultipleDimensionStringRefs::new(take(&mut seri.root_in_dimensions))); }

        } 
    }
} 

#[allow(unused_parens)]
pub fn add_oplists_to_map(
    mut cmd: Commands, 
    oplist_map: Option<ResMut<OpListEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId, ),(Added<StrId>, With<OperationList>)>,) 
{
    if let Some(mut oplist_map) = oplist_map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = oplist_map.0.insert(str_id, ent, ) {

                error!("{} {} already in OpListEntityMap : {}", prefix, str_id, err);
                cmd.entity(ent).despawn();
            }
        }
    }
}

#[allow(unused_parens)]
pub fn init_oplists_bifurcations(
    mut cmd: Commands,
    mut seris_handles: ResMut<OpListSerisHandles>,
    mut assets: ResMut<Assets<OpListSerialization>>, 
    oplist_map: Res<OpListEntityMap>,
    mut oplist_query: Query<(&mut OperationList, )>,
    is_root: Query<(&MultipleDimensionStringRefs)>,
) -> Result {
    for handle in take(&mut seris_handles.handles) {
        if let Some(seri) = assets.remove(&handle) {
            let oplist_ent = oplist_map.0.get(&seri.id)?;
            let (mut oplist, ) = oplist_query.get_mut(oplist_ent)?;

            for (i, seri_bifurcation) in seri.bifs.iter().enumerate() {
                let bifurcation_str = seri_bifurcation.0.trim();
                if bifurcation_str.is_empty() { continue; }

                let Ok(bifurcation_ent) = oplist_map.0.get(&bifurcation_str.to_string()) else {
                    error!(
                        "bifurcation entity with id '{}' not found in OpListEntityMap",
                        bifurcation_str
                    );
                    continue;
                };
                if oplist_ent == bifurcation_ent {
                    error!("bifurcation entity with id '{}' would make parent diverge into itself ", bifurcation_str);
                    continue;
                }
                if is_root.get(bifurcation_ent).is_ok() {
                    error!("bifurcation entity with id '{}' must not be a root oplist", bifurcation_str);
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

    // Helper closure for DFS cycle detection
    fn dfs(
        query: &Query<(Entity, &OperationList, Has<MultipleDimensionStringRefs>)>,
        current: Entity,
        visited: &mut HashSet<Entity>,
        stack: &mut Vec<Entity>,
    ) -> bool {
        if stack.contains(&current) {
            error!("Cycle detected, caused by oplist entity {:?}'s bifurcations", current);
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
            error!("Cycle detected starting from root oplist {:?}", root);
        }
    }
}


#[allow(unused_parens)]
pub fn oplist_init_dim_refs(mut cmd: Commands, 
    oplist_query: Query<(Entity, &StrId, &MultipleDimensionRefs),(With<OperationList>, )>,
    dimension_query: Query<(&StrId, Option<&DimensionRootOplist>), With<Dimension>>,
) {
    let mut assignments: EntityHashMap<Entity> = EntityHashMap::new();
    
    for (ent, my_str_id, dim_refs) in oplist_query.iter() {
        for &dim_ent in dim_refs.0.iter() {
            let Ok((dim_str_id, root_op_list)) = dimension_query.get(dim_ent) else {
                error!(target: "dimension_loading", "Dimension entity '{}' referenced by DimensionEntityMap is not spawned in world", dim_ent);
                continue;
            };

            match (assignments.get(&dim_ent), root_op_list) {
                (Some(&other_ent), _) => {
                    if other_ent == ent { warn!("self is already dimoplist"); continue; }
                    let Ok((_, other_id, _, )) = oplist_query.get(other_ent) else {
                        continue;
                    };
                    error!("Dimension {} already has root operation list {}; couldn't assign {} as its root oplist", dim_str_id, other_id, my_str_id);
                    continue;
                },
                (_, Some(&DimensionRootOplist(other_ent))) => {
                    if other_ent == ent { warn!("self is already dimoplist"); continue; }

                    let Ok((_, other_id, _, )) = oplist_query.get(other_ent) else {
                        continue;
                    };
                    error!("Dimension {} already has root operation list {}; couldn't assign {} as its root oplist", dim_str_id, other_id, my_str_id);
                continue;
                },
                (None, None) => {
                    assignments.insert(dim_ent, ent);
                    cmd.entity(dim_ent).insert(DimensionRootOplist(ent));
                },
            }
        }
        cmd.entity(ent).remove::<MultipleDimensionRefs>();
    }
}



#[allow(unused_parens)]
pub fn client_remap_operation_entities(
    mut query: Query<(&mut OperationList), (Added<OperationList>)>, 
    mut map: ResMut<ServerEntityMap>,
)
{
    for mut oplist in query.iter_mut() {

        for (_, operands, _) in oplist.trunk.iter_mut() {
            for operand in operands.iter_mut() {
                if let OperandElement::NoiseEntity(ent, _, _, _) = &mut operand.element {
                    match map.server_entry(*ent).get() {
                        Some(new_ent) => {
                            info!(
                                target: "oplist_loading",
                                "Remapped noise entity {:?} to {:?} in OperandElement",
                                ent,
                                new_ent
                            );
                            *ent = new_ent;
                        },      
                        None => {
                            error!(
                                target: "oplist_loading",
                                "Failed to remap noise entity {:?} in OperandElement: not found in ServerEntityMap",
                                ent
                            );
                        }
                    }
                }
            }
        }
        for bifur in oplist.bifurcations.iter_mut() {
            if let Some(oplist_entity) = bifur.oplist {
                match map.server_entry(oplist_entity).get() {
                    Some(new_ent) => {
                        info!(
                            target: "oplist_loading",
                            "Remapped oplist entity {:?} to {:?} in Bifurcation",
                            oplist_entity,
                            new_ent
                        );
                        bifur.oplist = Some(new_ent);
                    },      
                    None => {
                        error!(
                            target: "oplist_loading",
                            "Failed to remap oplist entity {:?} in Bifurcation: not found in ServerEntityMap",
                            oplist_entity
                        );
                        bifur.oplist = None;
                    }
                }
            }
            bifur.tiles.iter_mut().for_each(|tile_entity| *tile_entity = map.server_entry(*tile_entity).get().unwrap_or(*tile_entity));
        }
    }
}