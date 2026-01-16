#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::*;
use crate::tile::{tile_materials::*, tile_shader_components::*, tile_shader_resources::*};

#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands, 
    mut repeat_tex_handles: ResMut<ShaderRepeatTexSerisHandles>,
    mut repeat_assets: ResMut<Assets<ShaderRepeatTexSeri>>,
    mut voronoi_tex_handles: ResMut<ShaderVoronoiSerisHandles>,
    mut voronoi_assets: ResMut<Assets<ShaderVoronoiSeri>>,
    tileshader_map: Option<Res<TileShaderEntityMap>>,
) {
    if tileshader_map.is_some(){ return; }
    let mut tileshader_map = TileShaderEntityMap::default();
    let holder = cmd.spawn((EguiTileShaderHolder, )).id();
    let mut shader_comps_to_insert = Vec::new();

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
                    path_holder,
                    ChildOf(holder),
                )));
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
                    path_holder,
                    ChildOf(holder),
                )));
                tileshader_map.0.overwrite(&str_id, ent);
            },
            Err(err) => {
                error!("Failed to find image path for shader '{}': {}", str_id, err);
            }
        }
    }
    cmd.insert_resource(tileshader_map);
    cmd.insert_batch(shader_comps_to_insert);
}

#[allow(unused_parens)]
pub fn add_image_handle_to_tile_shader(
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut TileShader, AnyOf<(&ImagePathHolder, &MultipleImagePathHolder)>),(Or<(Changed<ImagePathHolder>, Changed<MultipleImagePathHolder>)>,)>,
) {
    for (mut tile_shader, (img_path, multiple_img_path)) in query.iter_mut() {
        if let Some(img_path) = img_path {
            let image_handle = asset_server.load(img_path.path());
            tile_shader.set_image_handle(image_handle);
        } else if let Some(multiple_img_path) = multiple_img_path {
            let paths = multiple_img_path.paths();
            let handles: Vec<Handle<Image>> = paths.iter().map(|path| asset_server.load(path)).collect();
            //tile_shader.set_multiple_image_handles(handles);

        }
    }
}