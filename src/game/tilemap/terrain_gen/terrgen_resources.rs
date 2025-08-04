use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use fastnoise_lite::FastNoiseLite;

use crate::game::{game_resources::GlobalEntityMap, tilemap::terrain_gen::terrgen_components::*};


#[derive(Resource, )]
pub struct WorldGenSettings {
    
    pub seed: i32,
    pub c_decrease_per_1km: f32,
    pub world_size: Option<u32>,
}
impl Default for WorldGenSettings {
    fn default() -> Self {
        Self { 
            seed: 0,
            c_decrease_per_1km: 15.0, //esto debería usarse para reducir o incrementar threshold
            world_size: None 
        }
    }
}

#[derive(Resource, Debug, Default )]
pub struct TerrGenEntityMap(pub HashMap<String, Entity>);

#[allow(unused_parens)]
impl TerrGenEntityMap {
    pub fn new_noise_ent_from_seri(
        &mut self, cmd: &mut Commands, seri: NoiseSerialization, 
    ) {
        if self.0.contains_key(&seri.id) {
            error!(target: "noise_loading", "NoiseSeri with id {:?} already exists in map, skipping", seri.id);
            return;
        }
        let min_id_length = 5;
        if seri.id.len() < min_id_length {
            error!(target: "noise_loading", "NoiseSeri id {} is too short or empty, skipping (min length: {})", seri.id, min_id_length);
            return;
        }
        
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
                    error!(target: "noise_loading", "Unknown noise type: {} for noise {}", noise_type, seri.id);
                    return;
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
                    error!(target: "noise_loading", "Unknown fractal type: {} for noise {}", fractal_type, seri.id);
                    return;
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
                    error!(target: "noise_loading", "Unknown cellular distance function: {} for noise {}", cellular_distance_function, seri.id);
                    return;
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
                    error!(target: "noise_loading", "Unknown cellular return type: {} for noise {}", cellular_return_type, seri.id);
                    return;
                }
            }));
        }
        if let Some(domain_warp_type) = seri.domain_warp_type {
            noise.set_domain_warp_type(Some(match domain_warp_type {
                0 => fastnoise_lite::DomainWarpType::OpenSimplex2,
                1 => fastnoise_lite::DomainWarpType::OpenSimplex2Reduced,
                2 => fastnoise_lite::DomainWarpType::BasicGrid,
                _ => {
                    error!(target: "noise_loading", "Unknown domain warp type: {} for noise {}", domain_warp_type, seri.id);
                    return;
                }
            }));
        }
        noise.set_cellular_jitter(seri.cellular_jitter);
        noise.set_domain_warp_amp(seri.domain_warp_amp);

        let noise_ent = cmd.spawn((
            Name::new(format!("Noise {}", seri.id)),
            TgenNoise::new(noise,),
        )).id();
        self.0.insert(seri.id.clone(), noise_ent);

    }
    
    pub fn get_entity<S: Into<String>>(&self, id: S) -> Option<Entity> { self.0.get(&id.into()).copied() }
    
    pub fn get_entities<I, S>(&self, ids: I) -> Vec<Entity> where I: IntoIterator<Item = S>, S: AsRef<str>, {
        ids.into_iter().filter_map(|id| self.0.get(id.as_ref()).copied()).collect()
    }
}

#[derive(AssetCollection, Resource)]
pub struct NoiseSerisHandles {
    #[asset(path = "ron/tilemap/terrgen/noise", collection(typed))]
    pub handles: Vec<Handle<NoiseSerialization>>,
}
#[derive(serde::Deserialize, Asset, TypePath, )]
pub struct NoiseSerialization {
    pub id: String,
    /// Default is 0.01
    pub frequency: Option<f32>,
    /// 0: OpenSimplex2, 1: OpenSimplex2S, 2: Cellular, 3: Perlin, 4: ValueCubic, 5: Value
    pub noise_type: Option<u32>,
    /// 0: None, 1: FBm, 2: Ridged, 3: PingPong, 4: DomainWarpProgressive, 5: DomainWarpIndependent,
    pub fractal_type: Option<u32>,
    /// Default is 3
    pub octaves: Option<u8>,
    /// Default is 2.0
    pub lacunarity: Option<f32>,
    /// Default is 0.5
    pub gain: Option<f32>,
    /// Default is 0.0
    pub weighted_strength: Option<f32>,
    /// Default is 2.0
    pub ping_pong_strength: Option<f32>,
    /// 0: Euclidean, 1: EuclideanSq, 2: Manhattan, 3: Hybrid
    pub cellular_distance_function: Option<u32>,
    /// 0: CellValue, 1: Distance, 2: Distance2, 3: Distance2Add, 4: Distance2Sub, 5: Distance2Mul, 6: Distance2Div
    pub cellular_return_type: Option<u32>,
    /// Default is 1.0
    pub cellular_jitter: Option<f32>,
    /// 0: OpenSimplex2, 1: OpenSimplex2Reduced, 2: BasicGrid
    pub domain_warp_type: Option<u32>,
    /// Default is 1.0
    pub domain_warp_amp: Option<f32>,
}


#[derive(Resource, Debug, Default )]
pub struct OpListEntityMap(pub HashMap<String, Entity>);

