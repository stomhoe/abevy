

use core::panic;

use bevy::prelude::*;

use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use fastnoise_lite::FastNoiseLite;

use common::common_components::{DisplayName, EntityPrefix, StrId};

use crate::{chunking_components::*, terrain_gen::{terrgen_components::*, terrgen_resources::*}, tile::tile_resources::*};

use std::mem::take;

#[allow(unused_parens)]
pub fn init_noises(
    mut cmd: Commands, 
    mut seris_handles: ResMut<NoiseSerisHandles>,
    mut assets: ResMut<Assets<NoiseSerialization>>,
    terrgen_map: Option<Res<TerrGenEntityMap>>,
) -> Result {
    if terrgen_map.is_some() { return Ok(()); }
    cmd.init_resource::<TerrGenEntityMap>();
    let mut result: Result = Ok(());

    for handle in take(&mut seris_handles.handles) {
        let Some(seri) = assets.remove(&handle) else { continue };

        let str_id = match StrId::new(seri.id.clone()) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for noise {}: {}", seri.id, e));
                error!(target: "noise_loading", "{}", err);
                result = Err(err);
                continue;
            }
        };

        let mut noise = FastNoiseLite::new();
        noise.set_frequency(seri.frequency);
        
        if let Some(noise_type) = seri.noise_type {
            noise.set_noise_type(Some(match noise_type {
                0 => fastnoise_lite::NoiseType::OpenSimplex2,
                1 => fastnoise_lite::NoiseType::OpenSimplex2S,
                2 => fastnoise_lite::NoiseType::Cellular,
                3 => fastnoise_lite::NoiseType::Perlin,
                4 => fastnoise_lite::NoiseType::ValueCubic,
                5 => fastnoise_lite::NoiseType::Value,
                _ => {
                    let err = BevyError::from(format!("Unknown noise type: {} for noise {}", noise_type, seri.id));
                    error!(target: "noise_loading", "{}", err);
                    result = Err(err);
                    continue;
                }
            }));
        }
        if let Some(fractal_type) = seri.fractal_type {
            noise.set_fractal_type(Some(match fractal_type {
                0 => fastnoise_lite::FractalType::None,
                1 => fastnoise_lite::FractalType::FBm,
                2 => fastnoise_lite::FractalType::Ridged,
                3 => fastnoise_lite::FractalType::PingPong,
                4 => fastnoise_lite::FractalType::DomainWarpProgressive,
                5 => fastnoise_lite::FractalType::DomainWarpIndependent,
                _ => {
                    let err = BevyError::from(format!("Unknown fractal type: {} for noise {}", fractal_type, seri.id));
                    error!(target: "noise_loading", "{}", err);
                    result = Err(err);
                    continue;
                }
            }));
        }
        noise.set_fractal_octaves(Some(seri.octaves.unwrap_or(3) as i32));
        noise.set_fractal_lacunarity(seri.lacunarity);
        noise.set_fractal_gain(seri.gain);
        noise.set_fractal_weighted_strength(seri.weighted_strength);
        noise.set_fractal_ping_pong_strength(seri.ping_pong_strength);
        if let Some(cellular_distance_function) = seri.cellular_distance_function {
            noise.set_cellular_distance_function(Some(match cellular_distance_function {
                0 => fastnoise_lite::CellularDistanceFunction::Euclidean,
                1 => fastnoise_lite::CellularDistanceFunction::EuclideanSq,
                2 => fastnoise_lite::CellularDistanceFunction::Manhattan,
                3 => fastnoise_lite::CellularDistanceFunction::Hybrid,
                _ => {
                    let err = BevyError::from(format!("Unknown cellular distance function: {} for noise {}", cellular_distance_function, seri.id));
                    error!(target: "noise_loading", "{}", err);
                    result = Err(err);
                    continue;
                }
            }));
        }
        
        if let Some(cellular_return_type) = seri.cellular_return_type {
            noise.set_cellular_return_type(Some(match cellular_return_type {
                0 => fastnoise_lite::CellularReturnType::CellValue,
                1 => fastnoise_lite::CellularReturnType::Distance,
                2 => fastnoise_lite::CellularReturnType::Distance2,
                3 => fastnoise_lite::CellularReturnType::Distance2Add,
                4 => fastnoise_lite::CellularReturnType::Distance2Sub,
                5 => fastnoise_lite::CellularReturnType::Distance2Mul,
                6 => fastnoise_lite::CellularReturnType::Distance2Div,
                _ => {
                    let err = BevyError::from(format!("Unknown cellular return type: {} for noise {}", cellular_return_type, seri.id));
                    error!(target: "noise_loading", "{}", err);
                    result = Err(err);
                    continue;
                }
            }));
        }
        if let Some(domain_warp_type) = seri.domain_warp_type {
            noise.set_domain_warp_type(Some(match domain_warp_type {
                0 => fastnoise_lite::DomainWarpType::OpenSimplex2,
                1 => fastnoise_lite::DomainWarpType::OpenSimplex2Reduced,
                2 => fastnoise_lite::DomainWarpType::BasicGrid,
                _ => {
                    let err = BevyError::from(format!("Unknown domain warp type: {} for noise {}", domain_warp_type, seri.id));
                    error!(target: "noise_loading", "{}", err);
                    result = Err(err);
                    continue;
                }
            }));
        }
        noise.set_cellular_jitter(seri.cellular_jitter);
        noise.set_domain_warp_amp(seri.domain_warp_amp);

        cmd.spawn((
            str_id,
            DisplayName::new(seri.id.clone()),
            FnlNoise::new(noise,),
        ));

    }
    result
}

