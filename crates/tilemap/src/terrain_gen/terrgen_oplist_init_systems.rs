

use core::panic;

use bevy::prelude::*;

use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use dimension::dimension_components::MultipleDimensionStringRefs;
use fastnoise_lite::FastNoiseLite;

use common::common_components::{DisplayName, EntityPrefix, StrId};

use crate::{chunking_components::*, terrain_gen::{terrgen_components::*, terrgen_resources::*}, tile::{tile_resources::*, tile_samplers_resources::TileWeightedSamplersMap}};

use std::mem::take;
use std::hash::{Hash, Hasher};

#[allow(unused_parens)]
pub fn init_oplists_from_assets(
    mut cmd: Commands, seris_handles: Res<OpListSerisHandles>,
    mut assets: ResMut<Assets<OpListSerialization>>, 
    terr_gen_map: Res<TerrGenEntityMap>,  tiling_map: Res<TileWeightedSamplersMap>,
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
            oplist.threshold = seri.threshold;

            for (operation, operands) in &seri.operation_operands {

                let op = match operation.as_str().trim() {
                    "" => continue,
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

                // All these ops expect at least one operand
                let operand_str = operands.get(0).cloned().unwrap_or_default();

                if operand_str.is_empty() {
                    warn!("Empty operand for operation '{}', skipping", operation);
                    continue;
                }
                
                let operand = if let Ok(value) = operand_str.parse::<f32>() {
                    Operand::Value(value)
                } else if operand_str == "hp" {
                    Operand::HashPos
                } else if operand_str == "pd" {
                    // PoissonDisk expects a value as second operand
                    let Some(val_str) = operands.get(1) else {
                        warn!("Missing PoissonDisk value for operand 'pd'");
                        continue;
                    };
                    let Ok(val) = val_str.parse::<u8>() else {
                        warn!("Invalid PoissonDisk value: {}", val_str);
                        continue;
                    };
                    let seed = if let Some(seed_str) = operands.get(2) {
                        match seed_str.parse::<u64>() {
                            Ok(seed) => seed,
                            Err(_) => {
                                warn!("Invalid PoissonDisk seed: {}", seed_str);
                                0
                            }
                        }
                    } else {
                        0
                    };
                    match Operand::new_poisson_disk(val, seed) {
                        Ok(op) => op,
                        Err(e) => {
                            warn!("Failed to create PoissonDisk operand: {}", e);
                            continue;
                        }
                    }
                } else if let Ok(first_entity) = terr_gen_map.0.get(&operand_str) {
                    let mut op_entities = vec![first_entity];
                    for entity_str in operands.iter().skip(1) {
                        if entity_str.is_empty() {
                            continue;
                        }
                        let Ok(ent) = terr_gen_map.0.get(entity_str) else {
                            warn!("Invalid entity operand: {}", entity_str);
                            continue;
                        };
                        op_entities.push(ent);
                    }
                    Operand::Entities(op_entities)
                } else {
                    warn!("Invalid operand string: {}", operand_str);
                    continue;
                };

                oplist.trunk.push((operand, op));    

            }
            let produced_tiles: ProducedTiles = ProducedTiles::new(
                seri.tiles
                    .iter()
                    .filter(|tile_str| !tile_str.is_empty())
                    .filter_map(|tile_str| {
                        match tiling_map.0.get(tile_str) {
                            Ok(tile_ent) => Some(tile_ent),
                            Err(_) => {
                                warn!("Tile {} not found in TilingEntityMap", tile_str);
                                None
                            }
                        }
                    })
                    .collect::<Vec<Entity>>()
            );

            let spawned_oplist = cmd.spawn(( str_id, oplist, produced_tiles, size)).id();
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
    tile_ents_map: Res<TileEntitiesMap>,
    mut oplist_query: Query<(&mut OperationList, Has<MultipleDimensionStringRefs>)>,
) -> Result {
    for handle in take(&mut seris_handles.handles) {
        if let Some(seri) = assets.remove(&handle) {
            let oplist_ent = oplist_map.0.get(&seri.id)?;

            if ! seri.bifurcation_over.is_empty() {
                let bifurcation_over_ent = oplist_map.0.get(&seri.bifurcation_over)?; 

                if oplist_ent == bifurcation_over_ent {
                    error!("bifurcation_over cannot be the parent oplist '{}'", seri.id);
                    continue;
                }
                match oplist_query.get(bifurcation_over_ent)? {
                    (_, false) => {
                        cmd.entity(bifurcation_over_ent).insert(ChildOf(oplist_ent));
                        let (mut oplist, _) = oplist_query.get_mut(oplist_ent)?;
                        oplist.bifurcation_over = Some(bifurcation_over_ent);
                    }
                    (_, true) => {
                        error!("bifurcation_over entity {} must not be a root oplist", seri.bifurcation_over);
                        continue;
                    }
                }
            }
            if !seri.bifurcation_under.is_empty() {
                let bifurcation_under_ent = oplist_map.0.get(&seri.bifurcation_under)?;
                
                if oplist_ent == bifurcation_under_ent {
                    error!("bifurcation_under cannot be the the parent oplist '{}'", seri.id.clone());
                    continue;
                }    match oplist_query.get(bifurcation_under_ent)? {
                    (_, false) => {
                        cmd.entity(bifurcation_under_ent).insert(ChildOf(oplist_ent));
                        let (mut oplist, _) = oplist_query.get_mut(oplist_ent)?;
                        oplist.bifurcation_under = Some(bifurcation_under_ent);
                    }
                    (_, true) => {
                        error!("bifurcation_under entity {} must not be a root oplist", seri.bifurcation_under);    
                        continue;
                    }
                }
               
            }
        }
    }
    Ok(())
}





 

