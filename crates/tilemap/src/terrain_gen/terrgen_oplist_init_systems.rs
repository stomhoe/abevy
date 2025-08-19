

use core::panic;

use bevy::prelude::*;

use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use dimension::dimension_components::MultipleDimensionStringRefs;
use fastnoise_lite::FastNoiseLite;

use common::common_components::{DisplayName, EntityPrefix, StrId};

use crate::{chunking_components::*, terrain_gen::{terrgen_components::*, terrgen_resources::*}, tile::{tile_resources::*, tile_samplers_resources::TileWeightedSamplersMap}};

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
            oplist.split = seri.split;

            //define a mutable array of 16 f64s here

            for (operation, str_operands, out) in seri.operation_operands.iter() {

                if *out >= VariablesArray::SIZE {
                    error!("Output index {} out of bounds for OperationList", out);
                    continue;
                }


                let operation = match operation.as_str().trim() {
                    "" => continue,
                    "CLEAR" => Operation::ClearArray,
                    "+" => Operation::Add,
                    "-" => Operation::Subtract,
                    "*" => Operation::Multiply,
                    "*opo" => Operation::MultiplyOpo,
                    "/" => Operation::Divide,
                    "mod" => Operation::Modulo,
                    "log" => Operation::Log,
                    "min" => Operation::Min,
                    "max" => Operation::Max,
                    "pow" => Operation::Pow,
                    "=" => Operation::Assign,
                    "mean" => Operation::Mean,
                    "abs" => Operation::Abs,
                    "*nm" => Operation::MultiplyNormalized,
                    "*nmabs" => Operation::MultiplyNormalizedAbs,
                    _ => {
                        error!("Unknown operation: {}", operation);
                        continue;
                    },
                };
                let mut operands = Vec::new();
                for operand in str_operands {
                    let operand = operand.trim();    
                    match (&operation, operand) {
                        (Operation::Abs, "") => {}
                        (Operation::Abs, _) => {warn!("{} has no effect on Abs operation", operand);}
                        (_, "") => { continue; }
                    _ => {}
                    }

                    let operand = if let Ok(value) = operand.parse::<f32>() {
                        Operand::Value(value)
                    } else if let Some(var_i) = operand.strip_prefix("$") {
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
                        let seed = seed_str.parse::<u64>().unwrap_or(0);
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
                    } else {
                        // Handle entity operand, possibly with '*' prefix for "opposited"
                        let (complement, operand_str) = if let Some(stripped) = operand.strip_prefix("COMP") {
                            (true, stripped)
                        } else {
                            (false, operand)
                        };

                        match terr_gen_map.0.get(&operand_str.to_string()) {
                            Ok(ent) => Operand::Entity(ent, complement as u64),
                            Err(_) => {
                                warn!("Entity not found in TerrGenEntityMap: {}", operand_str);
                                continue;
                            }
                        }
                    };

                    operands.push(operand);
                };
                oplist.trunk.push((operation, operands, *out));    
                    
            }
            
            oplist.tiles_over = seri.tiles_over
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

            oplist.tiles_under = seri.tiles_under
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

fn handle_bifurcation(
    cmd: &mut Commands,
    oplist_ent: Entity,
    bifurcation_str: &str,
    oplist_map: &OpListEntityMap,
    is_root: &Query<&MultipleDimensionStringRefs>,
    set_bifurcation: &mut dyn FnMut(Entity),
    id: &str,
    over_or_under: &str,
) {
    // Handle bifurcation string: either an OpList entity or a tile/tile sampler entity (ending with '*')
    let bifurcation_str = bifurcation_str.trim();
    if bifurcation_str.is_empty() { return; }


    let Ok(bifurcation_ent) = oplist_map.0.get(&bifurcation_str.to_string()) else {
        error!(
            "bifurcation_{} entity '{}' not found in OpListEntityMap",
            over_or_under, bifurcation_str
        );
        return;
    };
    if oplist_ent == bifurcation_ent {
        error!("bifurcation_{} cannot be the parent oplist '{}'", over_or_under, id);
        return;
    }
    if is_root.get(bifurcation_ent).is_ok() {
        error!("bifurcation_{} entity {} must not be a root oplist", over_or_under, bifurcation_str);
    } else {
        cmd.entity(bifurcation_ent).insert(ChildOf(oplist_ent));
        set_bifurcation(bifurcation_ent);
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
            handle_bifurcation(
                &mut cmd,
                oplist_ent,
                &seri.bifover,
                &oplist_map,
                &is_root,
                &mut |ent| oplist.bifurcation_over = Some(ent),
                &seri.id,
                "over",
            );
            handle_bifurcation(
                &mut cmd,
                oplist_ent,
                &seri.bifunder,
                &oplist_map,
                &is_root,
                &mut |ent| oplist.bifurcation_under = Some(ent),
                &seri.id,
                "under",
            );
        }
    }
    Ok(())
}



 