#[allow(unused_parens)]
pub fn add_noises_to_map(
    terrgen_map: Option<ResMut<TerrGenEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<StrId>, With<FnlNoise>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut terrgen_map) = terrgen_map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = terrgen_map.0.insert(str_id, ent, ) {
                error!(target: "noise_loading", "{} {} already in TerrGenEntityMap : {}", prefix, str_id, err);
                result = Err(err);
            }
        }
    }
    result
}

#[allow(unused_parens)]
pub fn init_oplists_from_assets(
    mut cmd: Commands, seris_handles: Res<OpListSerisHandles>,
    assets: Res<Assets<OpListSerialization>>, 
    terr_gen_map: Res<TerrGenEntityMap>,  tiling_map: Res<TilingEntityMap>,
    oplist_map: Option<Res<OpListEntityMap>>,
) -> Result {
    if oplist_map.is_some() { return Ok(()); }
    cmd.init_resource::<OpListEntityMap>();

    let mut result: Result = Ok(());
    for handle in seris_handles.handles.iter() {//ESTE VA CON ITER
        if let Some(seri) = assets.get(handle) {
            //info!(target: "oplist_loading", "Loading OpListSeri from handle: {:?}", handle);
            let str_id = StrId::new(seri.id.clone())?;

            if seri.root && seri.operation_operands.is_empty() {
                result = Err(BevyError::from("root OpListSeri has no operations"));
                error!(target: "oplist_loading", "{}", result.as_ref().unwrap_err());
            }    
            let mut oplist = OperationList::default();
            oplist.threshold = seri.threshold;
            
            for (operation, operands) in &seri.operation_operands {
                match operation.as_str() {
                    "+" | "-" | "*" | "/" | "mod" | "log" | "min" | "max" | "pow" | "=" => {

                        let op = match operation.as_str() {
                            "+" => Operation::Add,
                            "-" => Operation::Subtract,
                            "*" => Operation::Multiply,
                            "/" => Operation::Divide,
                            "mod" => Operation::Modulo,
                            "log" => Operation::Log,
                            "min" => Operation::Min,
                            "max" => Operation::Max,
                            "pow" => Operation::Pow,
                            "=" => Operation::Assign,
                            "mean" => Operation::Mean,
                            _ => {
                                error!(target: "oplist_loading", "Unknown operation: {}", operation);
                                panic!("This point shouldn't be reached: {}", operation);
                            },
                        };

                        // All these ops expect at least one operand
                        let operand_str = operands.get(0).cloned().unwrap_or_default();
                        let operand = if let Ok(value) = operand_str.parse::<f32>() {
                            Operand::Value(value)
                        } else if operand_str == "hp" {
                            Operand::HashPos
                        } else if operand_str == "pd" {
                            // PoissonDisk expects a value as second operand
                            let Some(val_str) = operands.get(1) else {
                                warn!(target: "oplist_loading", "Missing PoissonDisk value for operand 'pd'");
                                continue;
                            };
                            let Ok(val) = val_str.parse::<u8>() else {
                                warn!(target: "oplist_loading", "Invalid PoissonDisk value: {}", val_str);
                                continue;
                            };
                            let seed = if let Some(seed_str) = operands.get(2) {
                                match seed_str.parse::<u64>() {
                                    Ok(seed) => seed,
                                    Err(_) => {
                                        warn!(target: "oplist_loading", "Invalid PoissonDisk seed: {}", seed_str);
                                        0
                                    }
                                }
                            } else {
                                0
                            };
                            match Operand::new_poisson_disk(val, seed) {
                                Ok(op) => op,
                                Err(e) => {
                                    warn!(target: "oplist_loading", "Failed to create PoissonDisk operand: {}", e);
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
                                    warn!(target: "oplist_loading", "Invalid entity operand: {}", entity_str);
                                    continue;
                                };
                                op_entities.push(ent);
                            }
                            Operand::Entities(op_entities)
                        } else {
                            warn!(target: "oplist_loading", "Invalid operand string: {}", operand_str);
                            continue;
                        };

                        oplist.trunk.push((operand, op));    
                    }
                    _ => {
                        warn!(target: "oplist_loading", "Unknown operation: {}", operation);
                        continue;
                    }
                }
            }
            let mut produced_tiles: ProducedTiles = ProducedTiles::default();
            for tile_str in &seri.tiles {
                if tile_str.is_empty() { continue; }
                else if let Ok(tile_ent) = tiling_map.0.get(tile_str) {
                    produced_tiles.push(tile_ent);
                } else {
                    warn!(target: "oplist_loading", "Tile {} not found in TilingEntityMap", tile_str);
                }
            }

            let spawned_oplist = cmd.spawn(( str_id, oplist, produced_tiles, )).id();
            if seri.root { cmd.entity(spawned_oplist).insert(RootOpList); }
        } 
    }
    result
} 

