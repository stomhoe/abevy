use bevy::prelude::*;
use fnl::*;
use common::common_components::{Prefix, StrId};
use tilemap_shared::GlobalGenSettings;
use crate::terrain::{terrgen_components::*, terrgen_resources::*, TerrgenEntityMap};

#[allow(unused_parens)]
pub fn init_noises(
    mut cmd: Commands,
    terrgen_map: Res<TerrgenEntityMap>,
    mut settings: Query<&mut GlobalGenSettings>,
    noise_holder: Query<Entity, With<EguiTerrgensHolder>>,
) {
    let settings_from_defs = load_global_gen_settings_seri_defs()
        .into_iter()
        .next()
        .map(|seri| seri.to_global_gen_settings());
    if settings.is_empty() {
        let settings_to_spawn = settings_from_defs.clone().unwrap_or_default();
        cmd.spawn((settings_to_spawn, Prefix::trunc("AA_GLOBAL_GEN_SETTINGS")));
    } else if let Some(settings_from_defs) = settings_from_defs {
        for mut existing_settings in &mut settings {
            *existing_settings = settings_from_defs.clone();
        }
    }
    info!(target: "terrgen_init", "Loaded Global Gen Settings");

    if !terrgen_map.0.is_empty() { return; }

    let mut fnl_comps_to_insert = Vec::new();

    let holder = if noise_holder.is_empty() {
        cmd.spawn((EguiTerrgensHolder,)).id()
    } else {
        noise_holder.single().unwrap()
    };

    for seri in load_fnl_seri_defs() {

        let str_id = match StrId::new_with_result(seri.id.clone(), 3) {
            Ok(id) => id,
            Err(e) => {
                error!(target: "terrgen_init", "Failed to create StrId for noise {}: {}", seri.id, e);
                continue;
            }
        };
        let mut noise = FastNoiseLite::new(str_id.clone());

        if seri.frequency < 0.00000000001 {
            error!(target: "terrgen_init", "Frequency is too small (< 0.0001) for noise {}", seri.id);
        }
        noise.set_frequency(Some(seri.frequency));

        let noise_type = match seri.noise_type {
            0 => NoiseType::OpenSimplex2,
            1 => NoiseType::OpenSimplex2S,
            2 => NoiseType::Cellular,
            3 => NoiseType::Perlin,
            4 => NoiseType::ValueCubic,
            5 => NoiseType::Value,
            _ => {
                error!(target: "terrgen_init", "Unknown noise type: {} for noise {}", seri.noise_type, seri.id);
                continue;
            }
        };
        noise.set_noise_type(Some(noise_type));
        let fractal_type = match seri.fractal_type {
            0 => FractalType::None,
            1 => FractalType::FBm,
            2 => FractalType::Ridged,
            3 => FractalType::PingPong,
            4 => FractalType::DomainWarpProgressive,
            5 => FractalType::DomainWarpIndependent,
            _ => {
                error!(target: "terrgen_init", "Unknown fractal type: {} for noise {}", seri.fractal_type, seri.id);
                continue;
            }
        };
        noise.set_fractal_type(Some(fractal_type));
        noise.set_fractal_octaves(Some(seri.octaves as i32));
        noise.set_fractal_lacunarity(Some(seri.lacunarity));
        noise.set_fractal_gain(Some(seri.gain));
        noise.set_fractal_weighted_strength(Some(seri.weighted_strength));
        noise.set_fractal_ping_pong_strength(Some(seri.ping_pong_strength));
        let cellular_distance_function = match seri.cellular_distance_function {
            0 => CellularDistanceFunction::Euclidean,
            1 => CellularDistanceFunction::EuclideanSq,
            2 => CellularDistanceFunction::Manhattan,
            3 => CellularDistanceFunction::Hybrid,
            _ => {
                error!(target: "terrgen_init", "Unknown cellular distance function: {} for noise {}", seri.cellular_distance_function, seri.id);
                continue;
            }
        };
        noise.set_cellular_distance_function(Some(cellular_distance_function));
        let cellular_return_type = match seri.cellular_return_type {
            0 => CellularReturnType::CellValue,
            1 => CellularReturnType::Distance,
            2 => CellularReturnType::Distance2,
            3 => CellularReturnType::Distance2Add,
            4 => CellularReturnType::Distance2Sub,
            5 => CellularReturnType::Distance2Mul,
            6 => CellularReturnType::Distance2Div,
            _ => {
                error!(target: "terrgen_init", "Unknown cellular return type: {} for noise {}", seri.cellular_return_type, seri.id);
                continue;
            }
        };
        noise.set_cellular_return_type(Some(cellular_return_type));
        let domain_warp_type = match seri.domain_warp_type {
            0 => DomainWarpType::OpenSimplex2,
            1 => DomainWarpType::OpenSimplex2Reduced,
            2 => DomainWarpType::BasicGrid,
            _ => {
                error!(target: "terrgen_init", "Unknown domain warp type: {} for noise {}", seri.domain_warp_type, seri.id);
                continue;
            }
        };
        noise.set_domain_warp_type(Some(domain_warp_type));
        noise.set_cellular_jitter(Some(seri.cellular_jitter));
        noise.set_domain_warp_amp(Some(seri.domain_warp_amp));

        if let Ok(existing) = terrgen_map.0.get_cloned(&str_id) {
            error!(target: "terrgen_init", "{} already in TerrgenEntityMap : {:?}", str_id, existing);
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
