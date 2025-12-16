#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{AssetScoped, DisplayName, EntityPrefix, ImageHolder, ImageHolderMap, ImagePathHolder, MultipleImagePathHolder, StrId};
use crate::tile::{tile_materials::*, tile_resources::*, tile_shader_components::*};

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
    cmd.insert_resource(TileShaderEntityMap::default());
    let holder = cmd.spawn((EguiTileShaderHolder, )).id();

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
                cmd.spawn((
                    str_id,
                    TileShader::TexRepeat(MonoRepeatTextureOverlayMat::new(
                        Handle::default(), seri.mask_color.into(), seri.scale,
                    )),
                    path_holder,
                    ChildOf(holder),
                ));
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
                cmd.spawn((
                    str_id,
                    TileShader::Voronoi(VoronoiTextureOverlayMat::new(
                        Handle::default(), seri.mask_color.into(), seri.scale, seri.voronoi_scale, seri.voronoi_scale_random, seri.voronoi_rotation
                    )),
                    path_holder,
                    ChildOf(holder),
                ));
            },
            Err(err) => {
                error!("Failed to find image path for shader '{}': {}", str_id, err);
            }
        }
    }
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
            let mut handles = Vec::with_capacity(paths.len());
            for path in paths {
                let handle: Handle<Image> = asset_server.load(path);
                handles.push(handle);
            }
            //tile_shader.set_multiple_image_handles(handles);

        }
    }
}


#[allow(unused_parens)]
pub fn add_shaders_to_map(
    mut cmd: Commands,
    tileshader_map: Option<ResMut<TileShaderEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<TileShader>,)>,
) {
    let Some(mut tileshader_map) = tileshader_map else {
        return;
    };
    for (ent, prefix, str_id) in query.iter() {
        if let Err(err) = tileshader_map.0.insert(str_id, ent, ) {
            error!("{} {} already in TileShaderEntityMap : {}", prefix, str_id, err);
            cmd.entity(ent).try_despawn();
        }
    }
}
