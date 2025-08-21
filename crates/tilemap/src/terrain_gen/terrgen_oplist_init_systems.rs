


use bevy::prelude::*;

use dimension::dimension_components::MultipleDimensionStringRefs;

use common::common_components::{DisplayName, EntityPrefix, StrId};

use crate::{chunking_components::*, terrain_gen::{terrgen_components::FnlNoise, terrgen_oplist_components::*, terrgen_resources::*}, tile::{tile_resources::*, tile_samplers_resources::TileWeightedSamplersMap}};

use std::mem::take;

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
            let str_id = match StrId::new(seri.id.clone(), 3) {
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
                    if operand.is_empty() {
                        continue;
                    }

                    let operand = if let Ok(value) = operand.parse::<f32>() {
                        Operand::Value(value)
                    } else if let Some(tuple_str) = operand.strip_prefix("(") {
                        // Try to parse a tuple of f32, e.g. "(1.0,2.0)"
                        if let Some(end_idx) = tuple_str.find(')') {
                            let inner = &tuple_str[..end_idx];
                            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                            if parts.len() == 2 {
                                if let (Ok(a), Ok(b)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                                    Operand::Pair(a, b)
                                } else {
                                    warn!("Failed to parse tuple operand: '{}'", operand);
                                    continue;
                                }
                            } else {
                                warn!("Tuple operand does not have 2 elements: '{}'", operand);
                                continue;
                            }
                        } else {
                            warn!("Malformed tuple operand: '{}'", operand);
                            continue;
                        }
                    }
                    else if let Some(var_i) = operand.strip_prefix("$") {
                        match var_i.parse::<u8>() {
                            Ok(var_i) => {
                                if var_i >= VariablesArray::SIZE {
                                    warn!("Stack array index ${} is greater or equal to {}, which is out of bounds", var_i, VariablesArray::SIZE);
                                }
                                Operand::StackArray(var_i)
                            }
                            Err(_) => {
                                warn!("Failed to parse Stack array index from '{}'", operand);
                                continue;
                            }
                        }
                    } else if let Some(seed_str) = operand.strip_prefix("hp") {
                        let seed = seed_str.parse::<u64>().unwrap_or(1000);
                        Operand::HashPos(seed)    
                    } else if let Some(pd_str) = operand.strip_prefix("pd") {
                          // Parse PoissonDisk operand: "pd{min_dist}{seed}"
                          // Example: "pd3123" -> min_dist = 3, seed = 123
                          let (min_dist_str, seed_str) = pd_str.split_at(1);
                          let min_dist = match min_dist_str.parse::<u8>() {
                                Ok(val) => val,
                                Err(_) => {
                                     warn!("Invalid PoissonDisk min_dist: '{}'", min_dist_str);
                                     continue;
                                }
                          };
                          let seed = match seed_str.parse::<u64>() {
                                Ok(seed) => seed,
                                Err(_) => {
                                     warn!("Invalid PoissonDisk seed: '{}'", seed_str);
                                     0
                                }
                          };
                          match Operand::new_poisson_disk(min_dist, seed) {
                                Ok(op) => op,
                                Err(e) => {
                                     warn!("Failed to create PoissonDisk operand: {}", e);
                                     continue;
                                }
                          }
                    } else if let Some(ent_str) = operand.strip_prefix("fnl.") {
                        // Handle entity operand, possibly with 'COMP' prefix for complement
                        let (complement, ent_str) = if let Some(stripped) = ent_str.strip_prefix("^.") {
                            (true, stripped)
                        } else {
                            (false, ent_str)
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
                     

                        Operand::NoiseEntity(ent, fnl::NoiseSampleRange::ZeroToOne, complement, extra_seed)
                    } else {
                        error!("Unknown operand: {}", operand);
                        continue;
                    };

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
                    "pow" => Operation::Pow,
                    "avg" => Operation::Average,
                    "abs" => Operation::Abs,
                    "*nm" => Operation::MultiplyNormalized,
                    "*nmabs" => Operation::MultiplyNormalizedAbs,
                    "idxmax" => Operation::i_Max,
                    "lin" => Operation::Linear,
                    "spline" => {
                        let mut spline_vec: Vec<Vec2> = Vec::new();
                        while operands.len() > 1 {
                            let operand = operands.remove(0);
                            if let Operand::Pair(x, y) = operand {
                                spline_vec.push(Vec2::new(x, y));
                            } else {
                                error!("Expected Pair operand for Spline operation, got {:?}", operand);
                            }
                        }    

                    
                        if let Some(Operand::Pair(_, _)) = operands.last() {
                            error!("Expected single operand for sampling Spline, got {:?}", operands.last());
                            continue;
                        } else {

                            let Ok(curve) = CubicCardinalSpline::new(1.0, spline_vec).to_curve() else{
                                error!("Failed to create curve from spline points");
                                continue;
                            };
                            //let positions: Vec<_> = curve.iter_positions(100).collect();
                            Operation::Curve(curve)
                        }
                    },
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
                } else {
                    cmd.entity(bifurcation_ent).insert(ChildOf(oplist_ent));
                    oplist.bifurcations[i].oplist = Some(bifurcation_ent);
                }
            }
        }
    }
    Ok(())
}



 