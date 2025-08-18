use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon_renet::renet::RenetClient;
use common::common_components::{DisplayName, EntityPrefix, ImageHolder, ImageHolderMap, StrId};
use game_common::{game_common_components::MyZ, };
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{tile::{tile_components::*, tile_resources::*, tile_materials::*}, };


#[allow(unused_parens)]
pub fn init_shaders(
    mut cmd: Commands, asset_server: Res<AssetServer>, 
    mut repeat_tex_handles: ResMut<ShaderRepeatTexSerisHandles>,
    mut assets: ResMut<Assets<ShaderRepeatTexSeri>>,
    tileshader_map: Option<Res<TileShaderEntityMap>>,
) -> Result {
    let mut result: Result = Ok(());
    if tileshader_map.is_some(){ return Ok(());}
    cmd.insert_resource(TileShaderEntityMap::default());
    
    for handle in repeat_tex_handles.handles.drain(..) {
        if let Some(seri) = assets.remove(&handle) {
            info!("Loading Shader from handle: {:?}", handle);

            let str_id = StrId::new(seri.id, 4)?;

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
                    result = Err(err);
                }
            }
        }
    }

    //FOR PA OTRO SHADER (CREAR OTRO FORMATO DE SHADER SERIALIZED, NO UNIFICARLOS EN UN SOLO FORMATO)
    
    result
}

#[allow(unused_parens)]
pub fn add_shaders_to_map(
    mut cmd: Commands,
    terrgen_map: Option<ResMut<TileShaderEntityMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<TileShader>,)>,
) -> Result {
    let Some(mut tileshader_map) = terrgen_map else {
        return Err(BevyError::from("Failed to get TileShaderEntityMap"));
    };
    let mut result: Result = Ok(());
    for (ent, prefix, str_id) in query.iter() {
        if let Err(err) = tileshader_map.0.insert(str_id, ent, ) {
            error!("{} {} already in TileShaderEntityMap : {}", prefix, str_id, err);
            cmd.entity(ent).despawn();
            result = Err(err);
        }
    }
    result
}

#[allow(unused_parens)]
pub fn init_tiles(
    mut cmd: Commands,  asset_server: Res<AssetServer>,
    mut seris_handles: ResMut<TileSerisHandles>, mut assets: ResMut<Assets<TileSeri>>,
    shader_map: Res<TileShaderEntityMap>,
    tiling_map: Option<Res<TileEntitiesMap>>,
) -> Result {
    if tiling_map.is_some() { return Ok(()); }
    cmd.insert_resource(TileEntitiesMap::default());

    let mut result: Result = Ok(());
    for handle in seris_handles.handles.drain(..) {
        //info!("Loading TileSeri from handle: {:?}", handle);
        if let Some(seri) = assets.remove(&handle) {

            let str_id = StrId::new(seri.id, Tile::MIN_ID_LENGTH)?;
            let my_z = MyZ(seri.z);
            let enti = cmd.spawn((
                Tile, str_id.clone(), Disabled,
                my_z.clone(),
            )).id();

            let [r, g, b, a] = seri.color.unwrap_or([255, 255, 255, 255]);
            let color = Color::srgba_u8(r, g, b, a);

             if ! seri.name.is_empty() {
                cmd.entity(enti).insert(DisplayName(seri.name.clone()));
            }
            if seri.tmapchild {
                cmd.entity(enti).insert(TilemapChild);
            }
           

            if seri.img_paths.is_empty() {
                error!("Tile '{}' has no images", str_id);
                continue;
            }

            //TODO HACER Q LAS TILES PUEDAN TENER MUCHAS IMÁGENES (PARA IR CAMBIANDO ENTRE ELLAS SEGÚN EL ESTADO, USANDO EL INDEX)
            if ! seri.sprite {
                let tile_handles = TileIdsHandles::from_paths(&asset_server, seri.img_paths, );

                let tile_handles = match tile_handles {
                    Ok(tile_handles) => tile_handles,
                    Err(err) => {
                        error!("Failed to create TileHandles for tile '{}': {}", str_id, err);
                        result = Err(err);
                        continue;
                    }
                };

                cmd.entity(enti).insert((
                    TileColor::from(color),
                    tile_handles,
                ));
                if seri.shader.len() > 2 {
                    match shader_map.0.get(&seri.shader) {
                        Ok(shader_ent) => {
                            cmd.entity(enti).insert(TileShaderRef(shader_ent));
                        }
                        Err(err) => {
                            warn!("Tile '{}' references missing shader '{}': {}", str_id, seri.shader, err);
                            result = Err(err);
                        }
                    }
                } else if seri.shader.len() > 0 {
                    warn!("Tile {} shader {} is too short for a shader", str_id, seri.shader);
                }
            }
            else{
                let map = match ImageHolderMap::from_paths(&asset_server, seri.img_paths) {
                    Ok(map) => map,
                    Err(err) => {
                        error!("Failed to create ImageHolderMap for tile '{}': {}", str_id, err);
                        result = Err(err);
                        continue;
                    }
                };

                cmd.entity(enti).insert((
                    Sprite{
                        image: map.first_handle(),
                        color,
                        ..Default::default()
                    },
                    map,
                    Transform::from_translation(Vec2::from_array(seri.offset).extend(my_z.as_float())),
                ));
                if ! seri.shader.is_empty() {
                    warn!("Tile {} tilemap shaders ('{}') are not compatible with sprite=true, ignoring", str_id, seri.shader);
                }
            }

           
        }
    }
    result
} 

pub fn add_tiles_to_map(
    mut cmd: Commands,
    map: Option<ResMut<TileEntitiesMap>>,
    query: Query<(Entity, &EntityPrefix, &StrId), (Added<Tile>, Added<Disabled>, Without<TilePos>)>,
) -> Result {
    let mut result: Result = Ok(());
    if let Some(mut map) = map {
        for (ent, prefix, str_id) in query.iter() {
            if let Err(err) = map.0.insert(str_id, ent, ) {
                error!("{} {} already in TilingEntityMap : {}", prefix, str_id, err);
                cmd.entity(ent).despawn();
                result = Err(err);
            } else {
                info!("Inserted tile '{}' into TilingEntityMap with entity {:?}", str_id, ent);
            }
        }
    }
    result
}

#[allow(unused_parens, )]
pub fn client_map_server_tiling(
    trigger: Trigger<TileEntitiesMap>, 
    client: Option<Res<RenetClient>>,
    mut entis_map: ResMut<ServerEntityMap>, own_map: Res<TileEntitiesMap>,
) {
    if client.is_none() { return; }

    let TileEntitiesMap(received_map) = trigger.event().clone();
    for (hash_id, &server_entity) in received_map.0.iter() {
        if let Ok(client_entity) = own_map.0.get_with_hash(hash_id) {

            debug!("Mapping server entity {:?} to local entity {:?}", server_entity, client_entity);
            entis_map.insert(server_entity, client_entity);
        } else {
            error!("Received entity {:?} with hash id {:?} not found in own map", server_entity, hash_id);
        }
    }
}
