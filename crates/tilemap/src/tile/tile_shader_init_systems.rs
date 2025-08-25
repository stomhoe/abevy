use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::{RenetClient, RenetServer};
use common::common_components::{DisplayName, EntityPrefix, ImageHolder, ImageHolderMap, StrId};
use game_common::game_common_components::{MyZ, YSortOrigin};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{tile::{tile_components::*, tile_resources::*, tile_materials::*}, };


#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands, asset_server: Res<AssetServer>, 
    mut repeat_tex_handles: ResMut<ShaderRepeatTexSerisHandles>,
    mut repeat_assets: ResMut<Assets<ShaderRepeatTexSeri>>,
    mut voronoi_tex_handles: ResMut<ShaderVoronoiSerisHandles>,
    mut voronoi_assets: ResMut<Assets<ShaderVoronoiSeri>>,
    tileshader_map: Option<Res<TileShaderEntityMap>>,
) {
    if tileshader_map.is_some(){ return; }
    cmd.insert_resource(TileShaderEntityMap::default());
    
    for handle in repeat_tex_handles.handles.drain(..) {
        let Some(seri) = repeat_assets.remove(&handle) else {
          continue;
        };
        info!("Loading Shader from handle: {:?}", handle);

        let str_id = match StrId::new(seri.id.clone(), 4) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create StrId for shader '{}': {}", seri.id, err);
                continue;
            }
        };

        match ImageHolder::new(&asset_server, seri.img_path) {
            Ok(img_holder) => {
                cmd.spawn((
                    str_id,
                    TileShader::TexRepeat(MonoRepeatTextureOverlayMat::new(
                        img_holder, seri.mask_color.into(), seri.scale,
                    )),
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

        let str_id = match StrId::new(seri.id.clone(), 4) {
            Ok(id) => id,
            Err(err) => {
                error!("Failed to create StrId for shader '{}': {}", seri.id, err);
                continue;
            }
        };

        match ImageHolder::new(&asset_server, seri.img_path) {
            Ok(img_holder) => {
                cmd.spawn((
                    str_id,
                    TileShader::Voronoi(VoronoiTextureOverlayMat::new(
                        img_holder, seri.mask_color.into(), seri.scale, seri.voronoi_scale, seri.voronoi_scale_random, seri.voronoi_rotation
                    )),
                ));
            },
            Err(err) => {
                error!("Failed to create ImagePathHolder for shader '{}': {}", str_id, err);
            }
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
            cmd.entity(ent).despawn();
        }
    }
}