#[allow(unused_parens)]
impl OpListEntityMap {
    pub fn new_oplist_ent_from_seri(
        &mut self, cmd: &mut Commands, seri: &OpListSeri, terr_gen_map: &TerrGenEntityMap,
    ) {
        if self.0.contains_key(&seri.id) {
            error!(target: "oplist_loading", "OpListSeri with id {:?} already exists in map, skipping", seri.id);
            return;
        }
        if seri.id.len() <= 2 {
            error!(target: "oplist_loading", "OpListSeri id is too short or empty, skipping");
            return;
        }
        
        if seri.operations.is_empty() {
            error!(target: "oplist_loading", "OpListSeri id {:?} has no operations, skipping", seri.id);
            return;
        }
        if seri.operations.len() != seri.operands_per_op.len() {
            error!(target: "oplist_loading", "OpListSeri id {:?} has mismatched operations and operands, skipping", seri.id);
            return;
        }
        
        let mut oplist = OperationList::default();
        
        for (operation, operands) in seri.operations.iter().zip(seri.operands_per_op.iter()) {
            match operation.as_str() {
                "+" | "-" | "*" | "/" | "mod" | "log" | "min" | "max" | "pow" | "set" => {

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
                        "set" => Operation::Assign,
                        "intp" => Operation::Interpolate,
                        _ => panic!("Unexpected operation string encountered in OpListSeri: {}", operation),
                    };

                    // All these ops expect at least one operand
                    let operand_str = operands.get(0).cloned().unwrap_or_default();
                    let operand = if let Ok(value) = operand_str.parse::<f32>() {
                        Operand::Value(value)
                    } else if operand_str == "hp" {
                        Operand::HashPos
                    } else if operand_str == "pd" {
                        // PoissonDisk expects a value as second operand
                        if let Some(val_str) = operands.get(1) {
                            if let Ok(val) = val_str.parse::<u8>() && val > 0 {
                                Operand::new_poisson_disk(val)
                            } else {
                                error!(target: "oplist_loading", "Invalid PoissonDisk value: {}", val_str);
                                Operand::new_poisson_disk(1)
                            }
                        } else {
                            warn!(target: "oplist_loading", "Missing PoissonDisk value for operand 'pd'");
                            Operand::new_poisson_disk(1)
                        }
                    } else if let Some(first_entity) = terr_gen_map.get_entity(&operand_str) {
                        let mut op_entities = vec![first_entity];
                        for entity_str in operands.iter().skip(1) {
                            if let Some(ent) = terr_gen_map.get_entity(entity_str) {
                                op_entities.push(ent);
                            } else {
                                error!(target: "oplist_loading", "Entity {} not found in TerrGenEntityMap", entity_str);
                                return;
                            }
                        }
                        Operand::Entities(op_entities)
                    } else {
                        error!(target: "oplist_loading", "Unknown operand: {}", operand_str);
                        Operand::Value(0.0)
                    };
                    
                    
                    oplist.trunk.push((operand, op));
                }
                "tiles" => {
                    // Operation::GetTiles(ProducedTiles)
                }
                _ => {
                    error!(target: "oplist_loading", "Unknown operation: {}", operation);
                    continue;
                }
            }
        }
        let spawned_oplist = cmd.spawn((
            Name::new(format!("OpList {}", seri.id)),
            oplist,
        )).id();
        if seri.root {
            cmd.entity(spawned_oplist).insert(RootOpList);
        }

        self.0.insert(seri.id.clone(), spawned_oplist);
    }
    
    pub fn set_bifurcations(
        &mut self, cmd: &mut Commands, seri: OpListSeri, oplist: &mut OperationList,
    ) {
        
        if  ! seri.bifurcation_over.is_empty() {
            if let Some(bifurcation_over_ent) = self.get_entity(&seri.bifurcation_over) {
                oplist.bifurcation_over = Some(bifurcation_over_ent);

            } else {
                error!(target: "oplist_loading", "Bifurcation over entity {} not found in map", seri.bifurcation_over);
                return;
            }
        }
        if ! seri.bifurcation_under.is_empty() {
            if let Some(bifurcation_under_ent) = self.get_entity(&seri.bifurcation_under) {
                oplist.bifurcation_under = Some(bifurcation_under_ent);
            } else {
                error!(target: "oplist_loading", "Bifurcation under entity {} not found in map", seri.bifurcation_under);
                return;
            }
        }
         

      
    }
    
    pub fn get_entity<S: Into<String>>(&self, id: S) -> Option<Entity> { self.0.get(&id.into()).copied() }
    
    pub fn get_entities<I, S>(&self, ids: I) -> Vec<Entity> where I: IntoIterator<Item = S>, S: AsRef<str>, {
        ids.into_iter().filter_map(|id| self.0.get(id.as_ref()).copied()).collect()
    }
}

#[derive(AssetCollection, Resource)]
pub struct OpListSerisHandles {
    #[asset(path = "ron/oplist", collection(typed))]
    pub handles: Vec<Handle<OpListSeri>>,
}
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct OpListSeri {
    pub id: String,
    pub root: bool,
    pub operations: Vec<String>,
    pub operands_per_op: Vec<Vec<String>>,
    pub bifurcation_over: String,
    pub threshold: f32,
    pub bifurcation_under: String,
}