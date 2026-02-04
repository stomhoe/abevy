use bevy::prelude::*;
use fnl::*;
use common::common_components::{DisplayName, Prefix, StrId};
use tilemap_shared::GlobalGenSettings;
use crate::terrain_gen::{terrgen_components::*, terrgen_resources::*, TerrGenEntityMap};
use std::mem::take;

#[allow(unused_parens)]
pub fn init_noises(
    mut cmd: Commands, 
    mut seris_handles: ResMut<NoiseSerisHandles>,
    mut assets: ResMut<Assets<NoiseSerialization>>,
    terrgen_map: Res<TerrGenEntityMap>,
    settings: Query<&GlobalGenSettings>,
    noise_holder: Query<Entity, With<EguiNoiseHolder>>,
) {
    if !terrgen_map.0.is_empty() { return; }

    let mut fnl_comps_to_insert = Vec::new();
    
    if settings.is_empty(){
        cmd.spawn((GlobalGenSettings::default(), Prefix::trunc("AA_GLOBAL_GEN_SETTINGS")));
    }
    info!(target: "terrgen_init", "Spawning Global Gen Settings entity");

    let holder = if noise_holder.is_empty() {
        cmd.spawn((EguiNoiseHolder,)).id()
    } else {
        noise_holder.single().unwrap()
    };

    for handle in take(&mut seris_handles.handles) {
        let Some(seri) = assets.remove(&handle) else { continue };

        let str_id = match StrId::new_with_result(seri.id.clone(), 3) {
            Ok(id) => id,
            Err(e) => {
                error!(target: "terrgen_init", "Failed to create StrId for noise {}: {}", seri.id, e);
                continue;
            }    
        };
        let mut noise = FastNoiseLite::new(str_id.clone());
        
        if let Some(frequency) = seri.frequency {
            if frequency < 0.00000000001 {
                error!(target: "terrgen_init", "Frequency is too small (< 0.0001) for noise {}", seri.id);
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
                    error!(target: "terrgen_init", "Unknown noise type: {} for noise {}", noise_type, seri.id);
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
                    error!(target: "terrgen_init", "Unknown fractal type: {} for noise {}", fractal_type, seri.id);
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
                    error!(target: "terrgen_init", "Unknown cellular distance function: {} for noise {}", cellular_distance_function, seri.id);
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
                    error!(target: "terrgen_init", "Unknown cellular return type: {} for noise {}", cellular_return_type, seri.id);   
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
                    error!(target: "terrgen_init", "Unknown domain warp type: {} for noise {}", domain_warp_type, seri.id);
                    continue;
                }
            }));
        }
        noise.set_cellular_jitter(seri.cellular_jitter);
        noise.set_domain_warp_amp(seri.domain_warp_amp);

        if let Ok(existing) = terrgen_map.0.get_cloned(&str_id) {
            error!(target: "terrgen_init", "{} already in TerrGenEntityMap : {:?}", str_id, existing);
            continue;
        }
        let noise_ent = cmd.spawn_empty().id();
        fnl_comps_to_insert.push((
            noise_ent,
            (
                str_id.clone(),
                FnlNoiseComp(noise),
                ChildOf(holder),
            ),
        ));
    }
    cmd.insert_batch(fnl_comps_to_insert);
}

