use bevy::prelude::*;
use fnl::*;
use common::common_components::{DisplayName, EntityPrefix, StrId};
use tilemap_shared::GlobalGenSettings;
use crate::terrain_gen::{terrgen_components::*, terrgen_resources::*};
use std::mem::take;

#[allow(unused_parens)]
pub fn init_noises(
    mut cmd: Commands, 
    mut seris_handles: ResMut<NoiseSerisHandles>,
    mut assets: ResMut<Assets<NoiseSerialization>>,
    terrgen_map: Option<Res<TerrGenEntityMap>>,
) {
    if terrgen_map.is_some() { return; }

    let mut terrgen_map = TerrGenEntityMap::default();
    let mut fnl_comps_to_insert = Vec::new();
    
    cmd.spawn((GlobalGenSettings::default(), EntityPrefix::new_truncated("AA_GLOBAL_GEN_SETTINGS")));
    info!("Spawning Global Gen Settings entity");

    let holder = cmd.spawn((NoiseHolder,)).id();

    for handle in take(&mut seris_handles.handles) {
        let Some(seri) = assets.remove(&handle) else { continue };

        let str_id = match StrId::new_with_result(seri.id.clone(), 3) {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to create StrId for noise {}: {}", seri.id, e);
                continue;
            }    
        };
        let mut noise = FastNoiseLite::new(str_id.clone());
        
        if let Some(frequency) = seri.frequency {
            if frequency < 0.00000000001 {
                error!("Frequency is too small (< 0.0001) for noise {}", seri.id);
            }
        }
        noise.set_frequency(seri.frequency);

        if let Some(noise_type) = seri.noise_type {
            noise.set_noise_type(Some(match noise_type {
                0 => NoiseType::OpenSimplex2,
                1 => NoiseType::OpenSimplex2S,
                2 => NoiseType::Cellular,
                3 => NoiseType::Perlin,
                4 => NoiseType::ValueCubic,
                5 => NoiseType::Value,
                _ => {
                    error!("Unknown noise type: {} for noise {}", noise_type, seri.id);
                    continue;
                }
            }));
        }
        if let Some(fractal_type) = seri.fractal_type {
            noise.set_fractal_type(Some(match fractal_type {
                0 => FractalType::None,
                1 => FractalType::FBm,
                2 => FractalType::Ridged,
                3 => FractalType::PingPong,
                4 => FractalType::DomainWarpProgressive,
                5 => FractalType::DomainWarpIndependent,
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
                0 => CellularDistanceFunction::Euclidean,
                1 => CellularDistanceFunction::EuclideanSq,
                2 => CellularDistanceFunction::Manhattan,
                3 => CellularDistanceFunction::Hybrid,
                _ => {
                    error!("Unknown cellular distance function: {} for noise {}", cellular_distance_function, seri.id);
                    continue;
                }
            }));
        }
        
        if let Some(cellular_return_type) = seri.cellular_return_type {
            noise.set_cellular_return_type(Some(match cellular_return_type {
                0 => CellularReturnType::CellValue,
                1 => CellularReturnType::Distance,
                2 => CellularReturnType::Distance2,
                3 => CellularReturnType::Distance2Add,
                4 => CellularReturnType::Distance2Sub,
                5 => CellularReturnType::Distance2Mul,
                6 => CellularReturnType::Distance2Div,
                _ => {
                    error!("Unknown cellular return type: {} for noise {}", cellular_return_type, seri.id);   
                    continue;
                }
            }));
        }
        if let Some(domain_warp_type) = seri.domain_warp_type {
            noise.set_domain_warp_type(Some(match domain_warp_type {
                0 => DomainWarpType::OpenSimplex2,
                1 => DomainWarpType::OpenSimplex2Reduced,
                2 => DomainWarpType::BasicGrid,
                _ => {
                    error!("Unknown domain warp type: {} for noise {}", domain_warp_type, seri.id);
                    continue;
                }
            }));
        }
        noise.set_cellular_jitter(seri.cellular_jitter);
        noise.set_domain_warp_amp(seri.domain_warp_amp);

        if let Ok(existing) = terrgen_map.0.get(&str_id) {
            error!("{} already in TerrGenEntityMap : {:?}", str_id, existing);
            continue;
        }
        let noise_ent = cmd.spawn_empty().id();
        terrgen_map.0.overwrite(&str_id, noise_ent);
        fnl_comps_to_insert.push((
            noise_ent,
            (
                str_id.clone(),
                DisplayName::new(seri.id.clone()),
                FnlNoiseComp(noise),
                ChildOf(holder),
            ),
        ));
    }
    cmd.insert_resource(terrgen_map);
    cmd.insert_batch(fnl_comps_to_insert);
}



 