#[allow(unused_parens)]
pub fn add_oplists_to_map(
    oplist_map: Option<ResMut<OpListEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId, ),(Added<StrId>, With<OperationList>)>,) 
-> Result {

    let mut result: Result = Ok(());
    if let Some(mut oplist_map) = oplist_map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = oplist_map.0.insert(str_id, ent, ) {

                error!(target: "oplist_loading", "{} {} already in OpListEntityMap : {}", prefix, str_id, err);
                result = Err(err);
            }
        }
    }
    result
}

#[allow(unused_parens)]
pub fn init_oplists_bifurcations(
    mut cmd: Commands,
    mut seris_handles: ResMut<OpListSerisHandles>,
    mut assets: ResMut<Assets<OpListSerialization>>, map: Res<OpListEntityMap>,
    mut oplist_query: Query<(&mut OperationList, Option<&RootOpList>)>,
) -> Result {
    let mut result: Result = Ok(());

    for handle in take(&mut seris_handles.handles) {
        if let Some(seri) = assets.remove(&handle) {
            let oplist_ent = map.0.get(&seri.id)?;

            if ! seri.bifurcation_over.is_empty() {
                let bifurcation_over_ent = map.0.get(&seri.bifurcation_over)?; 

                if oplist_ent == bifurcation_over_ent {
                    result = Err(BevyError::from(format!(
                        "bifurcation_over cannot be the parent oplist '{}'", seri.id
                    )));    
                    error!(target: "oplist_loading", "{}", result.as_ref().unwrap_err());
                    continue;
                }
                match oplist_query.get(bifurcation_over_ent)? {
                    (_, None) => {
                        cmd.entity(bifurcation_over_ent).insert(ChildOf(oplist_ent));
                        let (mut oplist, _) = oplist_query.get_mut(oplist_ent)?;
                        oplist.bifurcation_over = Some(bifurcation_over_ent);
                    }
                    (_, Some(_)) => {
                        result = Err(BevyError::from(format!(
                            "bifurcation_over entity {} must not be a root oplist", seri.bifurcation_over
                        )));
                        error!(target: "oplist_loading", "{}", result.as_ref().unwrap_err());
                        continue;
                    }
                }
            }
            if !seri.bifurcation_under.is_empty() {
                let bifurcation_under_ent = map.0.get(&seri.bifurcation_under)?;
                
                if oplist_ent == bifurcation_under_ent {
                    result = Err(BevyError::from(format!(
                        "bifurcation_under cannot be the the parent oplist '{}'", seri.id.clone()
                    )));
                    error!(target: "oplist_loading", "{}", result.as_ref().unwrap_err());
                    continue;
                }
                match oplist_query.get(bifurcation_under_ent)? {
                    (_, None) => {
                        cmd.entity(bifurcation_under_ent).insert(ChildOf(oplist_ent));
                        let (mut oplist, _) = oplist_query.get_mut(oplist_ent)?;
                        oplist.bifurcation_under = Some(bifurcation_under_ent);
                    }
                    (_, Some(_)) => {
                        result = Err(BevyError::from(format!(
                            "bifurcation_under entity {} must not be a root oplist", seri.bifurcation_under
                        )));
                        error!(target: "oplist_loading", "{}", result.as_ref().unwrap_err());
                        continue;
                    }
                }
               
            }
        }
    }
    result
}





 

