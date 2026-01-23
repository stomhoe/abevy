#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::*;
use crate::tile::tile_shader::{tile_material::prelude::*, tile_shader_components::*, tile_shader_resources::*};


#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands, 
    mut repeat_tex_handles: ResMut<ShaderRepeatTexSerisHandles>,
    mut repeat_assets: ResMut<Assets<ShaderRepeatTexSeri>>,
    mut voronoi_tex_handles: ResMut<ShaderVoroshuSerisHandles>,
    mut voronoi_assets: ResMut<Assets<ShaderVoronoiShuffleSeri>>,
    mut wavy_handles: ResMut<ShaderWavySerisHandles>,
    mut wavy_assets: ResMut<Assets<ShaderWavySeri>>,
    mut tileshader_map: ResMut<TileShaderEntityMap>,
) {
    if !tileshader_map.0.is_empty(){ return; }
    let holder = cmd.spawn((EguiTileShaderHolder, )).id();
    let mut shader_comps_to_insert = Vec::new();
    let mut path_holders_to_insert = Vec::new();

    for handle in repeat_tex_handles.handles.drain(..) {
        let Some(seri) = repeat_assets.remove(&handle) else {
          continue;
        };
        info!("Loading Shader from handle: {:?}", handle);

        let str_id = match StrId::new_with_result(seri.id.clone(), 4) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create StrId for shader '{}': {}", seri.id, err);
                continue;
            }
        };

        match ImagePathHolder::new(seri.img_path) {
            Ok(path_holder) => {
                if let Ok(existing) = tileshader_map.0.get(&str_id) {
                    error!("TileShader '{}' already in TileShaderEntityMap : {:?}", str_id, existing);
                    continue;
                }
                let ent = cmd.spawn_empty().id();
                
                shader_comps_to_insert.push((ent, (
                    str_id.clone(),
                    TileShader::TexRepeat(MonoRepeatTextureOverlayMat::new(
                        Handle::default(), seri.mask_color.into(), seri.scale,
                    )),
                    ChildOf(holder),
                )));
                path_holders_to_insert.push((ent, path_holder));
                tileshader_map.0.overwrite(&str_id, ent);

            },
            Err(err) => {
                error!("Failed to create ImagePathHolder for shader '{}': {}", str_id, err);
            }
        }
    }
    for handle in voronoi_tex_handles.handles.drain(..) {
        let Some(seri) = voronoi_assets.remove(&handle) else {
          continue;
        };
        info!("Loading Shader from handle: {:?}", handle);

        let str_id = match StrId::new_with_result(seri.id.clone(), 4) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create StrId for shader '{}': {}", seri.id, err);
                continue;
            }
        };

        match ImagePathHolder::new(seri.img_path) {
            Ok(path_holder) => {
                if let Ok(existing) = tileshader_map.0.get(&str_id) {
                    error!("TileShader '{}' already in TileShaderEntityMap : {:?}", str_id, existing);
                    continue;
                }
                let ent = cmd.spawn_empty().id();
                
                shader_comps_to_insert.push((ent, (
                    str_id.clone(),
                    TileShader::Voronoi(VoronoiTextureOverlayMat::new(
                        Handle::default(), seri.mask_color.into(), seri.scale, seri.voronoi_scale, seri.voronoi_scale_random, seri.voronoi_rotation
                    )),
                    ChildOf(holder),
                )));
                path_holders_to_insert.push((ent, path_holder));
                tileshader_map.0.overwrite(&str_id, ent);
            },
            Err(err) => {
                error!("Failed to find image path for shader '{}': {}", str_id, err);
            }
        }
    }

    // Wavy shaders (procedural, no image paths)
    for handle in wavy_handles.handles.drain(..) {
        let Some(seri) = wavy_assets.remove(&handle) else { continue; };
        info!("Loading Wavy Shader from handle: {:?}", handle);

        let str_id = match StrId::new_with_result(seri.id.clone(), 4) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create StrId for shader '{}': {}", seri.id, err);
                continue;
            }
        };

        if let Ok(existing) = tileshader_map.0.get(&str_id) {
            error!("TileShader '{}' already in TileShaderEntityMap : {:?}", str_id, existing);
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

            ChildOf(holder),
        )));
        match ImagePathHolder::new(seri.img_path) {
            Ok(path_holder) => {
                path_holders_to_insert.push((ent, path_holder));
            },
            Err(err) => {
                error!("Failed to create ImagePathHolder for wavy shader '{}': {}", str_id, err);
                continue;
            }
        }

        tileshader_map.0.overwrite(&str_id, ent);
    }

    cmd.insert_batch(path_holders_to_insert);
    cmd.insert_batch(shader_comps_to_insert);
}


#[allow(unused_parens)]
pub fn remove_tile_shader_from_map_on_despawn(
    trigger: On<Despawn, TileShader>,
    query: Query<(&StrId),(AnyDisabling)>,
    mut map: ResMut<TileShaderEntityMap>,
) {
    if let Ok(str_id) = query.get(trigger.entity) {
        if let Ok(found_entity) = map.0.get(str_id) {
            if found_entity == trigger.entity {
                map.0.remove(str_id.as_str());
            }
        }
    }
}