#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::*;
use common::log_targets::TILE_SHADER_INIT;
use crate::tile::tile_shader::{TileShaderEntityMap, tile_material::prelude::*, tile_shader_components::*, tile_shader_resources::*};


#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands,
    tileshader_map: Option<Res<TileShaderEntityMap>>,
) {
    if let Some(tileshader_map) = &tileshader_map
        && !tileshader_map.0.is_empty() {
        return;
    }
    let mut shader_comps_to_insert = Vec::new();
    let mut path_holders_to_insert = Vec::new();

    for seri in load_shader_repeat_tex_seri_defs() {
            //trace!(target: TILE_SHADER_INIT, "Loading Shader from handle: {:?}", handle);

            let str_id = match StrId::new_with_result(seri.id.clone(), 1) {
                Ok(id) => id,
                Err(err) => {
                    error!(target: TILE_SHADER_INIT, "Failed to create StrId for shader '{}': {}", seri.id, err);
                    continue;
                }
            };

            match ImagePathHolder::new(seri.img_path) {
                Ok(path_holder) => {

                    let ent = cmd.spawn_empty().id();

                    shader_comps_to_insert.push((ent, (
                        str_id.clone(),
                        TileShader::TexRepeat(MonoRepeatTextureOverlayMat::new(
                            Handle::default(),
                            seri.mask_color.into(),
                            seri.scale,
                            seri.tint_color.unwrap_or([1.0, 1.0, 1.0, 0.0]).into(),
                        )),
                    )));
                    path_holders_to_insert.push((ent, path_holder));

                },
                Err(err) => {
                    error!(target: TILE_SHADER_INIT, "Failed to create ImagePathHolder for shader '{}': {}", str_id, err);
                }
            }
    }
    /*
    if let (Some(voronoi_tex_handles), Some(voronoi_assets)) = (&mut voronoi_tex_handles, &mut voronoi_assets) {
    for handle in voronoi_tex_handles.handles.drain(..) {
        let Some(seri) = voronoi_assets.remove(&handle) else {
            continue;
        };
        trace!(target: TILE_SHADER_INIT, "Loading Shader from handle: {:?}", handle);

        let str_id = match StrId::new_with_result(seri.id.clone(), 4) {
            Ok(id) => id,
            Err(err) => {
                error!(target: TILE_SHADER_INIT, "Failed to create StrId for shader '{}': {}", seri.id, err);
                continue;
            }
        };

        match ImagePathHolder::new(seri.img_path) {
            Ok(path_holder) => {

                let ent = cmd.spawn_empty().id();

                shader_comps_to_insert.push((ent, (
                    str_id.clone(),
                    TileShader::Voronoi(VoronoiTextureOverlayMat::new(
                        Handle::default(), seri.mask_color.into(), seri.scale, seri.voronoi_scale, seri.voronoi_scale_random, seri.voronoi_rotation
                    )),
                )));
                path_holders_to_insert.push((ent, path_holder));
            },
            Err(err) => {
                error!(target: TILE_SHADER_INIT, "Failed to find image path for shader '{}': {}", str_id, err);
            }
        }
    }}
*/
    for seri in load_shader_wavy_seri_defs() {
            //trace!(target: TILE_SHADER_INIT, "Loading Wavy Shader from handle: {:?}", handle);

            let str_id = match StrId::new_with_result(seri.id.clone(), 4) {
                Ok(id) => id,
                Err(err) => {
                    error!(target: TILE_SHADER_INIT, "Failed to create StrId for shader '{}': {}", seri.id, err);
                    continue;
                }
            };

            if let Some(tileshader_map) = &tileshader_map
                && let Ok(existing) = tileshader_map.0.get_cloned(&str_id) {
                error!(target: TILE_SHADER_INIT, "TileShader '{}' already in TileShaderEntityMap : {:?}", str_id, existing);
                continue;
            }
            let ent = cmd.spawn_empty().id();

            shader_comps_to_insert.push((ent, (
                str_id.clone(),
                TileShader::Wavy(WavyMat::new(
                    seri.mask_color.into(),
                    seri.scale,
                    0.0,
                    seri.speed,
                    seri.debug_mode,
                )),
            )));
            match ImagePathHolder::new(seri.img_path) {
                Ok(path_holder) => {
                    path_holders_to_insert.push((ent, path_holder));
                },
                Err(err) => {
                    error!(target: TILE_SHADER_INIT, "Failed to create ImagePathHolder for wavy shader '{}': {}", str_id, err);
                    continue;
                }
            }
    }
    for seri in load_shader_rocky_terrain_seri_defs() {
            //trace!(target: TILE_SHADER_INIT, "Loading Rocky Terrain Shader from handle: {:?}", handle);

            let str_id = match StrId::new_with_result(seri.id.clone(), 4) {
                Ok(id) => id,
                Err(err) => {
                    error!(target: TILE_SHADER_INIT, "Failed to create StrId for shader '{}': {}", seri.id, err);
                    continue;
                }
            };

            if let Some(tileshader_map) = &tileshader_map
                && let Ok(existing) = tileshader_map.0.get_cloned(&str_id) {
                error!(target: TILE_SHADER_INIT, "TileShader '{}' already in TileShaderEntityMap : {:?}", str_id, existing);
                continue;
            }
            let ent = cmd.spawn_empty().id();

            shader_comps_to_insert.push((ent, (
                str_id.clone(),
                TileShader::RockyTerrain(RockyTerrainMat::new(
                    seri.roughness,
                    seri.scale,
                    seri.height_scale,
                    seri.color_base.into(),
                    seri.color_shadow.into(),
                )),
            )));
    }

    cmd.insert_batch(path_holders_to_insert);
    cmd.insert_batch(shader_comps_to_insert);
}
