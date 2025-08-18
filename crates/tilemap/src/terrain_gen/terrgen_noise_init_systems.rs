


use bevy::prelude::*;

use fastnoise_lite::FastNoiseLite;

use common::common_components::{DisplayName, EntityPrefix, StrId};

use crate::terrain_gen::{terrgen_components::*, terrgen_resources::*};

use std::mem::take;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[allow(unused_parens)]
pub fn init_noises(
    mut cmd: Commands, 
    mut seris_handles: ResMut<NoiseSerisHandles>,
    mut assets: ResMut<Assets<NoiseSerialization>>,
    terrgen_map: Option<Res<TerrGenEntityMap>>,
) {
    if terrgen_map.is_some() { return; }
    cmd.insert_resource(TerrGenEntityMap::default());


    for handle in take(&mut seris_handles.handles) {
        let Some(seri) = assets.remove(&handle) else { continue };

        let str_id = match StrId::new(seri.id.clone(), 3) {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to create StrId for noise {}: {}", seri.id, e);
                continue;
            }    
        };

        let mut noise = FastNoiseLite::new();
        noise.set_frequency(seri.frequency);

        let mut hasher = DefaultHasher::new();
        seri.id.hash(&mut hasher);
        let seed = hasher.finish() as i32;
        noise.set_seed(Some(seed));

        if let Some(noise_type) = seri.noise_type {
            noise.set_noise_type(Some(match noise_type {
                0 => fastnoise_lite::NoiseType::OpenSimplex2,
                1 => fastnoise_lite::NoiseType::OpenSimplex2S,
                2 => fastnoise_lite::NoiseType::Cellular,
                3 => fastnoise_lite::NoiseType::Perlin,
                4 => fastnoise_lite::NoiseType::ValueCubic,
                5 => fastnoise_lite::NoiseType::Value,
                _ => {
                    error!("Unknown noise type: {} for noise {}", noise_type, seri.id);
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
                    error!("Unknown fractal type: {} for noise {}", fractal_type, seri.id);
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
                    error!("Unknown cellular distance function: {} for noise {}", cellular_distance_function, seri.id);
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
                    error!("Unknown cellular return type: {} for noise {}", cellular_return_type, seri.id);   
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
                    error!("Unknown domain warp type: {} for noise {}", domain_warp_type, seri.id);
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
}

#[allow(unused_parens)]
pub fn add_noises_to_map(
    mut cmd: Commands, 
    terrgen_map: Option<ResMut<TerrGenEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<StrId>, With<FnlNoise>)>,
) {
    let Some(mut terrgen_map) = terrgen_map else {
        return;
    };
    for (ent, prefix, str_id) in query.iter() {
        if let Err(err) = terrgen_map.0.insert(str_id, ent, ) {
            error!("{} {} already in TerrGenEntityMap : {}", prefix, str_id, err);
            cmd.entity(ent).despawn();
        }
    }
    
}

 

