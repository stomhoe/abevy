use bevy::prelude::*;
use ::fnl::*;
use common::common_components::{AddHashIdFromStrId, Prefix, StrId};
use tilemap_shared::GlobalGenSettings;
use crate::terrain::{
    TerrgenEntityMap,
    terrgen_components::*,
    terrgen_resources::*,
    terrgen_seris::*,
};

#[allow(unused_parens)]
pub fn init_noises(
    mut cmd: Commands,
    terrgen_map: Res<TerrgenEntityMap>,
    mut settings: Query<&mut GlobalGenSettings>,
    noise_holder: Query<Entity, With<EguiTerrgensHolder>>,
) {
    let settings_from_defs = load_terrgen_settings_seri_defs()
        .into_iter()
        .next()
        .map(|seri| seri.to_terrgen_settings());
    if settings.is_empty() {
        let settings_to_spawn = settings_from_defs.clone().unwrap_or_default();
        cmd.spawn((settings_to_spawn, Prefix::trunc("AA_GLOBAL_GEN_SETTINGS")));
    } else if let Some(settings_from_defs) = settings_from_defs {
        for mut existing_settings in &mut settings {
            *existing_settings = settings_from_defs.clone();
        }
    }
    trace!(target: "terrgen_init", "Loaded Global Gen Settings");

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

        let noise_type = match seri.noise_type.trim() {
            "OpenSimplex2" => NoiseType::OpenSimplex2,
            "OpenSimplex2S" => NoiseType::OpenSimplex2S,
            "Cellular" => NoiseType::Cellular,
            "Perlin" => NoiseType::Perlin,
            "ValueCubic" => NoiseType::ValueCubic,
            "Value" => NoiseType::Value,
            "ValueLan" | "ValueLanczos" => NoiseType::ValueLanczos,
            "River" | "RiverFlow" => NoiseType::River,
            other => {
                error!(target: "terrgen_init", "Unknown noise_type '{other}' for noise {}", seri.id);
                continue;
            }
        };
        let fractal_type = match seri.fractal_type.trim() {
            "None" => FractalType::None,
            "FBm" => FractalType::FBm,
            "Ridged" => FractalType::Ridged,
            "PingPong" => FractalType::PingPong,
            "DomainWarpProgressive" => FractalType::DomainWarpProgressive,
            "DomainWarpIndependent" => FractalType::DomainWarpIndependent,
            other => {
                error!(target: "terrgen_init", "Unknown fractal_type '{other}' for noise {}", seri.id);
                continue;
            }
        };
        let cellular_distance_function = match seri.cellular_distance_function.trim() {
            "Euclidean" => CellularDistanceFunction::Euclidean,
            "EuclideanSq" => CellularDistanceFunction::EuclideanSq,
            "Manhattan" => CellularDistanceFunction::Manhattan,
            "Hybrid" => CellularDistanceFunction::Hybrid,
            other => {
                error!(target: "terrgen_init", "Unknown cellular_distance_function '{other}' for noise {}", seri.id);
                continue;
            }
        };
        let cellular_return_type = match seri.cellular_return_type.trim() {
            "CellValue" => CellularReturnType::CellValue,
            "Distance" => CellularReturnType::Distance,
            "Distance2" => CellularReturnType::Distance2,
            "Distance2Add" => CellularReturnType::Distance2Add,
            "Distance2Sub" => CellularReturnType::Distance2Sub,
            "Distance2Mul" => CellularReturnType::Distance2Mul,
            "Distance2Div" => CellularReturnType::Distance2Div,
            other => {
                error!(target: "terrgen_init", "Unknown cellular_return_type '{other}' for noise {}", seri.id);
                continue;
            }
        };
        let domain_warp_type = match seri.domain_warp_type.trim() {
            "OpenSimplex2" => DomainWarpType::OpenSimplex2,
            "OpenSimplex2Reduced" => DomainWarpType::OpenSimplex2Reduced,
            "BasicGrid" => DomainWarpType::BasicGrid,
            other => {
                error!(target: "terrgen_init", "Unknown domain_warp_type '{other}' for noise {}", seri.id);
                continue;
            }
        };

        noise.set_noise_type(Some(noise_type));
        noise.set_fractal_type(Some(fractal_type));
        noise.set_fractal_octaves(Some(seri.octaves as i32));
        noise.set_fractal_lacunarity(Some(seri.lacunarity));
        noise.set_fractal_gain(Some(seri.gain));
        noise.set_fractal_weighted_strength(Some(seri.weighted_strength));
        noise.set_fractal_ping_pong_strength(Some(seri.ping_pong_strength));
        noise.set_cellular_distance_function(Some(cellular_distance_function));
        noise.set_cellular_return_type(Some(cellular_return_type));
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
                common::AssetScoped,
                common::ReplicateIfServerStarts,
                FnlNoiseComp { fnl: noise, is_tect: seri.tect },
                ChildOf(holder),
                Terrgen,
                Prefix::trunc("Noise"),
                common::SelectedForHotReload,
            ),
        ));
    }
    cmd.insert_batch(fnl_comps_to_insert);
}
